use final_circuit::FinalCircuitInput;
use header_chain::{
    apply_blocks, BlockHeaderCircuitOutput, ChainState, HeaderChainCircuitInput,
    HeaderChainPrevProofType, NETWORK_CONSTANTS, NETWORK_TYPE,
};
use risc0_zkvm::guest::env::{self};
use zkvm::ZkvmGuest;

pub mod header_chain;
pub use risc0_zkvm;
pub mod final_circuit;
pub mod merkle_tree;
pub mod mmr_guest;
pub mod mmr_native;
pub mod spv;
pub mod transaction;
pub mod utils;
pub mod zkvm;

/// The main entry point of the header chain circuit.
pub fn header_chain_circuit(guest: &impl ZkvmGuest) {
    let start = risc0_zkvm::guest::env::cycle_count();

    let input: HeaderChainCircuitInput = guest.read_from_host();
    // println!("Detected network: {:?}", NETWORK_TYPE);
    // println!("NETWORK_CONSTANTS: {:?}", NETWORK_CONSTANTS);
    let mut chain_state = match input.prev_proof {
        HeaderChainPrevProofType::GenesisBlock => ChainState::new(),
        HeaderChainPrevProofType::PrevProof(prev_proof) => {
            assert_eq!(prev_proof.method_id, input.method_id);
            guest.verify(input.method_id, &prev_proof);
            prev_proof.chain_state
        }
    };

    apply_blocks(&mut chain_state, input.block_headers);

    guest.commit(&BlockHeaderCircuitOutput {
        method_id: input.method_id,
        chain_state,
    });
    let end = risc0_zkvm::guest::env::cycle_count();
    println!("Header chain circuit took {:?} cycles", end - start);
}

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            2248912604, 1234851133, 3899523001, 3135677760, 2213900580, 1325314208, 3015256769,
            3245476678,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            3712544376, 1780998879, 2274605232, 4170210407, 3028771271, 1050228934, 2142079250,
            384569559,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            2505747325, 1955876420, 574581856, 3353890285, 3532350318, 525285021, 1929433269,
            4290774758,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            2630284498, 413134112, 1074063885, 2399637541, 2966190734, 346080206, 2302785018,
            689164190,
        ],
        None => [
            2248912604, 1234851133, 3899523001, 3135677760, 2213900580, 1325314208, 3015256769,
            3245476678,
        ],
        _ => panic!("Invalid network type"),
    }
};

/// The final circuit that verifies the output of the header chain circuit.
pub fn final_circuit(guest: &impl ZkvmGuest) {
    let start = env::cycle_count();
    let input: FinalCircuitInput = guest.read_from_host::<FinalCircuitInput>();
    guest.verify(HEADER_CHAIN_GUEST_ID, &input.block_header_circuit_output);
    input.spv.verify(
        input
            .block_header_circuit_output
            .chain_state
            .block_hashes_mmr,
    );
    let mut hasher = blake3::Hasher::new();

    hasher.update(&input.spv.transaction.txid());
    hasher.update(
        &input
            .block_header_circuit_output
            .chain_state
            .best_block_hash,
    );
    hasher.update(&input.block_header_circuit_output.chain_state.total_work);
    let final_output = hasher.finalize();
    guest.commit(final_output.as_bytes());
    let end = env::cycle_count();
    println!("Final circuit took {:?} cycles", end - start);
}
