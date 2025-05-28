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
    use ark_groth16::{PreparedVerifyingKey, Proof, VerifyingKey};
    use ark_serialize::CanonicalSerialize;
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

    fn return_risc0_json_vk() -> VerifyingKeyJson {
        let vk: VerifyingKeyJson =
            serde_json::from_str(TEST_VK_JSON).expect("JSON was not well-formatted");
        vk
    }

    #[test]
    fn test_risc0_json_vk() {
        let risc0_vk = return_risc0_json_vk();
        let risc0_vk = risc0_vk.verifying_key().unwrap();
        println!("Risc0 Verifying Key: {:?}", risc0_vk);
        let ark_vk = get_ark_verifying_key();
        println!("Ark Verifying Key: {:?}", ark_vk);
        let prepared_vk = PreparedVerifyingKey::from(ark_vk);
        let mut writer = Vec::new();
        prepared_vk.serialize_uncompressed(&mut writer).unwrap();
        println!("Prepared Verifying Key: {:?}", writer);
    }

    fn get_ark_verifying_key() -> ark_groth16::VerifyingKey<Bn254> {
        let alpha_g1 = G1Affine::new(
            Fq::from_str(
                "16428432848801857252194528405604668803277877773566238944394625302971855135431",
            )
            .unwrap(),
            Fq::from_str(
                "16846502678714586896801519656441059708016666274385668027902869494772365009666",
            )
            .unwrap(),
        );

        let beta_g2 = G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "16348171800823588416173124589066524623406261996681292662100840445103873053252",
                )
                .unwrap(),
                Fq::from_str(
                    "3182164110458002340215786955198810119980427837186618912744689678939861918171",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "19687132236965066906216944365591810874384658708175106803089633851114028275753",
                )
                .unwrap(),
                Fq::from_str(
                    "4920802715848186258981584729175884379674325733638798907835771393452862684714",
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

        // Updated delta_g2 values
        let delta_g2 = G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "10227997703039283181582290960627338196156229704916784218538069646562187677677",
                )
                .unwrap(),
                Fq::from_str(
                    "20411751185568622699958613893854546380396578054362905610380980460447192812957",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "19410622549689385489266868415499937589520826181469262652493580648026799385326",
                )
                .unwrap(),
                Fq::from_str(
                    "18221822472624366517242451377355744163249204584158587724927194855761320078427",
                )
                .unwrap(),
            ),
        );

        // Updated gamma_abc_g1 values
        let gamma_abc_g1 = vec![
            G1Affine::new(
                Fq::from_str(
                    "19571076099348637099198083015310160790012533760996919719024648653145917055235",
                )
                .unwrap(),
                Fq::from_str(
                    "14879329443510433701746076883691028441387468955171876671347067459297419820024",
                )
                .unwrap(),
            ),
            G1Affine::new(
                Fq::from_str(
                    "10079665865378451362267730226014122216710720050586600511586560143601231078654",
                )
                .unwrap(),
                Fq::from_str(
                    "6555725917251677038656436643851406043022984638203884443430328235761293805514",
                )
                .unwrap(),
            ),
            G1Affine::new(
                Fq::from_str(
                    "3746333528780224148274759859636790034689520459098146898537815416836710828366",
                )
                .unwrap(),
                Fq::from_str(
                    "17030821594486428559778458728007385819329787664484856860645285960092799238265",
                )
                .unwrap(),
            ),
            G1Affine::new(
                Fq::from_str(
                    "21166318804742046403588005558374944701389292337216737233200323273349949954996",
                )
                .unwrap(),
                Fq::from_str(
                    "9189763389841105991448076412642486651871722517787332796715721565525698770119",
                )
                .unwrap(),
            ),
            G1Affine::new(
                Fq::from_str(
                    "20629837294837532470051827510201131048682785475119896473306547067537877253283",
                )
                .unwrap(),
                Fq::from_str(
                    "8928178873195373938160898270349633387476514526385703052044232982487910612791",
                )
                .unwrap(),
            ),
            G1Affine::new(
                Fq::from_str(
                    "14614047749979193665601186890775563952311141066002994546563083712022091326254",
                )
                .unwrap(),
                Fq::from_str(
                    "784784563887479235780656620002906644652870851389520267327853031196971317177",
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
        let final_proof = include_bytes!("../../data/proofs/mainnet/test_mainnet_first_10.bin");
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

        // println!("Final Circuit Receipt: {:?}", receipt);
        // println!(
        //     "Final Circuit Receipt Seal: {:?}",
        //     receipt.inner.succinct().unwrap().seal
        // );

        // let succinct_receipt = receipt.inner.succinct().unwrap().clone();
        // let receipt_claim = succinct_receipt.clone().claim;
        // println!("Receipt claim: {:#?}", receipt_claim);
        let journal: [u8; 32] = receipt.journal.bytes.clone().try_into().unwrap();
        // let (proof, output_json_bytes) =
        //     stark_to_succinct(succinct_receipt, &receipt.journal.bytes);
        stark_to_succinct(receipt.clone(), &receipt.journal.bytes);
    }
}

pub const TEST_VK_JSON: &str = r#"{
 "protocol": "groth16",
 "curve": "bn128",
 "nPublic": 5,
 "vk_alpha_1": [
  "16428432848801857252194528405604668803277877773566238944394625302971855135431",
  "16846502678714586896801519656441059708016666274385668027902869494772365009666",
  "1"
 ],
 "vk_beta_2": [
  [
   "16348171800823588416173124589066524623406261996681292662100840445103873053252",
   "3182164110458002340215786955198810119980427837186618912744689678939861918171"
  ],
  [
   "19687132236965066906216944365591810874384658708175106803089633851114028275753",
   "4920802715848186258981584729175884379674325733638798907835771393452862684714"
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
   "10227997703039283181582290960627338196156229704916784218538069646562187677677",
   "20411751185568622699958613893854546380396578054362905610380980460447192812957"
  ],
  [
   "19410622549689385489266868415499937589520826181469262652493580648026799385326",
   "18221822472624366517242451377355744163249204584158587724927194855761320078427"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_alphabeta_12": [
  [
   [
    "5275725312362878540782176211860327475781113689246818544623830805017503247034",
    "700769043921060225711174322502145319612473365595920873303028146383045646735"
   ],
   [
    "16577533945604560505206253312979863148043263406037367789711279754781525822966",
    "9408338099405950952721388539539775335199747835458172188116297223654842340186"
   ],
   [
    "12663399896275491035004982800573482669934131767886952660443268164480899034271",
    "4432711152773877173921024337047412943791122852326272337530740732443732395954"
   ]
  ],
  [
   [
    "13121778684901402722679281862736806628725205381360313795132945954337708567513",
    "9534744673358550231812045647241180985734073058548683258847806241019905135720"
   ],
   [
    "21329152369227346659770815132468371951064045353268189088026893413117512652875",
    "17209195434408943681049655974234541356066884378594227002358272904159790622854"
   ],
   [
    "5346467096835895366917814311591075634165750361894629082277248282132405045579",
    "15508364027636868967189209273443690126627947943852338696115233789046842639684"
   ]
  ]
 ],
 "IC": [
  [
   "19571076099348637099198083015310160790012533760996919719024648653145917055235",
   "14879329443510433701746076883691028441387468955171876671347067459297419820024",
   "1"
  ],
  [
   "10079665865378451362267730226014122216710720050586600511586560143601231078654",
   "6555725917251677038656436643851406043022984638203884443430328235761293805514",
   "1"
  ],
  [
   "3746333528780224148274759859636790034689520459098146898537815416836710828366",
   "17030821594486428559778458728007385819329787664484856860645285960092799238265",
   "1"
  ],
  [
   "21166318804742046403588005558374944701389292337216737233200323273349949954996",
   "9189763389841105991448076412642486651871722517787332796715721565525698770119",
   "1"
  ],
  [
   "20629837294837532470051827510201131048682785475119896473306547067537877253283",
   "8928178873195373938160898270349633387476514526385703052044232982487910612791",
   "1"
  ],
  [
   "14614047749979193665601186890775563952311141066002994546563083712022091326254",
   "784784563887479235780656620002906644652870851389520267327853031196971317177",
   "1"
  ]
 ]
}"#;
