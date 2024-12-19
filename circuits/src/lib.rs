use final_circuit::FinalCircuitInput;
use header_chain::{apply_blocks, BlockHeaderCircuitOutput, ChainState, HeaderChainCircuitInput, HeaderChainPrevProofType};
use risc0_zkvm::guest::env::{self};
use zkvm::ZkvmGuest;

pub mod header_chain;
pub use risc0_zkvm;
pub mod merkle_tree;
pub mod mmr_guest;
pub mod mmr_native;
pub mod spv;
pub mod transaction;
pub mod utils;
pub mod zkvm;
pub mod final_circuit;

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
            3698181899, 2493383995, 2645667453, 2581037551, 2903196337, 1963019158, 1819266610,
            2816371110,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            2054544620, 3893068247, 1562334679, 3817290253, 716342840, 155273260, 3527496151,
            3465943753,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            3135044400, 3159803097, 153443868, 2355119596, 521371102, 287799635, 3711739625,
            229413818,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            2566738292, 148345840, 1410711648, 1906697482, 1586982940, 3767445383, 3240218910,
            4082615392,
        ],
        None => [
            3698181899, 2493383995, 2645667453, 2581037551, 2903196337, 1963019158, 1819266610,
            2816371110,
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