#![no_main]
#![no_std]

use crypto_bigint::{Encoding, U256};
use risc0_zkvm::{guest::env, serde};
use bitcoin_pow_methods::CALCULATE_POW_ID;

risc0_zkvm::guest::entry!(main);

fn main() {
    let (pow, last_block_hash): ([u32; 8], [u32; 8]) = env::read();
    let mut pow_bytes = [0u8; 32];
    for i in 0..8 {
        pow_bytes[i * 4..(i + 1) * 4].copy_from_slice(&pow[i].to_le_bytes());
    }
    let mut last_block_hash_bytes = [0u8; 32];
    for i in 0..8 {
        last_block_hash_bytes[i * 4..(i + 1) * 4].copy_from_slice(&last_block_hash[i].to_le_bytes());
    }
    let mut serde_vec = serde::to_vec(&pow).unwrap();
    serde_vec.extend_from_slice(&serde::to_vec(&last_block_hash).unwrap());

    let mut serde_bytes = [0u8; 64];
    serde_bytes[..32].copy_from_slice(&pow_bytes);
    serde_bytes[32..].copy_from_slice(&last_block_hash_bytes);

    env::verify(CALCULATE_POW_ID, &serde_vec).unwrap();
    env::commit(blake3::hash(&serde_bytes).as_bytes());
}
