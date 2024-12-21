use final_circuit::FinalCircuitInput;
use risc0_zkvm::guest::env;
use zkvm::ZkvmGuest;

pub mod final_circuit;
pub mod merkle_tree;
pub mod spv;
pub mod transaction;
pub mod utils;
pub mod zkvm;
pub use risc0_zkvm;

/// The method ID for the header chain circuit.
const HEADER_CHAIN_GUEST_ID: [u32; 8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => [
            1874050464, 2689887502, 3153099208, 1433217801, 1010795994, 3495614330, 1514631802,
            705530185,
        ],
        Some(network) if matches!(network.as_bytes(), b"testnet4") => [
            3705424756, 543944435, 310444994, 3544614572, 4202379659, 3395254428, 3613679557,
            1505811184,
        ],
        Some(network) if matches!(network.as_bytes(), b"signet") => [
            4011030505, 2215786487, 1131886289, 1783419557, 2838559247, 3313003175, 4197454454,
            4201597713,
        ],
        Some(network) if matches!(network.as_bytes(), b"regtest") => [
            56245057, 2825216973, 1197491796, 3918125544, 3605945036, 3768637786, 1252449105,
            2463404667,
        ],
        None => [
            1874050464, 2689887502, 3153099208, 1433217801, 1010795994, 3495614330, 1514631802,
            705530185,
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
