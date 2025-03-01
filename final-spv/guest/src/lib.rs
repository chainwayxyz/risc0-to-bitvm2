use risc0_to_bitvm2_core::{final_circuit::FinalCircuitInput, zkvm::ZkvmGuest};

use risc0_zkvm::guest::env;

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            2983458985, 2161501593, 359976410, 419030338, 1687870493, 1538890916, 4123667265,
            2550671961,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            2582274947, 2839630431, 2154066918, 859640159, 1252432262, 3754744755, 421317738,
            1438302269,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            2095596628, 1385070286, 2911707900, 3778482028, 1721857715, 1966814918, 1697275343,
            472879915,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            1963755986, 2115339117, 816900315, 3049436998, 617625294, 3727118749, 961073940,
            3008676475,
        ],
        None => [
            2983458985, 2161501593, 359976410, 419030338, 1687870493, 1538890916, 4123667265,
            2550671961,
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
