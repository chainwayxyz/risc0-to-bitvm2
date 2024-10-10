#![doc = include_str!("../README.md")]

use risc0_zkvm::ProverOpts;
use risc0_zkvm::{default_prover, ExecutorEnv, Journal, Receipt};
pub use verify_methods::VERIFY_ELF;
pub use verify_methods::VERIFY_ID;

pub fn verify_stark(
    stark_receipt: Receipt,
    stark_journal: Journal,
    assumption_method_id: [u32; 8],
) -> (Receipt, [u8; 32], [u32; 8]) {
    // Hard-code the general purpose circuit ID for the verify_stark method so that VERIFY_ID has its commitment.
    // Will have to change it from circuit to circuit.

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

    let prover = default_prover();
    let prover_opts = ProverOpts::succinct();
    let verify_receipt = prover.prove_with_opts(env, VERIFY_ELF, &prover_opts).unwrap().receipt;

    let blake3_digest_u32x8: [u32; 8] = verify_receipt.journal.decode().unwrap();
    let mut blake3_digest = [0u8; 32];
    for i in 0..8 {
        blake3_digest[i * 4..(i + 1) * 4].copy_from_slice(&blake3_digest_u32x8[i].to_le_bytes());
    }

    verify_receipt.verify(VERIFY_ID).unwrap();
    return (verify_receipt, blake3_digest, VERIFY_ID);
}
