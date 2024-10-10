#![no_main]

use crypto_bigint::{Encoding, U256};
use risc0_zkvm::{guest::env, serde};
use bitcoin_pow_methods::CALCULATE_POW_ID;
use std::io::Read;
risc0_zkvm::guest::entry!(main);

fn main() {
    let assumption_method_id: [u32; 8] = CALCULATE_POW_ID; // Change this from circuit to circuit.
    let journal_length: u32 = env::read();
    let mut journal_vec = vec![0u8; journal_length as usize + 1];
    for i in 0..journal_length as usize + 1 {
        journal_vec[i as usize] = env::read();
    }
    // env::stdin().read_to_end(&mut journal).unwrap();
    println!("Journal: {:?}", journal_vec);
    println!("Journal length: {:?}", journal_length);
    println!("ASSUMPTION METHOD ID: {:?}", assumption_method_id);


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

    env::verify(assumption_method_id, &journal_vec[1..journal_length as usize+1]).unwrap();
    let digest: [u8; 32] = blake3::hash(&journal_vec).into();
    let mut digest_u32x8: [u32; 8] = [0u32; 8];
    for i in 0..8 {
        digest_u32x8[i] = u32::from_be_bytes([digest[i * 4], digest[i * 4 + 1], digest[i * 4 + 2], digest[i * 4 + 3]]);
    }
    env::commit(&digest_u32x8);
}
