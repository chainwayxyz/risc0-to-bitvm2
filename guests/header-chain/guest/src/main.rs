#![no_main]

use borsh::BorshDeserialize;
use circuits::ZkvmGuest;
use risc0_zkvm::guest::env::{self, Write};
risc0_zkvm::guest::entry!(main);


pub struct Risc0Guest;

impl Risc0Guest {
    pub fn new() -> Self {
        Self {}
    }
}

impl ZkvmGuest for Risc0Guest {
    fn read_from_host<T: borsh::BorshDeserialize>(&self) -> T {
        let mut reader = env::stdin();
        // deserialize
        BorshDeserialize::deserialize_reader(&mut reader)
            .expect("Failed to deserialize input from host")
    }

    fn commit<T: borsh::BorshSerialize>(&self, item: &T) {
        // use risc0_zkvm::guest::env::Write as _;
        let buf = borsh::to_vec(item).expect("Serialization to vec is infallible");
        let mut journal = env::journal();
        journal.write_slice(&buf);
    }

    fn verify(&self, method_id: [u32; 8], journal: &[u32]) {
        env::verify(method_id, journal).unwrap();
    }
}


fn main() {
    let zkvm_guest = Risc0Guest::new();
    circuits::header_chain::header_chain_circuit(&zkvm_guest);
}