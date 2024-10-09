#![no_main]

use crypto_bigint::{Encoding, U256};
use risc0_zkvm::{guest::env, serde};
use bitcoin_pow_methods::CALCULATE_POW_ID;
use std::io::Read;
risc0_zkvm::guest::entry!(main);

fn main() {
    let journal_length: u32 = env::read();
    let mut journal_vec = vec![0u8; journal_length as usize + 1];
    for i in 0..journal_length as usize + 1 {
        journal_vec[i as usize] = env::read();
    }
    // env::stdin().read_to_end(&mut journal).unwrap();
    println!("Journal: {:?}", journal_vec);
    println!("Journal length: {:?}", journal_length);
    println!("CALCULATE_POW_ID: {:?}", CALCULATE_POW_ID);


    // let mut pow_bytes = [0u8; 32];
    // for i in 0..8 {
    //     pow_bytes[i * 4..(i + 1) * 4].copy_from_slice(&pow[i].to_le_bytes());
    // }
    // let mut last_block_hash_bytes = [0u8; 32];
    // for i in 0..8 {
    //     last_block_hash_bytes[i * 4..(i + 1) * 4].copy_from_slice(&last_block_hash[i].to_le_bytes());
    // }
    // let mut serde_vec = serde::to_vec(&pow).unwrap();
    // serde_vec.extend_from_slice(&serde::to_vec(&last_block_hash).unwrap());

    // let mut serde_bytes = [0u8; 64];
    // serde_bytes[..32].copy_from_slice(&pow_bytes);
    // serde_bytes[32..].copy_from_slice(&last_block_hash_bytes);

    env::verify(CALCULATE_POW_ID, &journal_vec[1..journal_length as usize+1]).unwrap();
    env::commit(blake3::hash(&journal_vec).as_bytes());
}
