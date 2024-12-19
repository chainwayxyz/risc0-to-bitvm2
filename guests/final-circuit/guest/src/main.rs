#![no_main]
circuits::risc0_zkvm::guest::entry!(main);
fn main() {
    let zkvm_guest = circuits::zkvm::Risc0Guest::new();
    circuits::final_circuit(&zkvm_guest);
}