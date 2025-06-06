use borsh::BorshDeserialize;
use risc0_zkvm::{default_prover, ExecutorEnv};

use risc0_circuit_recursion::control_id::BN254_IDENTITY_CONTROL_ID;
use risc0_to_bitvm2_core::header_chain::{
    BlockHeaderCircuitOutput, CircuitBlockHeader, HeaderChainCircuitInput, HeaderChainPrevProofType,
};
use risc0_zkvm::{compute_image_id, sha::Digestible};
use risc0_zkvm::{ProverOpts, Receipt, SuccinctReceiptVerifierParameters, SystemState};
use sha2::Digest;
use sha2::Sha256;
use std::{env, fs};

pub mod docker;

const HEADER_CHAIN_GUEST_ELF: &[u8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => {
            include_bytes!("../../elfs/mainnet-header-chain-guest.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"testnet4") => {
            include_bytes!("../../elfs/testnet4-header-chain-guest.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"signet") => {
            include_bytes!("../../elfs/signet-header-chain-guest.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"regtest") => {
            include_bytes!("../../elfs/regtest-header-chain-guest.bin")
        }
        None => include_bytes!("../../elfs/mainnet-header-chain-guest.bin"),
        _ => panic!("Invalid path or ELF file"),
    }
};

const HEADERS: &[u8] = {
    match option_env!("BITCOIN_NETWORK") {
        Some(network) if matches!(network.as_bytes(), b"mainnet") => {
            include_bytes!("../../data/headers/mainnet-headers.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"testnet4") => {
            include_bytes!("../../data/headers/testnet4-headers.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"signet") => {
            include_bytes!("../../data/headers/signet-headers.bin")
        }
        Some(network) if matches!(network.as_bytes(), b"regtest") => {
            include_bytes!("../../data/headers/regtest-headers.bin")
        }
        None => include_bytes!("../../data/headers/mainnet-headers.bin"),
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

    let header_chain_guest_id: [u32; 8] = compute_image_id(HEADER_CHAIN_GUEST_ELF)
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
        println!("Previous Receipt Journal: {:?}", receipt.journal);

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
        method_id: header_chain_guest_id,
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
        .unwrap();

    println!("New Receipt: {:?}", receipt.stats);
    let receipt = receipt.receipt;
    println!("New Receipt Journal: {:?}", receipt.journal);

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

#[cfg(test)]
mod tests {

    use ark_bn254::{Bn254, Fq, Fq2, G1Affine, G2Affine};
    use ark_ff::{Field, PrimeField};
    use ark_groth16::{Proof, VerifyingKey};
    use risc0_to_bitvm2_core::{
        final_circuit::FinalCircuitInput, header_chain::BlockHeaderCircuitOutput,
        merkle_tree::BitcoinMerkleTree, mmr_native::MMRNative, spv::SPV,
        transaction::CircuitTransaction,
    };
    use std::str::FromStr;

    use risc0_groth16::{Fr, Seal, VerifyingKeyJson};

    use docker::stark_to_succinct;
    use hex_literal::hex;
    use risc0_zkp::verify;
    use risc0_zkvm::compute_image_id;

    const MAINNET_BLOCK_HASHES: [[u8; 32]; 11] = [
        hex!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"),
        hex!("4860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000"),
        hex!("bddd99ccfda39da1b108ce1a5d70038d0a967bacb68b6b63065f626a00000000"),
        hex!("4944469562ae1c2c74d9a535e00b6f3e40ffbad4f2fda3895501b58200000000"),
        hex!("85144a84488ea88d221c8bd6c059da090e88f8a2c99690ee55dbba4e00000000"),
        hex!("fc33f596f822a0a1951ffdbf2a897b095636ad871707bf5d3162729b00000000"),
        hex!("8d778fdc15a2d3fb76b7122a3b5582bea4f21f5a0c693537e7a0313000000000"),
        hex!("4494c8cf4154bdcc0720cd4a59d9c9b285e4b146d45f061d2b6c967100000000"),
        hex!("c60ddef1b7618ca2348a46e868afc26e3efc68226c78aa47f8488c4000000000"),
        hex!("0508085c47cc849eb80ea905cc7800a3be674ffc57263cf210c59d8d00000000"),
        hex!("e915d9a478e3adf3186c07c61a22228b10fd87df343c92782ecc052c00000000"),
    ];

    fn get_ark_verifying_key() -> ark_groth16::VerifyingKey<Bn254> {
        let alpha_g1 = G1Affine::new(
            Fq::from_str(
                "20491192805390485299153009773594534940189261866228447918068658471970481763042",
            )
            .unwrap(),
            Fq::from_str(
                "9383485363053290200918347156157836566562967994039712273449902621266178545958",
            )
            .unwrap(),
        );

        let beta_g2 = G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "6375614351688725206403948262868962793625744043794305715222011528459656738731",
                )
                .unwrap(),
                Fq::from_str(
                    "4252822878758300859123897981450591353533073413197771768651442665752259397132",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "10505242626370262277552901082094356697409835680220590971873171140371331206856",
                )
                .unwrap(),
                Fq::from_str(
                    "21847035105528745403288232691147584728191162732299865338377159692350059136679",
                )
                .unwrap(),
            ),
        );

        let gamma_g2 = G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "10857046999023057135944570762232829481370756359578518086990519993285655852781",
                )
                .unwrap(),
                Fq::from_str(
                    "11559732032986387107991004021392285783925812861821192530917403151452391805634",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "8495653923123431417604973247489272438418190587263600148770280649306958101930",
                )
                .unwrap(),
                Fq::from_str(
                    "4082367875863433681332203403145435568316851327593401208105741076214120093531",
                )
                .unwrap(),
            ),
        );

        let delta_g2 = G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "19928663713463533589216209779412278386769407450988172849262535478593422929698",
                )
                .unwrap(),
                Fq::from_str(
                    "19916519943909223643323234301580053157586699704876134064841182937085943926141",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "4584600978911428195337731119171761277167808711062125916470525050324985708782",
                )
                .unwrap(),
                Fq::from_str(
                    "903010326261527050999816348900764705196723158942686053018929539519969664840",
                )
                .unwrap(),
            ),
        );

        let gamma_abc_g1 = vec![
            G1Affine::new(
                Fq::from_str(
                    "6698887085900109660417671413804888867145870700073340970189635830129386206569",
                )
                .unwrap(),
                Fq::from_str(
                    "10431087902009508261375793061696708147989126018612269070732549055898651692604",
                )
                .unwrap(),
            ),
            G1Affine::new(
                Fq::from_str(
                    "20225609417084538563062516991929114218412992453664808591983416996515711931386",
                )
                .unwrap(),
                Fq::from_str(
                    "3236310410959095762960658876334609343091075204896196791007975095263664214628",
                )
                .unwrap(),
            ),
        ];

        VerifyingKey::<Bn254> {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1,
        }
    }

    // fn from_seal(seal: &[u8; 256]) -> Proof<Bn254> {
    fn from_seal(seal: Seal) -> Proof<Bn254> {
        let seal_bytes: [u8; 256] = seal.to_vec().try_into().unwrap();

        let a = G1Affine::new(
            ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[0..32]),
            ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[32..64]),
        );

        let b = G2Affine::new(
            ark_bn254::Fq2::from_base_prime_field_elems([
                ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[96..128]),
                ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[64..96]),
            ])
            .unwrap(),
            ark_bn254::Fq2::from_base_prime_field_elems([
                ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[160..192]),
                ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[128..160]),
            ])
            .unwrap(),
        );

        let c = G1Affine::new(
            ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[192..224]),
            ark_bn254::Fq::from_be_bytes_mod_order(&seal_bytes[224..256]),
        );

        Proof {
            a: a.into(),
            b: b.into(),
            c: c.into(),
        }
    }

    use super::*;
    // #[ignore = "This is to only test final proof generation"]
    /// Run this test only when build for the mainnet
    #[test]
    fn test_final_circuit() {
        let final_circuit_elf = include_bytes!("../../elfs/mainnet-final-spv-guest.bin");
        let header_chain_circuit_elf = include_bytes!("../../elfs/mainnet-header-chain-guest.bin");
        println!(
            "Header chain circuit id: {:#?}",
            compute_image_id(header_chain_circuit_elf)
                .unwrap()
                .as_words()
        );
        let final_proof = include_bytes!("../../data/proofs/mainnet/mainnet_first_10.bin");
        let final_circuit_id = compute_image_id(final_circuit_elf).unwrap();

        let receipt: Receipt = Receipt::try_from_slice(final_proof).unwrap();

        let mut mmr_native = MMRNative::new();
        for block_hash in MAINNET_BLOCK_HASHES.iter() {
            mmr_native.append(*block_hash);
        }

        let output = BlockHeaderCircuitOutput::try_from_slice(&receipt.journal.bytes).unwrap();
        let tx: CircuitTransaction = CircuitTransaction(bitcoin::consensus::deserialize(&hex::decode("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000").unwrap()).unwrap());
        let block_header: risc0_to_bitvm2_core::header_chain::CircuitBlockHeader = CircuitBlockHeader::try_from_slice(hex::decode("0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c").unwrap().as_slice()).unwrap();
        let bitcoin_merkle_tree: BitcoinMerkleTree = BitcoinMerkleTree::new(vec![tx.txid()]);
        let bitcoin_inclusion_proof = bitcoin_merkle_tree.generate_proof(0);
        let (_, mmr_inclusion_proof) = mmr_native.generate_proof(0);
        let spv: SPV = SPV::new(
            tx,
            bitcoin_inclusion_proof,
            block_header,
            mmr_inclusion_proof,
        );
        let final_circuit_input: FinalCircuitInput = FinalCircuitInput {
            block_header_circuit_output: output,
            spv: spv,
        };
        let env = ExecutorEnv::builder()
            .write_slice(&borsh::to_vec(&final_circuit_input).unwrap())
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
        println!("Proof: {:?}", proof);
        let constants_digest = calculate_succinct_output_prefix(final_circuit_id.as_bytes());
        println!("Constants digest: {:?}", constants_digest);
        println!("Journal: {:?}", receipt.journal);

        let mut hasher = blake3::Hasher::new();
        hasher.update(&constants_digest);
        hasher.update(&journal);
        let final_output = hasher.finalize();
        let final_output_bytes: [u8; 32] = final_output.try_into().unwrap();
        let final_output_trimmed: [u8; 31] = final_output_bytes[..31].try_into().unwrap();
        assert_eq!(final_output_trimmed, output_json_bytes);

        let ark_proof = from_seal(proof);
        let public_input_scalar = ark_bn254::Fr::from_be_bytes_mod_order(&final_output_trimmed);
        println!("Public input scalar: {:?}", public_input_scalar);
        let ark_vk = get_ark_verifying_key();
        let ark_pvk = ark_groth16::prepare_verifying_key(&ark_vk);

        let res = ark_groth16::Groth16::<ark_bn254::Bn254>::verify_proof(
            &ark_pvk,
            &ark_proof,
            &[public_input_scalar],
        )
        .unwrap();

        println!("Verification result: {:?}", res);
        assert!(res, "Verification failed");
    }
}
