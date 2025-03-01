use risc0_to_bitvm2_core::{final_circuit::FinalCircuitInput, zkvm::ZkvmGuest};

use risc0_zkvm::guest::env;

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            1805746912, 446248593, 3917403517, 207005930, 1113013022, 885125075, 2943144587,
            596995352,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            2531205848, 135472811, 1335395342, 1429693256, 2485864912, 3110840830, 2084288568,
            2813388680,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            2254695834, 3318328809, 1164031302, 2689739582, 2398216623, 1628823923, 1530308949,
            1679698712,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            2888941100, 2943910977, 2019759508, 3659619392, 3315765939, 1861807077, 2348555341,
            3203242207,
        ],
        None => [
            1805746912, 446248593, 3917403517, 207005930, 1113013022, 885125075, 2943144587,
            596995352,
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
