use core::{final_circuit::FinalCircuitInput, zkvm::ZkvmGuest};

use risc0_zkvm::guest::env;

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            2911241211,
            3315942831,
            1476163032,
            4133049906,
            1138516084,
            1128188034,
            3047146247,
            2247002047,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            315564451,
            2876767614,
            1900410986,
            386182960,
            832946183,
            2025236746,
            1577426007,
            2333543617,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            1305109859,
            1539478049,
            3922864768,
            1185061640,
            4282351879,
            1584032594,
            3983907461,
            2500064781,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            2034575366,
            4001293808,
            196826868,
            2111188001,
            3594700167,
            3443663373,
            2139244664,
            3470578189,
        ],
        None => [
            2911241211,
            3315942831,
            1476163032,
            4133049906,
            1138516084,
            1128188034,
            3047146247,
            2247002047,
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
