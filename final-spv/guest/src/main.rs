#![no_main]

use final_spv_guest::final_circuit;

risc0_zkvm::guest::entry!(main);
fn main() {
    let zkvm_guest = risc0_to_bitvm2_core::zkvm::Risc0Guest::new();
    final_circuit(&zkvm_guest);
}
