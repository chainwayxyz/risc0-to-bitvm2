use borsh::BorshDeserialize;
use circuits::{
    header_chain::{
        BlockHeader, BlockHeaderCircuitOutput, HeaderChainCircuitInput, HeaderChainPrevProofType,
    },
    risc0_zkvm::{default_prover, ExecutorEnv},
};
use header_chain_circuit::{HEADER_CHAIN_GUEST_ELF, HEADER_CHAIN_GUEST_ID};
use risc0_zkvm::{ProverOpts, Receipt};
use std::{env, fs};

pub mod docker;

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

    // Download the headers.bin file from https://zerosync.org/chaindata/headers.bin
    let headers = include_bytes!("../../headers.bin");
    let headers = headers
        .chunks(80)
        .map(|header| BlockHeader::try_from_slice(header).unwrap())
        .collect::<Vec<BlockHeader>>();

    // Set the previous proof type based on input_proof argument
    let prev_receipt = if input_proof.to_lowercase() == "none" {
        None
    } else {
        let proof_bytes = fs::read(input_proof).expect("Failed to read input proof file");
        let receipt: Receipt = Receipt::try_from_slice(&proof_bytes).unwrap();
        Some(receipt)
        // let prev_output = BlockHeaderCircuitOutput::try_from_slice(&receipt.journal.bytes).unwrap();
        // HeaderChainPrevProofType::PrevProof(prev_output)
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
    println!("output method id: {:#?}", output.method_id);

    // println!("Total work: {:#?}", output);
    println!("Method id: {:#?}", HEADER_CHAIN_GUEST_ID);
    println!("Journal: {:#?}", receipt.journal);


    // Save the receipt to the specified output file path
    let receipt_bytes = borsh::to_vec(&receipt).unwrap();
    fs::write(output_file_path, &receipt_bytes).expect("Failed to write receipt to output file");
    println!("Receipt saved to {}", output_file_path);
}


#[cfg(test)]
mod tests {
    use risc0_zkvm::compute_image_id;

    use super::*;
    #[ignore = "This is to only test final proof generation"]
    #[test]
    fn test_final_circuit() {
        let final_circuit_elf = include_bytes!("../../target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/final_guest/final-guest");
        let header_chain_circuit_elf = include_bytes!("../../target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/header_chain_guest/header-chain-guest");
        let final_proof = include_bytes!("../../first_10.bin");

        println!("final circuit id: {}",compute_image_id(final_circuit_elf).unwrap());
        println!("header chain circuit id: {}",compute_image_id(header_chain_circuit_elf).unwrap());

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

        println!("Journal: {:#?}", receipt.journal);
    }
}