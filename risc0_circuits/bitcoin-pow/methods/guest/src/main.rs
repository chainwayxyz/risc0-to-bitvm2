#![no_main]
// #![no_std]

use crypto_bigint::{Encoding, Limb, U256};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

// The minimum amount of work required for a block to be valid (represented as `bits`)
const MAX_BITS: u32 = 0x1d00FFFF;
// "0x00000000FFFF0000000000000000000000000000000000000000000000000000"
const MAX_TARGET: U256 = U256::new([
    Limb(0),
    Limb(0),
    Limb(0),
    Limb(0),
    Limb(0),
    Limb(0),
    Limb(0xFFFF0000),
    Limb(0),
]);
// An epoch should be two weeks (represented as number of seconds)
// seconds/minute * minutes/hour * hours/day * 14 days
const EXPECTED_EPOCH_TIMESPAN: u32 = 60 * 60 * 24 * 14;
// Number of blocks per epoch
const BLOCKS_PER_EPOCH: u32 = 2016;

#[derive(Debug, Clone)]
struct BlockHeader {
    version: i32,
    prev_block_hash: [u8; 32], // The hash of the previous block in little endian form
    merkle_root: [u8; 32],     // The Merkle root of the block's transactions in little endian form
    time: u32,
    bits: u32,
    nonce: u32,
}

#[derive(Debug, Clone)]
struct ChainState {
    block_height: Option<u32>,
    total_work: U256,
    best_block_hash: [u8; 32],
    current_target: U256,  // Maybe just use u32 bits to use less data?
    epoch_start_time: u32, // Represents the time of the first block in the current epoch (the difficulty adjustment timestamp)
    prev_11_timestamps: [u32; 11],
    // network: u32, // Testnet4 (0) or Mainnet (1) NOT IN USE CURRENTLY
}

fn double_sha256_hash(data_parts: &[&[u8]]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();

    // First round of SHA256 hashing
    for data in data_parts {
        hasher.update(data);
    }
    let first_hash_result = hasher.finalize_reset();

    // Second round of SHA256 hashing
    hasher.update(first_hash_result);
    let result: [u8; 32] = hasher
        .finalize()
        .try_into()
        .expect("SHA256 should produce a 32-byte output");
    result
}

fn median(arr: [u32; 11]) -> u32 {
    // Sort the array
    let mut sorted_arr = arr;
    sorted_arr.sort_unstable();

    // Return the middle element
    sorted_arr[5]
}

pub fn validate_timestamp(block_time: u32, prev_11_timestamps: [u32; 11]) {
    let median_time = median(prev_11_timestamps);
    if block_time <= median_time {
        panic!("Block time is not valid");
    }
}

pub fn compact_target_to_bytes(bits: [u8; 4]) -> [u8; 32] {
    let mut bits = bits;
    bits.reverse();

    let mut target = [0u8; 32];
    let exponent = bits[0] as usize;
    let value = ((bits[1] as u32) << 16) | ((bits[2] as u32) << 8) | (bits[3] as u32);

    if exponent <= 3 {
        // If the target size is 3 bytes or less, place the value at the end
        let start_index = 4 - exponent;
        for i in 0..exponent {
            target[31 - i] = (value >> (8 * (start_index + i))) as u8;
        }
    } else {
        // If the target size is more than 3 bytes, place the value at the beginning and shift accordingly
        for i in 0..3 {
            target[exponent - 3 + i] = (value >> (8 * i)) as u8;
        }
    }
    target
}

pub fn decode_compact_target(bits: [u8; 4]) -> U256 {
    let target: [u8; 32] = compact_target_to_bytes(bits);
    U256::from_le_bytes(target)
}

fn check_hash_valid(hash: [u8; 32], target_bytes: [u8; 32]) {
    // println!("Validating hash...");
    // println!("Target bytes: {:?}", target_bytes);
    // println!("Hash: {:?}", hash);
    // for loop from 31 to 0
    for i in (0..32).rev() {
        if hash[i] < target_bytes[i] {
            // The hash is valid because a byte in hash is less than the corresponding byte in target
            return;
        } else if hash[i] > target_bytes[i] {
            // The hash is invalid because a byte in hash is greater than the corresponding byte in target
            panic!("Hash is not valid");
        }
        // If the bytes are equal, continue to the next byte
    }
    // If we reach this point, all bytes are equal, so the hash is valid
}

pub fn validate_threshold(block_header_bits: [u8; 4], block_hash: [u8; 32]) {
    // Step 1: Decode the target from the 'bits' field
    let target = compact_target_to_bytes(block_header_bits);
    // Step 2: Compare the block hash with the target
    check_hash_valid(block_hash, target);
}

fn validate_target(nbits: [u8; 4], current_target: U256) {
    // println!("nbits: {:?}", nbits);
    // println!("current_target: {:?}", current_target);
    let target = decode_compact_target(nbits);
    // println!("target from nbits: {:?}", target);
    if target != current_target {
        // panic!("Target is not valid");
    }
}

pub fn calculate_work(target: U256) -> U256 {
    let target_plus_one = target.saturating_add(&U256::ONE);
    let work = U256::MAX.wrapping_div(&target_plus_one);
    work
}


pub fn add_work(block_header_bits: [u8; 4], old_work: U256) -> U256 {
    let target = decode_compact_target(block_header_bits);
    let work = calculate_work(target);

    old_work.wrapping_add(&work)
}

pub fn calculate_new_difficulty(
    epoch_start_time: u32,
    last_timestamp: u32,
    current_target: U256,
) -> U256 {
    // println!("Calculating new difficulty...");
    // println!("Epoch start time: {:?}", epoch_start_time);
    // println!("Last timestamp: {:?}", last_timestamp);
    // Step 1: Calculate the actual timespan of the epoch
    let mut actual_timespan = last_timestamp - epoch_start_time;
    if actual_timespan < EXPECTED_EPOCH_TIMESPAN / 4 {
        actual_timespan = EXPECTED_EPOCH_TIMESPAN / 4;
    } else if actual_timespan > EXPECTED_EPOCH_TIMESPAN * 4 {
        actual_timespan = EXPECTED_EPOCH_TIMESPAN * 4;
    }
    // println!("Actual timespan: {:?}", actual_timespan);
    // let target = decode_compact_target(nbits);
    // println!("Old Target: {:?}", current_target);
    // Step 2: Calculate the new target
    let mut new_target = current_target;
    new_target = new_target
        .wrapping_mul(&U256::from(actual_timespan))
        .wrapping_div(&U256::from(EXPECTED_EPOCH_TIMESPAN));
    // println!("New target: {:?}", new_target);
    // println!("Max target: {:?}", MAX_TARGET);
    // Step 3: Clamp the new target to the maximum target
    if new_target > MAX_TARGET {
        // println!("Clamping new target to the maximum target");
        new_target = MAX_TARGET;
    }
    // println!("New target after checks: {:?}", new_target);
    let new_target_bits = new_target.bits();
    // println!("New target bits: {:?}", new_target_bits);
    let size = (263 - new_target_bits) / 8;
    // println!("Size: {:?}", size);
    new_target = new_target >> ((30 - size) * 8);
    new_target = new_target << ((30 - size) * 8);
    new_target
}

// fn target_to_compact(target: [u8; 32]) -> [u8; 4] {
//     let target_u256 = U256::from_le_bytes(target);
//     let target_bits = target_u256.bits();
//     let size = (263 - target_bits) / 8;
//     let mut compact_target = [0u8; 4];
//     compact_target[3] = (size + 3) as u8;
//     compact_target[0] = target[31 - size as usize];
//     compact_target[1] = target[30 - size as usize];
//     compact_target[2] = target[29 - size as usize];
//     compact_target
// }

pub fn validate_and_apply_block_header(block_header: BlockHeader, chain_state: &mut ChainState) {
    let new_block_hash = double_sha256_hash(&[
        &block_header.version.to_le_bytes(),
        &block_header.prev_block_hash,
        &block_header.merkle_root,
        &block_header.time.to_le_bytes(),
        &block_header.bits.to_le_bytes(),
        &block_header.nonce.to_le_bytes(),
    ]);
    // println!("New block hash: {:?}", new_block_hash);
    // Step 1: Validate the timestamp
    validate_timestamp(block_header.time, chain_state.prev_11_timestamps);
    // println!("Timestamp is valid");
    // Step 2: Validate the target and add work
    if let Some(block_height) = chain_state.block_height {
        if block_header.time - chain_state.prev_11_timestamps[(block_height % 11) as usize] >= 1201
            && block_header.bits == 486604799
        {
            // println!("Testnet 4 specific block detected"); // This does not have to be true here, but it works
            validate_target(block_header.bits.to_le_bytes(), MAX_TARGET);
        } else {
            validate_target(block_header.bits.to_le_bytes(), chain_state.current_target);
        }
        // println!("Target is valid");
        // println!("Prev Block hash: {:?}", block_header.prev_block_hash);
        let bits_le: [u8; 4] = block_header.bits.to_le_bytes();
        validate_threshold(bits_le, new_block_hash);
        // println!("Threshold is valid");

        chain_state.best_block_hash = new_block_hash;

        let work = add_work(bits_le, chain_state.total_work);
        // println!("Work is added: {:?}", work);

        // Step 3: Update the chain state
        chain_state.total_work = work;

        // Step 4: Update the epoch start time and the previous 11 timestamps
        if block_height % BLOCKS_PER_EPOCH == BLOCKS_PER_EPOCH - 1 {
            chain_state.epoch_start_time = block_header.time;
        }
        chain_state.prev_11_timestamps[(block_height + 1) as usize % 11] = block_header.time;

        // Step 4: Update the current target
        if block_height % BLOCKS_PER_EPOCH == BLOCKS_PER_EPOCH - 2 {
            let new_target = calculate_new_difficulty(
                chain_state.epoch_start_time,
                block_header.time,
                chain_state.current_target,
            );
            // println!("New target: {:?}", new_target);
            chain_state.current_target = new_target;
        }

        chain_state.block_height = Some(block_height + 1);
        // println!("Applied block header for height: {:?}", chain_state.block_height);
    } else {
        // When block_height is None, assume starting from height 0
        let block_height = 0;

        // Same code, but without checking for block_height > 0 since it's 0 now
        validate_target(block_header.bits.to_le_bytes(), chain_state.current_target);

        // println!("Target is valid");
        // println!("Prev Block hash: {:?}", block_header.prev_block_hash);
        validate_threshold(block_header.bits.to_le_bytes(), new_block_hash);
        // println!("Threshold is valid");

        chain_state.best_block_hash = new_block_hash;

        let work = add_work(block_header.bits.to_le_bytes(), chain_state.total_work);
        // println!("Work is added: {:?}", work);

        // Step 3: Update the chain state
        chain_state.total_work = work;

        // Step 4: Update the epoch start time and the previous 11 timestamps
        chain_state.epoch_start_time = block_header.time;
        chain_state.prev_11_timestamps[block_height as usize % 11] = block_header.time;

        chain_state.block_height = Some(0);
        // println!("Applied block header for height: {:?}", chain_state.block_height);
    }
}

fn main() {
    let start = env::cycle_count();
    let mut chain_state = ChainState {
        block_height: None,
        total_work: U256::ZERO,
        best_block_hash: [0u8; 32],
        current_target: MAX_TARGET,
        epoch_start_time: 0,
        prev_11_timestamps: [0u32; 11],
    };
    let prev_proof_type: u32 = env::read(); // To determine whether we are going to start from the beginning, or to verify an existing proof and continue.
    // println!("READ Proof type: {:?}", prev_proof_type);
    if prev_proof_type == 1 { // If we are verifying an existing proof, we are sure that it must be of Mode0.
        let assumption_method_id: [u32; 8] = env::read(); // NOTE: Change this from circuit to circuit.
        println!("READ Assumption method ID: {:?}", assumption_method_id);
        // let journal_length: u32 = env::read();
        let mut journal_slice = [0u8; 148];
        let journal_len: u32 = env::read(); // We will not need this.
        for i in 0..148 {
            journal_slice[i] = env::read();
            // println!("READ Journal slice: {:?}", journal_slice[i]);
        }
        // println!("READ Journal: {:?}", journal_slice);
        env::verify(assumption_method_id, &journal_slice).unwrap();
        // println!("VERIFY ASSUMPTION");
        chain_state.block_height =
            Some(u32::from_le_bytes(journal_slice[0..4].try_into().unwrap()));
        // println!("PREVIOUS Block height: {:?}", chain_state.block_height);
        chain_state.total_work = U256::from_be_bytes(journal_slice[4..36].try_into().unwrap());
        // println!("PREVIOUS Total work: {:?}", chain_state.total_work);
        chain_state.best_block_hash = journal_slice[36..68].try_into().unwrap();
        // println!("PREVIOUS Best block hash: {:?}", chain_state.best_block_hash);
        chain_state.current_target =
            U256::from_le_bytes(journal_slice[68..100].try_into().unwrap());
        // println!("PREVIOUS Current target: {:?}", chain_state.current_target);
        chain_state.epoch_start_time =
            u32::from_le_bytes(journal_slice[100..104].try_into().unwrap());
        // println!("PREVIOUS Epoch start time: {:?}", chain_state.epoch_start_time);
        for i in 0..11 {
            chain_state.prev_11_timestamps[i] =
                u32::from_le_bytes(journal_slice[104 + 4 * i..108 + 4 * i].try_into().unwrap());
        }
        // println!("PREVIOUS Prev 11 timestamps: {:?}", chain_state.prev_11_timestamps);
    }

    let k: u32 = env::read(); // Number of blocks to be read
    // println!("READ Number of blocks: {:?}", k);
    for i in 0..k {
        let curr_version: i32 = env::read();
        // println!("READ Current version: {:?}", curr_version);
        let curr_merkle_root: [u8; 32] = env::read();
        // println!("READ Current merkle root: {:?}", curr_merkle_root);
        let curr_time: u32 = env::read();
        if prev_proof_type == 0 && i == 0 {
            chain_state.epoch_start_time = curr_time;
        }
        // println!("READ Current time: {:?}", curr_time);
        let curr_bits: u32 = env::read();
        // println!("READ Current bits: {:?}", curr_bits);
        let curr_nonce: u32 = env::read();
        // println!("READ Current nonce: {:?}", curr_nonce);
        let curr_block_header = BlockHeader {
            version: curr_version,
            prev_block_hash: chain_state.best_block_hash,
            merkle_root: curr_merkle_root,
            time: curr_time,
            bits: curr_bits,
            nonce: curr_nonce,
        };
        // println!("READ Block header: {:?}", curr_block_header);

        validate_and_apply_block_header(curr_block_header, &mut chain_state);
        // println!("Chain state: {:?}", chain_state);
    }
    let output_type: u32 = env::read();
    // println!("READ Mode: {:?}", output_type);
    if output_type == 0 {
        let mut env_curr_prev_block_hash: [u32; 8] = [0u32; 8];
        for i in 0..8 {
            env_curr_prev_block_hash[i] = (chain_state.best_block_hash[4 * i] as u32)
                + ((chain_state.best_block_hash[4 * i + 1] as u32) << 8)
                + ((chain_state.best_block_hash[4 * i + 2] as u32) << 16)
                + ((chain_state.best_block_hash[4 * i + 3] as u32) << 24);
        }
        let mut env_total_work: [u32; 8] = [0u32; 8];
        let total_work_bytes: [u8; 32] = chain_state.total_work.to_be_bytes();
        for i in 0..8 {
            env_total_work[i] = (total_work_bytes[4 * i] as u32)
                + ((total_work_bytes[4 * i + 1] as u32) << 8)
                + ((total_work_bytes[4 * i + 2] as u32) << 16)
                + ((total_work_bytes[4 * i + 3] as u32) << 24);
        }
    
        let env_target: [Limb; 8] = chain_state.current_target.to_limbs();
        // Outputs:
        env::commit(&chain_state.block_height.unwrap());
        // println!("COMMIT Block height: {:?}", chain_state.block_height);
        env::commit(&env_total_work);
        // println!("COMMIT Total work: {:?}", env_total_work);
        env::commit(&env_curr_prev_block_hash);
        // println!("COMMIT Best block hash: {:?}", env_curr_prev_block_hash);
        for i in 0..8 {
            env::commit(&env_target[i].0);
        }
        // println!("COMMIT Current target: {:?}", chain_state.current_target);
        env::commit(&chain_state.epoch_start_time);
        // println!("COMMIT Epoch start time: {:?}", chain_state.epoch_start_time);
        for i in 0..11 {
            env::commit(&chain_state.prev_11_timestamps[i]);
            // println!("COMMIT Prev 11 timestamps: {:?}", chain_state.prev_11_timestamps[i]);
        }
    } else if output_type == 1 {
        let mut env_curr_prev_block_hash: [u32; 8] = [0u32; 8];
        for i in 0..8 {
            env_curr_prev_block_hash[i] = (chain_state.best_block_hash[4 * i] as u32)
                + ((chain_state.best_block_hash[4 * i + 1] as u32) << 8)
                + ((chain_state.best_block_hash[4 * i + 2] as u32) << 16)
                + ((chain_state.best_block_hash[4 * i + 3] as u32) << 24);
        }
        env::commit(&env_curr_prev_block_hash);
        // println!("COMMIT Best block hash: {:?}", env_curr_prev_block_hash);
    } else if output_type == 2 {
        let k_depth: u32 = env::read();
        // println!("READ K depth: {:?}", k_depth);
        let k_depth_bh = env::read();
        // println!("READ K depth block hash: {:?}", k_depth_bh);
        let mut prev_bh: [u8; 32] = [0u8; 32];
        let mut curr_bh: [u8; 32] = k_depth_bh;
        for i in 0..(k_depth - 1) {
            prev_bh = curr_bh;
            let curr_version: i32 = env::read();
            // println!("READ Current version: {:?}", curr_version);
            let curr_merkle_root: [u8; 32] = env::read();
            // println!("READ Current merkle root: {:?}", curr_merkle_root);
            let curr_time: u32 = env::read();
            // println!("READ Current time: {:?}", curr_time);
            let curr_bits: u32 = env::read();
            // println!("READ Current bits: {:?}", curr_bits);
            let curr_nonce: u32 = env::read();
            // println!("READ Current nonce: {:?}", curr_nonce);
            curr_bh = double_sha256_hash(&[
                &curr_version.to_le_bytes(),
                &prev_bh,
                &curr_merkle_root,
                &curr_time.to_le_bytes(),
                &curr_bits.to_le_bytes(),
                &curr_nonce.to_le_bytes(),
            ]);
            // println!("Calculated block hash: {:?}", curr_bh);
        }
        assert!(curr_bh == chain_state.best_block_hash);
        let mut env_k_depth_bh: [u32; 8] = [0u32; 8];
        for i in 0..8 {
            env_k_depth_bh[i] = (curr_bh[4 * i] as u32)
                + ((curr_bh[4 * i + 1] as u32) << 8)
                + ((curr_bh[4 * i + 2] as u32) << 16)
                + ((curr_bh[4 * i + 3] as u32) << 24);
        }
        let mut env_total_work: [u32; 8] = [0u32; 8];
        let total_work_bytes: [u8; 32] = chain_state.total_work.to_be_bytes();
        for i in 0..8 {
            env_total_work[i] = (total_work_bytes[4 * i] as u32)
                + ((total_work_bytes[4 * i + 1] as u32) << 8)
                + ((total_work_bytes[4 * i + 2] as u32) << 16)
                + ((total_work_bytes[4 * i + 3] as u32) << 24);
        }
        env::commit(&k_depth);
        // println!("COMMIT K depth: {:?}", k_depth);
        env::commit(&env_total_work);
        // println!("COMMIT Total work: {:?}", env_total_work);
        env::commit(&env_k_depth_bh);
        // println!("COMMIT K depth block hash: {:?}", env_k_depth_bh);
    }

    let end = env::cycle_count();
    println!("TOTAL CYCLE COUNT FOR CHUNK: {:?}", end - start);
}
