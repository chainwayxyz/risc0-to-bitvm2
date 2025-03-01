#![no_main]

use risc0_to_bitvm2_core::zkvm;

use header_chain_guest::header_chain_circuit;

risc0_zkvm::guest::entry!(main);
fn main() {
    let zkvm_guest = zkvm::Risc0Guest::new();
    header_chain_circuit(&zkvm_guest);
}