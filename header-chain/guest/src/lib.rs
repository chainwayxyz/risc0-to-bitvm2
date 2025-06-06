use risc0_to_bitvm2_core::{header_chain::{BlockHeaderCircuitOutput, ChainState, HeaderChainCircuitInput, HeaderChainPrevProofType}, zkvm::ZkvmGuest};

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

    chain_state.apply_blocks(input.block_headers);

    guest.commit(&BlockHeaderCircuitOutput {
        method_id: input.method_id,
        chain_state,
    });
    let end = risc0_zkvm::guest::env::cycle_count();
    println!("Header chain circuit took {:?} cycles", end - start);
}
