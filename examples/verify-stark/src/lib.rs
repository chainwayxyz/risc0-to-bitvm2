#![doc = include_str!("../README.md")]

use risc0_zkvm::{default_prover, guest::env, ExecutorEnv, Journal, Receipt};
use bitcoin_pow_methods::CALCULATE_POW_ID;
pub use verify_methods::VERIFY_ID;
pub use verify_methods::VERIFY_ELF;

pub fn verify_stark(pow_receipt: Receipt, pow: [u32; 8], last_block_hash: [u32; 8], method_id: [u32; 8]) -> (Receipt, [u8; 32]) {
    let env = ExecutorEnv::builder()
        .add_assumption(pow_receipt)
        .write(&(pow, last_block_hash))
        .unwrap()
        .build()
        .unwrap();

    let verify_receipt = default_prover()
    .prove(env, VERIFY_ELF)
    .unwrap()
    .receipt;

    let blake3_digest: [u8; 32] = verify_receipt.journal.decode().unwrap();

    verify_receipt.verify(VERIFY_ID).unwrap();
    return (verify_receipt, blake3_digest);
}

