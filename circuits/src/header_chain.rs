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

/// https://learnmeabitcoin.com/technical/block/#bits
fn bits_to_target(bits: u32) -> [u8; 32] {
    let size = (bits >> 24) as usize;
    let word = bits & 0x00ffffff;

    // Prepare U256 target
    let target = if size <= 3 {
        U256::from(word >> (8 * (3 - size)))
    } else {
        U256::from(word) << (8 * (size - 3))
    };

    target.to_be_bytes()
}

fn target_to_bits(target: &[u8; 32]) -> u32 {
    let target_u256 = U256::from_be_slice(target);

    // Clamp target if it exceeds the maximum target
    let mut clamped_target = target_u256;
    if clamped_target > MAX_TARGET {
        clamped_target = MAX_TARGET;
    }

    // Determine the size based on the bit length of the target
    let size = ((clamped_target.bits() + 7) / 8) as u32;

    // Calculate the compact representation
    let mut compact: u32 = if size <= 3 {
        // Right-shift and convert the lower 3 bytes
        let shifted = clamped_target >> (8 * (3 - size)) as usize;
        let bytes = shifted.to_be_bytes();
        u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])
    } else {
        // Right-shift to fit into 3 bytes and mask for the mantissa
        let shifted = clamped_target >> (8 * (size - 3)) as usize;
        let bytes = shifted.to_be_bytes();
        u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]) & 0x007fffff
    };

    // Adjust if compact exceeds mantissa limits
    if compact & 0x00800000 != 0 {
        compact >>= 8;
    }

    // Return the final compact bits
    (size << 24) | compact
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
    let new_target_bytes = bits_to_target(current_target);
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
        chain_state.current_target_bits = target_to_bits(&new_target_bytes);
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
    use hex_literal::hex;

    // From block 800000 to 800015
    const BLOCK_HEADERS: [[u8; 80]; 15] = [
        hex!("00601d3455bb9fbd966b3ea2dc42d0c22722e4c0c1729fad17210100000000000000000055087fab0c8f3f89f8bcfd4df26c504d81b0a88e04907161838c0c53001af09135edbd64943805175e955e06"),
        hex!("00a0012054a02827d7a8b75601275a160279a3c5768de4c1c4a702000000000000000000394ddc6a5de035874cfa22167bfe923953187b5a19fbb84e186dea3c78fd871c9bedbd6494380517f5c93c8c"),
        hex!("00a00127470972473293f6a3514d69d9ede5acc79ef19c236be20000000000000000000035aa0cba25ae1517a257d8c913e24ec0a152fd6a84b7f9ef303626c91cdcd6b287efbd649438051761ba50fb"),
        hex!("00e0ff3fe80aaef89174e3668cde4cefecae739cd2f337251e12050000000000000000004ed17162f118bd27ae283be8dabe8afe7583bd353087e2eb712c48e3c3240c3ea3efbd64943805178e55bb2f"),
        hex!("00006020f9dd40733234ec3084fa55ae955d2e95f63db75382b4030000000000000000006f440ea93df1e46fa47a6135ce1661cbdb80e703e4cfb6d2c0bcf49ea50f2a1530f5bd64943805175d3a7efb"),
        hex!("0040f526ba869c2271583b645767c3bc4acee3f4a5a1ac727d07050000000000000000006ce5ff483f5e9fe028725bd30196a064a761b3ea831e5b81cf1473d5aa11810efbf6bd64943805174c75b45d"),
        hex!("0000c0204d770ec7842342bcfebba4447545383c639294a6c10c0500000000000000000059f61d610ef6cbcc1d05dec4ebc5e744c62dc975c4256c5f95833d350303c05521fabd64943805172b8e799e"),
        hex!("00400020d9ea5216f276b3f623834e8db837f8b41a8afbda6e8800000000000000000000d5dea9ae25f7f8e6d66064b21c7f5d1481d08d162658785fde59716b1bf98ff50505be6494380517a33ee2b0"),
        hex!("0060262ebabd5319d7013811214809650a635c974444813935b203000000000000000000a0ab544e5055c443256debb20e85f8ded28f746436a57c00e914b9fd02ff058bcf07be64943805172436ed21"),
        hex!("00000020455bd24740ceb627a3c41c3cecaf097b45779719b0d40400000000000000000043ad55fc5619dd8f2edd7d18212d176cdb6aa2152f12addf9d38c9c29be0da60030bbe649438051704743edc"),
        hex!("00e0ff27d53e9a409bf8ce3054862f76d926437c1b1a84ce1ac0010000000000000000004fceebb8a6cee0eaba389e462ae6bb89a8e6dd5396eeba89dc5907ff51112e21760dbe64943805174bd6f6f6"),
        hex!("00e0ff3ff9d1af6c7009b9974b4d838a2505bc882a6333f92500030000000000000000002dff4798432eb3beaf3e5b7c7ca318c1b451ba05c560473b6b974138ac73a82f2b0ebe6494380517d26b2853"),
        hex!("00403a31ee9197174b65726fa7d78fe8b547c024519642009b4f0100000000000000000025f09dbf49cabe174066ebc2d5329211bd994a2b645e4086cadc5a2bbe7cac687e0ebe64943805171f930c95"),
        hex!("0000eb2f06d50bd6ead9973ec74d9f5d77aa9cc6262a497b7ef5040000000000000000004918ae9062a90bfc4c2befca6eb0569c86b53f20bfae39c14d56052eef74f39e2110be64943805176269f908"),
        hex!("00a0002049b01d8eea4b9d88fabd6a9633699c579145a8ddc91205000000000000000000368d0d166ae485674d0b794a8e2e2f4e94ac1e5b6d56612b3d725bc793f523514712be6494380517860d95e4")
    ];

    #[test]
    fn test_block_hash_calculation() {
        let merkle_root = hex!("3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a");
        let expected_block_hash =
            hex!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000");

        let block_header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: merkle_root,
            time: 1231006505,
            bits: 486604799,
            nonce: 2083236893,
        };

        let block_hash = block_header.compute_block_hash();
        assert_eq!(block_hash, expected_block_hash);
    }

    #[test]
    fn test_15_block_hash_calculation() {
        let block_headers = BLOCK_HEADERS
            .iter()
            .map(|header| BlockHeader::try_from_slice(header).unwrap())
            .collect::<Vec<BlockHeader>>();

        for i in 0..block_headers.len() - 1 {
            let block_hash = block_headers[i].compute_block_hash();
            let next_block = &block_headers[i + 1];
            assert_eq!(block_hash, next_block.prev_block_hash);
        }
    }

    #[test]
    fn test_median() {
        let arr = [3, 7, 2, 10, 1, 5, 9, 4, 8, 6, 11];
        assert_eq!(median(arr), 6);
    }

    #[test]
    fn test_timestamp_checks() {
        let block_headers = BLOCK_HEADERS
            .iter()
            .map(|header| BlockHeader::try_from_slice(header).unwrap())
            .collect::<Vec<BlockHeader>>();

        let first_11_timestamps = block_headers[..11]
            .iter()
            .map(|header| header.time)
            .collect::<Vec<u32>>();

        validate_timestamp(
            block_headers[11].time,
            first_11_timestamps.clone().try_into().unwrap(),
        );

        // The second validation is expected to panic
        let result = std::panic::catch_unwind(|| {
            validate_timestamp(
                block_headers[1].time,
                first_11_timestamps.try_into().unwrap(),
            );
        });

        assert!(
            result.is_err(),
            "Expected the second validation to panic, but it did not."
        );
    }

    #[test]
    fn test_target_conversion() {
        let block_headers = BLOCK_HEADERS
            .iter()
            .map(|header| BlockHeader::try_from_slice(header).unwrap())
            .collect::<Vec<BlockHeader>>();

        for header in block_headers {
            let compact_target = bits_to_target(header.bits);
            let nbits = target_to_bits(&compact_target);
            assert_eq!(nbits, header.bits);
        }
    }

    #[test]
    fn test_bits_to_target() {
        // https://learnmeabitcoin.com/explorer/block/00000000000000000002ebe388cb8fa0683fc34984cfc2d7d3b3f99bc0d51bfd
        let expected_target = hex!("00000000000000000002f1280000000000000000000000000000000000000000");
        let bits: u32 = 0x1702f128;
        let target = bits_to_target(bits);
        assert_eq!(target, expected_target);

        let converted_bits = target_to_bits(&target);

        println!("Original bits: {:?}", bits);
        assert_eq!(converted_bits, bits);
    }
}
