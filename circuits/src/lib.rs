pub mod header_chain;

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

