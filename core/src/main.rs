use borsh::BorshDeserialize;
use circuits::{
    header_chain::{BlockHeader, BlockHeaderCircuitOutput, HeaderChainCircuitInput},
    risc0_zkvm::{default_prover, ExecutorEnv},
};
use header_chain_circuit::{HEADER_CHAIN_GUEST_ELF, HEADER_CHAIN_GUEST_ID};


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
    // Download the headers.bin file from https://zerosync.org/chaindata/headers.bin
    let headers = include_bytes!("../../headers.bin");
    let headers = headers
        .chunks(80)
        .map(|header| BlockHeader::try_from_slice(header).unwrap())
        .collect::<Vec<BlockHeader>>();

    let input = HeaderChainCircuitInput {
        method_id: HEADER_CHAIN_GUEST_ID,
        prev_proof: circuits::header_chain::HeaderChainPrevProofType::GenesisBlock,
        block_headers: headers[..500].to_vec(),
    };

    let env = ExecutorEnv::builder()
        .write_slice(&borsh::to_vec(&input).unwrap())
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove(env, HEADER_CHAIN_GUEST_ELF).unwrap().receipt;

    // Extract journal of receipt
    let output =
        BlockHeaderCircuitOutput::try_from_slice(&receipt.journal.bytes).unwrap();

    println!("Total work: {:#?}", output);

    println!("Proof: {:#?}", receipt);
}
