#![doc = include_str!("../README.md")]

use bitcoin_pow_methods::CALCULATE_POW_ID;
use risc0_zkvm::{default_prover, guest::env, ExecutorEnv, Journal, Receipt};
pub use verify_methods::VERIFY_ELF;
pub use verify_methods::VERIFY_ID;

pub fn verify_stark(
    stark_receipt: Receipt,
    stark_journal: Journal,
    method_id: [u32; 8],
) -> (Receipt, [u8; 32]) {
    //TODO: Pass journal directly to bypass serialization problems

    println!("stark receipt: {:?}", stark_receipt);
    println!("stark journal: {:?}", stark_receipt.journal.bytes);
    println!("journal length: {:?}", stark_journal);

    let env = ExecutorEnv::builder()
        .add_assumption(stark_receipt)
        .write(&(stark_journal.bytes.len() as u32))
        .unwrap()
        .write(&stark_journal)
        .unwrap()
        .build()
        .unwrap();

    let verify_receipt = default_prover().prove(env, VERIFY_ELF).unwrap().receipt;

    let blake3_digest: [u8; 32] = verify_receipt.journal.decode().unwrap();

    verify_receipt.verify(VERIFY_ID).unwrap();
    return (verify_receipt, blake3_digest);
}
