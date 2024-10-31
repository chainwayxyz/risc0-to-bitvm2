use crate::ZkvmGuest;

use borsh::{BorshDeserialize, BorshSerialize};
use crypto_bigint::{Encoding, U256};
use sha2::{Digest, Sha256};

/// The minimum amount of work required for a block to be valid (represented as `bits`)
const MAX_BITS: u32 = 0x1d00FFFF;

const MAX_TARGET: U256 =
    U256::from_be_hex("00000000FFFF0000000000000000000000000000000000000000000000000000");

/// An epoch should be two weeks (represented as number of seconds)
/// seconds/minute * minutes/hour * hours/day * 14 days
const EXPECTED_EPOCH_TIMESPAN: u32 = 60 * 60 * 24 * 14;

/// Number of blocks per epoch
const BLOCKS_PER_EPOCH: u32 = 2016;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct BlockHeader {
    version: i32,
    prev_block_hash: [u8; 32], // The hash of the previous block in little endian form
    merkle_root: [u8; 32],     // The Merkle root of the block's transactions in little endian form
    time: u32,
    bits: u32,
    nonce: u32,
}

impl BlockHeader {
    pub fn compute_block_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new(); // Does this takes time? Can we use a global hasher?
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.prev_block_hash);
        hasher.update(&self.merkle_root);
        hasher.update(&self.time.to_le_bytes());
        hasher.update(&self.bits.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        let first_hash_result = hasher.finalize_reset();

        // Second round of SHA256 hashing
        hasher.update(first_hash_result);
        let result: [u8; 32] = hasher
            .finalize()
            .try_into()
            .expect("SHA256 should produce a 32-byte output");
        result
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct ChainState {
    block_height: u32,
    total_work: [u8; 32],
    best_block_hash: [u8; 32],
    current_target_bits: u32,
    current_target_bytes: [u8; 32],
    epoch_start_time: u32, // Represents the time of the first block in the current epoch (the difficulty adjustment timestamp)
    prev_11_timestamps: [u32; 11],
    // network: u32, // Testnet4 (0) or Mainnet (1) NOT IN USE CURRENTLY
}

fn median(arr: [u32; 11]) -> u32 {
    // Sort the array
    let mut sorted_arr = arr;
    sorted_arr.sort_unstable();

    // Return the middle element
    sorted_arr[5]
}

fn validate_timestamp(block_time: u32, prev_11_timestamps: [u32; 11]) {
    let median_time = median(prev_11_timestamps);
    if block_time <= median_time {
        panic!("Block time is not valid");
    }
}

fn compact_target_to_bytes(nbits: u32) -> [u8; 32] {
    let bits = nbits.to_be_bytes();

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

fn target_to_compact(target: [u8; 32]) -> u32 {
    let target_u256 = U256::from_le_bytes(target);
    let target_bits = target_u256.bits();
    let size = (263 - target_bits) / 8;
    let mut compact_target = [0u8; 4];
    compact_target[3] = (size + 3) as u8;
    compact_target[0] = target[31 - size as usize];
    compact_target[1] = target[30 - size as usize];
    compact_target[2] = target[29 - size as usize];
    u32::from_be_bytes(compact_target)
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

fn calculate_work(target: U256) -> U256 {
    let target_plus_one = target.saturating_add(&U256::ONE);
    let work = U256::MAX.wrapping_div(&target_plus_one);
    work
}

fn add_work(target: [u8; 32], old_work: &[u8; 32]) -> [u8; 32] {
    let target = U256::from_le_bytes(target);
    let work = calculate_work(target);

    U256::from_be_slice(old_work)
        .wrapping_add(&work)
        .to_be_bytes()
}

fn calculate_new_difficulty(
    epoch_start_time: u32,
    last_timestamp: u32,
    current_target: u32,
) -> [u8; 32] {
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
    let new_target_bytes = compact_target_to_bytes(current_target);
    let mut new_target = U256::from_be_bytes(new_target_bytes)
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
    new_target.to_be_bytes()
}

pub fn validate_and_apply_block_header(block_header: BlockHeader, chain_state: &mut ChainState) {
    assert_eq!(block_header.prev_block_hash, chain_state.best_block_hash);

    let new_block_hash = block_header.compute_block_hash();
    // println!("New block hash: {:?}", new_block_hash);
    // Step 1: Validate the timestamp
    validate_timestamp(block_header.time, chain_state.prev_11_timestamps);
    // println!("Timestamp is valid");
    // Step 2: Validate the target and add work
    assert_eq!(block_header.bits, chain_state.current_target_bits);

    check_hash_valid(new_block_hash, chain_state.current_target_bytes);
    // println!("Threshold is valid");

    chain_state.best_block_hash = new_block_hash;

    chain_state.total_work = add_work(chain_state.current_target_bytes, &chain_state.total_work);
    // println!("Work is added: {:?}", work);

    // Step 4: Update the epoch start time and the previous 11 timestamps
    if chain_state.block_height % BLOCKS_PER_EPOCH == BLOCKS_PER_EPOCH - 1 {
        chain_state.epoch_start_time = block_header.time;
    }
    chain_state.prev_11_timestamps[(chain_state.block_height + 1) as usize % 11] =
        block_header.time;

    // Step 4: Update the current target
    if chain_state.block_height % BLOCKS_PER_EPOCH == BLOCKS_PER_EPOCH - 2 {
        let new_target_bytes = calculate_new_difficulty(
            chain_state.epoch_start_time,
            block_header.time,
            chain_state.current_target_bits,
        );

        chain_state.current_target_bytes = new_target_bytes;
        chain_state.current_target_bits = target_to_compact(new_target_bytes);
    }

    chain_state.block_height = chain_state.block_height.wrapping_add(1);
    // println!("Applied block header for height: {:?}", chain_state.block_height);
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct BlockHeaderCircuitOutput {
    method_id: [u32; 8],
    chain_state: ChainState,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum HeaderChainPrevProofType {
    GenesisBlock,
    PrevProof(BlockHeaderCircuitOutput),
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct HeaderChainCircuitInput {
    pub method_id: [u32; 8],
    pub prev_proof: HeaderChainPrevProofType,
    pub block_headers: Vec<BlockHeader>,
}

pub fn header_chain_circuit(guest: &impl ZkvmGuest) {
    let input: HeaderChainCircuitInput = guest.read_from_host();

    let mut chain_state = match input.prev_proof {
        HeaderChainPrevProofType::GenesisBlock => ChainState {
            block_height: u32::MAX,
            total_work: [0u8; 32],
            best_block_hash: [0u8; 32],
            current_target_bits: MAX_BITS,
            current_target_bytes: MAX_TARGET.to_be_bytes(),
            epoch_start_time: 0,
            prev_11_timestamps: [0u32; 11],
        },
        HeaderChainPrevProofType::PrevProof(prev_proof) => {
            assert_eq!(prev_proof.method_id, input.method_id);

            let prev_proof_serialized = borsh::to_vec(&prev_proof).unwrap();
            let mut prev_proof_bytes = [0u32; 148];
            for i in 0..148 {
                prev_proof_bytes[i] = prev_proof_serialized[i] as u32;
            }

            guest.verify(input.method_id, &prev_proof_bytes);
            prev_proof.chain_state
        }
    };

    for block_header in input.block_headers {
        validate_and_apply_block_header(block_header, &mut chain_state);
    }

    BlockHeaderCircuitOutput {
        method_id: input.method_id,
        chain_state,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_hash_calculation() {
        let merkle_root: [u8; 32] =
            hex::decode("3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a")
                .unwrap()
                .try_into()
                .unwrap();
        let block_header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: merkle_root,
            time: 1231006505,
            bits: 486604799,
            nonce: 2083236893,
        };

        let expected_block_hash: [u8; 32] =
            hex::decode("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000")
                .unwrap()
                .try_into()
                .unwrap();
        let block_hash = block_header.compute_block_hash();
        assert_eq!(block_hash, expected_block_hash);
    }
}