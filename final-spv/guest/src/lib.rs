use risc0_to_bitvm2_core::{final_circuit::FinalCircuitInput, zkvm::ZkvmGuest};

use risc0_zkvm::guest::env;

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            3412405848, 876686415, 376037299, 1109202095, 1589802329, 2275634755, 3201903354,
            2891398008,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            3831541492, 1914582284, 3478727827, 2650042822, 3245962995, 821199570, 1987041398,
            1979635126,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            1257345173, 920106637, 2280869592, 2880057230, 260112968, 3231228398, 619282301,
            3239265840,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            4000559452, 3695057433, 105444513, 2976837995, 2374114311, 2300056661, 2472162979,
            2106663679,
        ],
        None => [
            3412405848, 876686415, 376037299, 1109202095, 1589802329, 2275634755, 3201903354,
            2891398008,
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
