#![no_main]
// #![no_std]

use crypto_bigint::{Encoding, Limb, U256};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

// The minimum amount of work required for a block to be valid (represented as `bits`)
const MAX_BITS: u32 = 0x1d00FFFF;
// The minimum amount of work required for a block to be valid (represented as `target`)
const MAX_TARGET: U256 = U256::new([Limb(0), Limb(0xFFFF), Limb(0), Limb(0), Limb(0), Limb(0), Limb(0), Limb(0)]); // "0x00000000FFFF0000000000000000000000000000000000000000000000000000"
// An epoch should be two weeks (represented as number of seconds)
// seconds/minute * minutes/hour * hours/day * 14 days
const EXPECTED_EPOCH_TIMESPAN: u32 = 60 * 60 * 24 * 14;
// Number of blocks per epoch
const BLOCKS_PER_EPOCH: u32 = 2016;

#[derive(Debug, Clone)]
struct BlockHeader {
    version: i32,
    prev_block_hash: [u8; 32],
    merkle_root: [u8; 32],
    time: u32,
    bits: u32,
    nonce: u32,
}

#[derive(Debug, Clone)]
struct ChainState {
    block_height: u32, // TODO: Maybe use u64?
    total_work: U256,
    best_block_hash: [u8; 32],
    current_target: [u8; 32], // Maybe just use u32 bits to use less data?
    epoch_start_time: u32, // Represents the time of the first block in the current epoch (the difficulty adjustment timestamp)
    prev_11_timestamps: [u32; 11],
    // network: u32, // Testnet4 (0) or Mainnet (1)
}

// pub const NUM_BLOCKS: u32 = 10;
// pub const PREV_BLOCK_HASH: [u8; 32] = [
//     111, 226, 140, 10, 182, 241, 179, 114, 193, 166, 162, 70, 174, 99, 247, 79, 147, 30, 131, 101,
//     225, 90, 8, 156, 104, 214, 25, 0, 0, 0, 0, 0,
// ];

macro_rules! double_sha256_hash {
    ($($data:expr),+) => {{
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        // First round of SHA256 hashing
        $(
            hasher.update($data);
        )+
        let first_hash_result = hasher.finalize_reset();
        // Second round of SHA256 hashing
        hasher.update(first_hash_result);
        let result: [u8; 32] = hasher.finalize().try_into().expect("SHA256 should produce a 32-byte output");
        result
    }};
}

fn median(arr: [u32; 11]) -> u32 {
    // Sort the array
    let mut sorted_arr = arr;
    sorted_arr.sort_unstable();

    // Return the middle element
    sorted_arr[5]
}

pub fn validate_timestamp(
    block_time: u32,
    prev_11_timestamps: [u32; 11],
) {
    let median_time = median(prev_11_timestamps);
    if block_time <= median_time {
        panic!("Block time is not valid");
    }
}

pub fn validate_threshold(
    block_header_bits: [u8; 4],
    block_hash: [u8; 32],
) {
    // Step 1: Decode the target from the 'bits' field
    let target = decode_compact_target(block_header_bits);

    // Step 2: Compare the block hash with the target
    check_hash_valid(block_hash, target);
}

pub fn add_work(    block_header_bits: [u8; 4],
    old_work: U256,)  -> U256 {
    let target = decode_compact_target(block_header_bits);
    let work = calculate_work(target);

    old_work.wrapping_add(&work)
}

pub fn calculate_new_difficulty(
    epoch_start_time: u32,
    last_timestamp: u32,
    nbits: [u8; 4],
) -> [u8; 32] {
    // Step 1: Calculate the actual timespan of the epoch
    let mut actual_timespan = epoch_start_time - last_timestamp;
    if actual_timespan < EXPECTED_EPOCH_TIMESPAN / 4 {
        actual_timespan = EXPECTED_EPOCH_TIMESPAN / 4;
    } else if actual_timespan > EXPECTED_EPOCH_TIMESPAN * 4 {
        actual_timespan = EXPECTED_EPOCH_TIMESPAN * 4;
    }
    let target = decode_compact_target(nbits);
    // Step 2: Calculate the new target
    let mut new_target = U256::from_le_bytes(target);
    new_target = new_target
        .wrapping_mul(&U256::from(actual_timespan))
        .wrapping_div(&U256::from(EXPECTED_EPOCH_TIMESPAN));

    // Step 3: Clamp the new target to the maximum target
    if new_target > MAX_TARGET {
        new_target = MAX_TARGET;
    }

    new_target.to_le_bytes()
}

fn validate_target(nbits: [u8; 4], current_target: [u8; 32]) {
    println!("nbits: {:?}", nbits);
    println!("current_target: {:?}", current_target);
    let mut target = decode_compact_target(nbits);
    println!("target from nbits: {:?}", target);
    target.reverse();
    println!("target from nbits reversed: {:?}", target);
    if target != current_target {
        panic!("Target is not valid");
    }
}

pub fn validate_and_apply_block_header(
    block_header: BlockHeader,
    chain_state: &mut ChainState,
 ) {
    // Step 1: Validate the timestamp
    validate_timestamp(block_header.time, chain_state.prev_11_timestamps);

    // Step 2: Calculate new difficulty


    // Step 1: Validate the target and add work
    validate_target(block_header.bits.to_le_bytes(), chain_state.current_target);
    validate_threshold(
        block_header.bits.to_le_bytes(),
        block_header.prev_block_hash,
    );

    let work = add_work(
        block_header.bits.to_le_bytes(),
        chain_state.total_work,
    );

    // Step 2: Update the chain state
    chain_state.block_height += 1;
    chain_state.total_work = work;
    chain_state.best_block_hash = double_sha256_hash!(
        &block_header.version.to_le_bytes(),
        &block_header.prev_block_hash,
        &block_header.merkle_root,
        &block_header.time.to_le_bytes(),
        &block_header.bits.to_le_bytes(),
        &block_header.nonce.to_le_bytes()
    );

    // Step 3: Update the epoch start time and the previous 11 timestamps
    if chain_state.block_height % BLOCKS_PER_EPOCH == 0 {
        chain_state.epoch_start_time = block_header.time;
    }
    chain_state.prev_11_timestamps[chain_state.block_height as usize % 11] =
    block_header.time;

    // Step 4: Update the current target
    if chain_state.block_height % 2016 == 0 {
        // Calculate the new target
        let new_target = calculate_new_difficulty(
            chain_state.epoch_start_time,
            block_header.time,
            block_header.bits.to_le_bytes(),
        );
        chain_state.current_target = new_target;
    }
}

pub fn decode_compact_target(bits: [u8; 4]) -> [u8; 32] {
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

fn check_hash_valid(hash: [u8; 32], target: [u8; 32]) {
    // for loop from 31 to 0
    for i in (0..32).rev() {
        if hash[i] < target[i] {
            // The hash is valid because a byte in hash is less than the corresponding byte in target
            return;
        } else if hash[i] > target[i] {
            // The hash is invalid because a byte in hash is greater than the corresponding byte in target
            panic!("Hash is not valid");
        }
        // If the bytes are equal, continue to the next byte
    }
    // If we reach this point, all bytes are equal, so the hash is valid
}

pub fn calculate_work(target: [u8; 32]) -> U256 {
    let target_plus_one = U256::from_le_bytes(target).saturating_add(&U256::ONE);
    let work = U256::MAX.wrapping_div(&target_plus_one);
    work
}

fn main() {
    let mut chain_state = ChainState {
        block_height: 0,
        total_work: U256::ZERO,
        best_block_hash: [0u8; 32],
        current_target: MAX_TARGET.to_le_bytes(),
        epoch_start_time: 0,
        prev_11_timestamps: [0u32; 11],
    };
    let proof_type: u32 = env::read(); // To determine whether we are going to start from the beginning, or to verify an existing proof and continue.
    println!("READ Proof type: {:?}", proof_type);
    if proof_type == 1 {
        let assumption_method_id: [u32; 8] = env::read(); // NOTE: Change this from circuit to circuit.
        // let journal_length: u32 = env::read();
        let mut journal_slice = [0u8; 148];
        let journal_len: u32 = env::read(); // We will not need this.
        for i in 0..148 {
            journal_slice[i] = env::read();
            println!("READ Journal slice: {:?}", journal_slice[i]);
        }
        println!("READ Journal: {:?}", journal_slice);
        env::verify(assumption_method_id, &journal_slice).unwrap();
        println!("VERIFY ASSUMPTION");
        chain_state.block_height = u32::from_le_bytes(journal_slice[0..4].try_into().unwrap());
        chain_state.total_work = U256::from_le_bytes(journal_slice[4..36].try_into().unwrap());
        chain_state.best_block_hash = journal_slice[36..68].try_into().unwrap();
        chain_state.current_target = journal_slice[68..100].try_into().unwrap();
        chain_state.epoch_start_time = u32::from_le_bytes(journal_slice[100..104].try_into().unwrap());
        for i in 0..11 {
            chain_state.prev_11_timestamps[i] = u32::from_le_bytes(journal_slice[104 + 4 * i..108 + 4 * i].try_into().unwrap());
        }
        
    }

    let k: u32 = env::read(); // Number of blocks to be read
    println!("READ Number of blocks: {:?}", k);
    for _ in 0..k {
        let curr_version: i32 = env::read();
        println!("READ Current version: {:?}", curr_version);
        let curr_merkle_root: [u8; 32] = env::read();
        println!("READ Current merkle root: {:?}", curr_merkle_root);
        let curr_time: u32 = env::read();
        println!("READ Current time: {:?}", curr_time);
        let curr_bits: u32 = env::read();
        println!("READ Current bits: {:?}", curr_bits);
        let curr_nonce: u32 = env::read();
        println!("READ Current nonce: {:?}", curr_nonce);
        let curr_block_header = BlockHeader {
            version: curr_version,
            prev_block_hash: chain_state.best_block_hash,
            merkle_root: curr_merkle_root,
            time: curr_time,
            bits: curr_bits,
            nonce: curr_nonce,
        };
        println!("READ Block header: {:?}", curr_block_header);

        validate_and_apply_block_header(
            curr_block_header,
            &mut chain_state,
        );
        println!("Chain state: {:?}", chain_state);
    }

    let mut env_curr_prev_block_hash: [u32; 8] = [0u32; 8];
    for i in 0..8 {
        env_curr_prev_block_hash[i] = (chain_state.best_block_hash[4 * i] as u32) + ((chain_state.best_block_hash[4 * i + 1] as u32) << 8) + ((chain_state.best_block_hash[4 * i + 2] as u32) << 16) + ((chain_state.best_block_hash[4 * i + 3] as u32) << 24);
    }
    let mut env_total_work: [u32; 8] = [0u32; 8];
    let total_work_bytes: [u8; 32] = chain_state.total_work.to_be_bytes();
    for i in 0..8 {
        env_total_work[i] = (total_work_bytes[4 * i] as u32) + ((total_work_bytes[4 * i + 1] as u32) << 8) + ((total_work_bytes[4 * i + 2] as u32) << 16) + ((total_work_bytes[4 * i + 3] as u32) << 24);
    }
    let mut env_target: [u32; 8] = [0u32; 8];
    for i in 0..8 {
        env_target[i] = (chain_state.current_target[4 * i] as u32) + ((chain_state.current_target[4 * i + 1] as u32) << 8) + ((chain_state.current_target[4 * i + 2] as u32) << 16) + ((chain_state.current_target[4 * i + 3] as u32) << 24);
    }
    // Outputs:
    env::commit(&chain_state.block_height);
    println!("COMMIT Block height: {:?}", chain_state.block_height);
    env::commit(&env_total_work);
    println!("COMMIT Total work: {:?}", env_total_work);
    env::commit(&env_curr_prev_block_hash);
    println!("COMMIT Best block hash: {:?}", env_curr_prev_block_hash);
    env::commit(&env_target);
    println!("COMMIT Current target: {:?}", chain_state.current_target);
    env::commit(&chain_state.epoch_start_time);
    println!("COMMIT Epoch start time: {:?}", chain_state.epoch_start_time);
    for i in 0..11 {
        env::commit(&chain_state.prev_11_timestamps[i]);
        println!("COMMIT Prev 11 timestamps: {:?}", chain_state.prev_11_timestamps[i]);
    }

}

