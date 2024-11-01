use std::io::Read;

use borsh::BorshDeserialize;
use risc0_zkvm::guest::env::{self, Write};

pub mod header_chain;
pub use risc0_zkvm;

pub trait ZkvmGuest {
    fn read_from_host<T: borsh::BorshDeserialize>(&self) -> T;
    fn commit<T: borsh::BorshSerialize>(&self, item: &T);
    fn verify(&self, method_id: [u32; 8], journal: &[u32]);
}

#[derive(Debug)]
pub struct Proof {
    pub method_id: [u32; 8],
    pub journal: Vec<u8>,
}

pub trait ZkvmHost {
    // Adding data to the host
    fn write<T: borsh::BorshSerialize>(&self, value: &T);

    fn add_assumption(&self, proof: Proof);

    // Proves with the given data
    fn prove(&self, elf: &[u32]) -> Proof;
}

pub struct Risc0Guest;

impl Risc0Guest {
    pub fn new() -> Self {
        Self {}
    }
}

impl ZkvmGuest for Risc0Guest {
    fn read_from_host<T: borsh::BorshDeserialize>(&self) -> T {
        let mut reader = env::stdin();
        BorshDeserialize::deserialize_reader(&mut reader).expect("Failed to deserialize input from host")
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
