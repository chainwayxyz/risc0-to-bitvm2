use core::{final_circuit::FinalCircuitInput, zkvm::ZkvmGuest};

use risc0_zkvm::guest::env;

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            2745490169, 3592696843, 2360627451, 3116212354, 2518257786, 1379639231, 3136173318,
            1530608044,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            2411192457, 1867845907, 3456665098, 906017565, 3314621071, 3369985760, 234242410,
            3031710918,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            494773148, 735687816, 3530073507, 3601966540, 3409541450, 1614932774, 4152818397,
            1895340149,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            264567176, 755347641, 498754075, 3836714448, 3624501550, 2661964142, 2213609525,
            3444337059,
        ],
        None => [
            2745490169, 3592696843, 2360627451, 3116212354, 2518257786, 1379639231, 3136173318,
            1530608044,
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
