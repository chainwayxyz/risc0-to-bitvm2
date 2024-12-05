use borsh::BorshDeserialize;
use circuits::{
    header_chain::{
        BlockHeaderCircuitOutput, CircuitBlockHeader, HeaderChainCircuitInput,
        HeaderChainPrevProofType,
    },
    risc0_zkvm::{default_prover, ExecutorEnv},
};

use risc0_circuit_recursion::control_id::BN254_IDENTITY_CONTROL_ID;
use risc0_zkvm::{compute_image_id, sha::Digestible};
use risc0_zkvm::{ProverOpts, Receipt, SuccinctReceiptVerifierParameters, SystemState};
use sha2::Digest;
use sha2::Sha256;
use std::{env, fs};

pub mod docker;

const HEADER_CHAIN_GUEST_ELF: &[u8; 211744] = include_bytes!(
    "../../target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/header_chain_guest/header-chain-guest"
);

const HEADERS: &[u8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => {
            include_bytes!("../../mainnet-headers.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"testnet4") => {
            include_bytes!("../../testnet4-headers.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"signet") => {
            include_bytes!("../../signet-headers.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"regtest") => {
            include_bytes!("../../regtest-headers.bin")
        }
        None => include_bytes!("../../mainnet-headers.bin"),
        _ => panic!("Invalid network type"),
    }
};

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: <program> <input_proof> <output_file_path> <batch_size>");
        return;
    }

    let input_proof = &args[1];
    let output_file_path = &args[2];
    let batch_size: usize = args[3].parse().expect("Batch size should be a number");

    let headers = HEADERS
        .chunks(80)
        .map(|header| CircuitBlockHeader::try_from_slice(header).unwrap())
        .collect::<Vec<CircuitBlockHeader>>();

    let HEADER_CHAIN_GUEST_ID: [u32; 8] = compute_image_id(HEADER_CHAIN_GUEST_ELF)
        .unwrap()
        .as_words()
        .try_into()
        .unwrap();

    // Set the previous proof type based on input_proof argument
    let prev_receipt = if input_proof.to_lowercase() == "none" {
        None
    } else {
        let proof_bytes = fs::read(input_proof).expect("Failed to read input proof file");
        let receipt: Receipt = Receipt::try_from_slice(&proof_bytes).unwrap();
        Some(receipt)
    };

    let mut start = 0;
    let prev_proof = match prev_receipt.clone() {
        Some(receipt) => {
            let output =
                BlockHeaderCircuitOutput::try_from_slice(&receipt.journal.bytes.clone()).unwrap();
            start = output.chain_state.block_height as usize + 1;
            HeaderChainPrevProofType::PrevProof(output)
        }
        None => HeaderChainPrevProofType::GenesisBlock,
    };

    // Prepare the input for the circuit
    let input = HeaderChainCircuitInput {
        method_id: HEADER_CHAIN_GUEST_ID,
        prev_proof,
        block_headers: headers[start..start + batch_size].to_vec(),
    };

    // Build ENV
    let mut binding = ExecutorEnv::builder();
    let mut env = binding.write_slice(&borsh::to_vec(&input).unwrap());
    if let Some(receipt) = prev_receipt {
        env = env.add_assumption(receipt);
    }
    let env = env.build().unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover
        .prove_with_opts(env, HEADER_CHAIN_GUEST_ELF, &ProverOpts::succinct())
        .unwrap()
        .receipt;

    // Extract journal of receipt
    let output = BlockHeaderCircuitOutput::try_from_slice(&receipt.journal.bytes).unwrap();

    println!("Output: {:#?}", output.method_id);

    // Save the receipt to the specified output file path
    let receipt_bytes = borsh::to_vec(&receipt).unwrap();
    fs::write(output_file_path, &receipt_bytes).expect("Failed to write receipt to output file");
    println!("Receipt saved to {}", output_file_path);
}

/// Sha256(control_root, pre_state_digest, post_state_digest, id_bn254_fr)
pub fn calculate_succinct_output_prefix(method_id: &[u8]) -> [u8; 32] {
    let succinct_verifier_params = SuccinctReceiptVerifierParameters::default();
    let succinct_control_root = succinct_verifier_params.control_root;
    let mut succinct_control_root_bytes: [u8; 32] =
        succinct_control_root.as_bytes().try_into().unwrap();
    for byte in succinct_control_root_bytes.iter_mut() {
        *byte = byte.reverse_bits();
    }
    let pre_state_bytes = method_id.to_vec();
    let control_id_bytes: [u8; 32] = BN254_IDENTITY_CONTROL_ID.into();

    // Expected post state for an execution that halted successfully
    let post_state: SystemState = risc0_binfmt::SystemState {
        pc: 0,
        merkle_root: risc0_zkp::core::digest::Digest::default(),
    };
    let post_state_bytes: [u8; 32] = post_state.digest().into();

    let mut hasher = Sha256::new();
    hasher.update(&succinct_control_root_bytes);
    hasher.update(&pre_state_bytes);
    hasher.update(&post_state_bytes);
    hasher.update(&control_id_bytes);
    let result: [u8; 32] = hasher
        .finalize()
        .try_into()
        .expect("SHA256 should produce a 32-byte output");

    result
}

fn reverse_bits_and_copy(input: &[u8], output: &mut [u8]) {
    for i in 0..8 {
        let temp = u32::from_be_bytes(input[4 * i..4 * i + 4].try_into().unwrap()).reverse_bits();
        output[4 * i..4 * i + 4].copy_from_slice(&temp.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use docker::stark_to_succinct;
    use risc0_zkvm::compute_image_id;

    use super::*;
    // #[ignore = "This is to only test final proof generation"]
    #[test]
    fn test_final_circuit() {
        let final_circuit_elf = include_bytes!(
            "../../target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/final_guest/final-guest"
        );
        let header_chain_circuit_elf = include_bytes!(
            "../../target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/header_chain_guest/header-chain-guest"
        );
        println!(
            "Header chain circuit id: {:#?}",
            compute_image_id(header_chain_circuit_elf)
                .unwrap()
                .as_words()
        );
        let final_proof = include_bytes!("../../first_10.bin");
        let final_circuit_id = compute_image_id(final_circuit_elf).unwrap();

        let receipt: Receipt = Receipt::try_from_slice(final_proof).unwrap();

        let output = BlockHeaderCircuitOutput::try_from_slice(&receipt.journal.bytes).unwrap();

        let env = ExecutorEnv::builder()
            .write_slice(&borsh::to_vec(&output).unwrap())
            .add_assumption(receipt)
            .build()
            .unwrap();

        let prover = default_prover();

        let receipt = prover
            .prove_with_opts(env, final_circuit_elf, &ProverOpts::succinct())
            .unwrap()
            .receipt;

        let succinct_receipt = receipt.inner.succinct().unwrap().clone();
        let receipt_claim = succinct_receipt.clone().claim;
        println!("Receipt claim: {:#?}", receipt_claim);
        let journal: [u8; 32] = receipt.journal.bytes.clone().try_into().unwrap();
        let (proof, output_json_bytes) =
            stark_to_succinct(succinct_receipt, &receipt.journal.bytes);
        print!("Proof: {:#?}", proof);
        let constants_digest = calculate_succinct_output_prefix(final_circuit_id.as_bytes());
        println!("Constants digest: {:#?}", constants_digest);
        println!("Journal: {:#?}", receipt.journal);
        let mut constants_blake3_input = [0u8; 32];
        let mut journal_blake3_input = [0u8; 32];

        reverse_bits_and_copy(&constants_digest, &mut constants_blake3_input);
        reverse_bits_and_copy(&journal, &mut journal_blake3_input);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&constants_blake3_input);
        hasher.update(&journal_blake3_input);
        let final_output = hasher.finalize();
        let final_output_bytes: [u8; 32] = final_output.try_into().unwrap();
        let final_output_trimmed: [u8; 31] = final_output_bytes[..31].try_into().unwrap();
        assert_eq!(final_output_trimmed, output_json_bytes);
    }
}
