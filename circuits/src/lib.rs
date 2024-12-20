use final_circuit::FinalCircuitInput;
use header_chain::{
    apply_blocks, BlockHeaderCircuitOutput, ChainState, HeaderChainCircuitInput, HeaderChainPrevProofType, NETWORK_CONSTANTS, NETWORK_TYPE
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
    println!("Detected network: {:?}", NETWORK_TYPE);
    println!("NETWORK_CONSTANTS: {:?}", NETWORK_CONSTANTS);
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
            2582868723, 1521080230, 649720670, 1875083250, 2004955238, 1828385669, 4236861372,
            3439193906,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            3734931792, 1866803528, 3946226235, 2178008193, 2076699023, 2013177735, 3599336517,
            2600457464,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            782875464, 3409446944, 1399471818, 163718061, 1363437585, 1238883005, 1901210571,
            1552828871,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            3144683151, 589096142, 2457639393, 3546468872, 4119149927, 2741065617, 2668617476,
            432086584,
        ],
        None => [
            2582868723, 1521080230, 649720670, 1875083250, 2004955238, 1828385669, 4236861372,
            3439193906,
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
