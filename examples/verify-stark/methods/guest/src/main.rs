#![no_main]

use crypto_bigint::{Encoding, U256};
use risc0_zkvm::{guest::env, serde};
use bitcoin_pow_methods::CALCULATE_POW_ID;
use std::io::Read;
risc0_zkvm::guest::entry!(main);

fn main() {
    let assumption_method_id: [u32; 8] = CALCULATE_POW_ID; // NOTE: Change this from circuit to circuit.
    let journal_length: u32 = env::read();
    let mut journal_vec = vec![0u8; journal_length as usize + 1];
    for i in 0..journal_length as usize + 1 {
        journal_vec[i as usize] = env::read();
    }

    println!("Journal: {:?}", journal_vec);
    println!("Journal length: {:?}", journal_length);
    println!("ASSUMPTION METHOD ID: {:?}", assumption_method_id);

    env::verify(assumption_method_id, &journal_vec[1..journal_length as usize+1]).unwrap();
    let digest: [u8; 32] = blake3::hash(&journal_vec).into();
    let mut digest_u32x8: [u32; 8] = [0u32; 8];
    for i in 0..8 {
        digest_u32x8[i] = u32::from_be_bytes([digest[i * 4], digest[i * 4 + 1], digest[i * 4 + 2], digest[i * 4 + 3]]);
    }
    env::commit(&digest_u32x8);
}
