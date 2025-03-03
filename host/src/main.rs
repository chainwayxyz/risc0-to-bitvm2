use borsh::BorshDeserialize;
use risc0_groth16::VerifyingKeyJson;
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
            include_bytes!("../../elfs/mainnet-header-chain-guest")
        }
        Some(network) if matches!(network.as_bytes(), b"testnet4") => {
            include_bytes!("../../elfs/testnet4-header-chain-guest")
        }
        Some(network) if matches!(network.as_bytes(), b"signet") => {
            include_bytes!("../../elfs/signet-header-chain-guest")
        }
        Some(network) if matches!(network.as_bytes(), b"regtest") => {
            include_bytes!("../../elfs/regtest-header-chain-guest")
        }
        None => include_bytes!("../../elfs/mainnet-header-chain-guest"),
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

fn reverse_bits_and_copy(input: &[u8], output: &mut [u8]) {
    for i in 0..8 {
        let temp = u32::from_be_bytes(input[4 * i..4 * i + 4].try_into().unwrap()).reverse_bits();
        output[4 * i..4 * i + 4].copy_from_slice(&temp.to_le_bytes());
    }
}

fn get_verifying_key_json() -> VerifyingKeyJson {
    let json_data = r#"
    {
        "protocol": "groth16",
        "curve": "bn128",
        "nPublic": 1,
        "vk_alpha_1": [
        "20491192805390485299153009773594534940189261866228447918068658471970481763042",
        "9383485363053290200918347156157836566562967994039712273449902621266178545958",
        "1"
        ],
        "vk_beta_2": [
        [
        "6375614351688725206403948262868962793625744043794305715222011528459656738731",
        "4252822878758300859123897981450591353533073413197771768651442665752259397132"
        ],
        [
        "10505242626370262277552901082094356697409835680220590971873171140371331206856",
        "21847035105528745403288232691147584728191162732299865338377159692350059136679"
        ],
        [
        "1",
        "0"
        ]
        ],
        "vk_gamma_2": [
        [
        "10857046999023057135944570762232829481370756359578518086990519993285655852781",
        "11559732032986387107991004021392285783925812861821192530917403151452391805634"
        ],
        [
        "8495653923123431417604973247489272438418190587263600148770280649306958101930",
        "4082367875863433681332203403145435568316851327593401208105741076214120093531"
        ],
        [
        "1",
        "0"
        ]
        ],
        "vk_delta_2": [
        [
        "17373390530484628175439079012547601221793532405373183847591328903803405586286",
        "4625210858552158309405374705253571552256748541870661454419080699362567957226"
        ],
        [
        "20292316235570350162741350858895467611317790503850491347042646354236531519055",
        "17004339328415633000851435380698565994375131307744525391751714344270706811231"
        ],
        [
        "1",
        "0"
        ]
        ],
        "vk_alphabeta_12": [
        [
        [
            "2029413683389138792403550203267699914886160938906632433982220835551125967885",
            "21072700047562757817161031222997517981543347628379360635925549008442030252106"
        ],
        [
            "5940354580057074848093997050200682056184807770593307860589430076672439820312",
            "12156638873931618554171829126792193045421052652279363021382169897324752428276"
        ],
        [
            "7898200236362823042373859371574133993780991612861777490112507062703164551277",
            "7074218545237549455313236346927434013100842096812539264420499035217050630853"
        ]
        ],
        [
        [
            "7077479683546002997211712695946002074877511277312570035766170199895071832130",
            "10093483419865920389913245021038182291233451549023025229112148274109565435465"
        ],
        [
            "4595479056700221319381530156280926371456704509942304414423590385166031118820",
            "19831328484489333784475432780421641293929726139240675179672856274388269393268"
        ],
        [
            "11934129596455521040620786944827826205713621633706285934057045369193958244500",
            "8037395052364110730298837004334506829870972346962140206007064471173334027475"
        ]
        ]
        ],
        "IC": [
        [
        "19647329884141636868838662743921462850093495460601527910594807780507527498755",
        "11866587864098764425295475199808859787294133529274334392579829950494218737898",
        "1"
        ],
        [
        "2244061991313498397063727186076860978321484653259630566498796511714519280220",
        "3313153727619754321539238199327739757956770721532533603738719136366368438484",
        "1"
        ]
        ]
    }
    "#;
    let vk: VerifyingKeyJson =
        serde_json::from_str(json_data).expect("JSON was not well-formatted");
    vk
}

#[cfg(test)]
mod tests {

    use risc0_groth16::VerifyingKeyJson;
    use risc0_to_bitvm2_core::{
        final_circuit::FinalCircuitInput, header_chain::BlockHeaderCircuitOutput,
        merkle_tree::BitcoinMerkleTree, mmr_native::MMRNative, spv::SPV,
        transaction::CircuitTransaction,
    };

    use docker::stark_to_succinct;
    use hex_literal::hex;
    use risc0_zkvm::{compute_image_id, SuccinctReceipt};

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

    use crate::docker::test_stark_to_succinct;

    use super::*;
    // #[ignore = "This is to only test final proof generation"]
    /// Run this test only when build for the mainnet
    #[test]
    fn test_final_circuit() {
        let final_circuit_elf = include_bytes!("../../elfs/mainnet-final-spv-guest");
        let header_chain_circuit_elf = include_bytes!("../../elfs/mainnet-header-chain-guest");
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
        println!("Proof: {:#?}", proof);
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

    #[derive(Debug, Clone)]
    pub struct BitVMSetup {
        // control_root: [u8; 32],
        // pre_state: [u8; 32],
        // post_state: [u8; 32],
        // id_bn254_fr: [u8; 32],
        move_to_vault_txid: [u8; 32],
        watcthower_challenge_wpks_hash: [u8; 32],
        operator_id: [u8; 32],
        payout_tx_blockhash: [u8; 20],
        latest_blockhash: [u8; 20],
        challenge_sending_watchtowers: [u8; 20],
    }

    pub const TEST_BITVM_SETUP: BitVMSetup = BitVMSetup {
        // control_root: [0u8; 32],
        // pre_state: [0u8; 32],
        // post_state: [0u8; 32],
        // id_bn254_fr: [0u8; 32],
        move_to_vault_txid: [
            187, 37, 16, 52, 104, 164, 103, 56, 46, 217, 245, 133, 18, 154, 212, 3, 49, 181, 68,
            37, 21, 93, 111, 15, 174, 140, 121, 147, 145, 238, 46, 127,
        ],
        watcthower_challenge_wpks_hash: [
            116, 216, 207, 17, 240, 166, 16, 227, 208, 229, 191, 107, 233, 150, 159, 42, 222, 101,
            77, 96, 233, 15, 56, 107, 30, 138, 206, 135, 242, 68, 78, 22,
        ],
        operator_id: [
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        payout_tx_blockhash: [
            203, 228, 88, 12, 216, 97, 185, 239, 128, 152, 124, 141, 167, 201, 168, 8, 0, 0, 0, 0,
        ],
        latest_blockhash: [
            3, 89, 234, 156, 226, 43, 141, 221, 113, 52, 235, 82, 90, 148, 0, 0, 0, 0, 0, 0,
        ],
        challenge_sending_watchtowers: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    pub const BRIDGE_CIRCUIT_ELF: &[u8] =
        include_bytes!("../../mock_bridge_proof/testnet4-bridge-circuit-guest");

    #[test]
    fn test_bridge_circuit() {
        let receipt_bytes: &[u8] = include_bytes!("../../mock_bridge_proof/bridge_proof.bin");
        let receipt: Receipt = Receipt::try_from_slice(receipt_bytes).unwrap();

        let succinct_receipt = receipt.inner.succinct().unwrap().clone();
        let receipt_claim = succinct_receipt.clone().claim;
        println!("Receipt claim: {:#?}", receipt_claim);
        let journal: [u8; 32] = receipt.journal.bytes.clone().try_into().unwrap();
        let (proof, public_inputs_json, output_json_bytes) =
            test_stark_to_succinct(succinct_receipt, &receipt.journal.bytes);
        print!("Proof: {:#?}", proof);
        let bridge_circuit_id = compute_image_id(BRIDGE_CIRCUIT_ELF).unwrap();
        let combined_method_id_constant =
            calculate_succinct_output_prefix(bridge_circuit_id.as_bytes()); // Check if the operations match the expected values part1
        println!("Constants digest: {:#?}", combined_method_id_constant);
        println!("Journal: {:#?}", receipt.journal);
        let mut constants_blake3_input = [0u8; 32];
        let mut journal_blake3_input = [0u8; 32];
        let vk_json = get_verifying_key_json();
        println!("Proof json: {:#?}", proof);
        println!("Public inputs json: {:#?}", public_inputs_json);
        println!("Verifying key json: {:#?}", vk_json);
        let g16_verifier =
            risc0_groth16::Verifier::from_json(proof, public_inputs_json, vk_json).unwrap();
        g16_verifier.verify().unwrap();

        reverse_bits_and_copy(&combined_method_id_constant, &mut constants_blake3_input);
        reverse_bits_and_copy(&journal, &mut journal_blake3_input);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&constants_blake3_input);
        hasher.update(&journal_blake3_input);
        let final_output = hasher.finalize();
        let final_output_bytes: [u8; 32] = final_output.try_into().unwrap();
        let final_output_trimmed: [u8; 31] = final_output_bytes[..31].try_into().unwrap();
        assert_eq!(final_output_trimmed, output_json_bytes); // Check if the operations match the expected values part2

        // Check if the operations match the expected values part3
        let mut hasher = Sha256::new();
        hasher.update(&TEST_BITVM_SETUP.move_to_vault_txid);
        hasher.update(&TEST_BITVM_SETUP.watcthower_challenge_wpks_hash);
        hasher.update(&TEST_BITVM_SETUP.operator_id);
        let deposit_constant: [u8; 32] = hasher
            .finalize()
            .try_into()
            .expect("SHA256 should produce a 32-byte output");

        let mut hasher = blake3::Hasher::new();
        hasher.update(&TEST_BITVM_SETUP.payout_tx_blockhash);
        hasher.update(&TEST_BITVM_SETUP.latest_blockhash);
        hasher.update(&TEST_BITVM_SETUP.challenge_sending_watchtowers);
        let x = hasher.finalize();
        let x_bytes: [u8; 32] = x.try_into().unwrap();

        let mut hasher = blake3::Hasher::new();
        hasher.update(&deposit_constant);
        hasher.update(&x_bytes);
        let y = hasher.finalize();
        let y_bytes: [u8; 32] = y.try_into().unwrap();

        let mut hasher = blake3::Hasher::new();
        hasher.update(&combined_method_id_constant);
        hasher.update(&y_bytes);
        let public_output = hasher.finalize();
        let public_output_bytes: [u8; 32] = public_output.try_into().unwrap();
        let public_output_trimmed: [u8; 31] = public_output_bytes[..31].try_into().unwrap();
        assert_eq!(public_output_trimmed, output_json_bytes);
    }
}
