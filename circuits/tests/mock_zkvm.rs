use circuits::{Proof, ZkvmGuest, ZkvmHost};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct ZkvmData {
    values: Vec<u8>,
    journal: Vec<u8>,
}

#[derive(Default)]
pub struct MockZkvmHost {
    data: Arc<Mutex<ZkvmData>>,
}

impl MockZkvmHost {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ZkvmGuest for MockZkvmHost {
    fn read_from_host<T: borsh::BorshDeserialize>(&self) -> T {
        let data = self.data.lock().unwrap();
        return T::try_from_slice(&data.values).unwrap();
    }
    fn commit<T: borsh::BorshSerialize>(&self, item: &T) {
        let mut data = self.data.lock().unwrap();
        let value = borsh::to_vec(item).unwrap();
        data.journal.extend_from_slice(&value);
    }

    fn verify(&self, _method_id: [u32; 8], _journal: &[u32]) {}
}

impl ZkvmHost for MockZkvmHost {
    fn write<T: borsh::BorshSerialize>(&self, value: &T) {
        let mut data = self.data.lock().unwrap();
        let value = borsh::to_vec(value).unwrap();
        data.values.extend_from_slice(&value);
    }
    fn prove(&self, _elf: &[u32]) -> Proof {
        let data = self.data.lock().unwrap();
        Proof {
            method_id: [42; 8],
            journal: data.journal.clone(),
        }
    }

    fn add_assumption(&self, _proof: Proof) {}
}
