use crypto_bigint::U256;
use hex::ToHex;
use num_bigint::BigUint;
use num_traits::Num;
use risc0_groth16::{to_json, ProofJson, Seal};
use risc0_zkvm::{sha::Digest, MaybePruned, Receipt, ReceiptClaim, SuccinctReceipt, SuccinctReceiptVerifierParameters, SystemState};
use serde_json::Value;
use risc0_zkvm::sha::Digestible;
use std::{
    env::consts::ARCH,
    fs,
    path::Path,
    process::{Command, Stdio},
};

use tempfile::tempdir;

pub fn stark_to_succinct(
    succinct_receipt: &SuccinctReceipt<ReceiptClaim>,
    receipt_claim: &ReceiptClaim,
    journal: &[u8],
    verify_stark_method_id: [u32; 8],
) -> Seal {
    let ident_receipt = risc0_zkvm::recursion::identity_p254(succinct_receipt).unwrap();
    let identity_p254_seal_bytes = ident_receipt.get_seal_bytes();

    // let pre_state_bits: risc0_zkvm::MaybePruned<SystemState> = receipt_claim.clone().pre;
    // println!("pre_state_bits: {:?}", pre_state_bits);
    // let pre_state_digest_bits = pre_state_bits.clone().digest();
    // println!("pre_state_digest_bits: {:?}", pre_state_digest_bits);
    // let post_state_bits: risc0_zkvm::MaybePruned<SystemState> = receipt_claim.clone().post;
    // println!("post_state_bits: {:?}", post_state_bits);
    // let post_state_digest_bits = post_state_bits.clone().digest();
    // println!("post_state_digest_bits: {:?}", post_state_digest_bits);

    // This part is from risc0-groth16
    // if !is_x86_architecture() {
    //     panic!("stark_to_snark is only supported on x86 architecture.")
    // }
    if !is_docker_installed() {
        panic!("Please install docker first.")
    }

    let tmp_dir = tempdir().unwrap();
    let work_dir = std::env::var("RISC0_WORK_DIR");
    let work_dir = work_dir.as_ref().map(Path::new).unwrap_or(tmp_dir.path());

    std::fs::write(work_dir.join("seal.r0"), identity_p254_seal_bytes.clone()).unwrap();
    let seal_path = work_dir.join("input.json");
    let proof_path = work_dir.join("proof.json");
    let mut seal_json = Vec::new();
    to_json(&*identity_p254_seal_bytes, &mut seal_json).unwrap();
    std::fs::write(seal_path.clone(), seal_json).unwrap();

    // Add additional fields to our input.json
    // let pre_state_bits: Vec<String> = verify_stark_method_id
    //     .iter()
    //     .flat_map(|item| {
    //         // Iterate over the bits from most significant (31) to least significant (0)
    //         (0..32).rev().map(move |n| ((item >> n) & 1).to_string())
    //     })
    //     .take(8 * 4) // Take the first 32 bits (8 items * 4 bytes)
    //     .collect();

    let pre_state: risc0_zkvm::MaybePruned<SystemState> = receipt_claim.clone().pre;
    println!("pre_state: {:?}", pre_state);
    let pre_state_digest: Digest = pre_state.clone().digest();
    let pre_state_digest_bits: Vec<String> = pre_state_digest.as_words().iter()
    .flat_map(|item| {
        // Iterate over the bits from most significant (31) to least significant (0)
        (0..32).rev().map(move |n| ((item >> n) & 1).to_string())
    })
    .take(8 * 4) // Take the first 32 bits (8 items * 4 bytes)
    .collect();;
    println!("pre_state_digest_bits: {:?}", pre_state_digest_bits);
    let post_state: risc0_zkvm::MaybePruned<SystemState> = receipt_claim.clone().post;
    println!("post_state: {:?}", post_state);
    let post_state_digest: Digest = post_state.clone().digest();
    let post_state_digest_bits: Vec<String> = post_state_digest.as_words().iter()
    .flat_map(|item| {
        // Iterate over the bits from most significant (31) to least significant (0)
        (0..32).rev().map(move |n| ((item >> n) & 1).to_string())
    })
    .take(8 * 4) // Take the first 32 bits (8 items * 4 bytes)
    .collect();;
    println!("post_state_digest_bits: {:?}", post_state_digest_bits);

    let journal_bits: Vec<String> = journal
        .iter()
        .flat_map(|&byte| (0..8).map(move |i| ((byte >> (7 - i)) & 1).to_string()))
        .collect();

    let control_root: [u8; 32] = SuccinctReceiptVerifierParameters::default()
        .control_root
        .as_bytes()
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>()
        .try_into()
        .expect("Slice conversion failed; expected 32 bytes");

    let a1_str = format!("0x{}", hex::encode(&control_root[0..16]));
    let a0_str = format!("0x{}", hex::encode(&control_root[16..32]));

    let id_bn254_fr_bits: Vec<String> = ident_receipt
        .control_id
        .as_bytes()
        .iter()
        .flat_map(|&byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect();

    let mut seal_json: Value = {
        let file_content = fs::read_to_string(&seal_path).unwrap();
        serde_json::from_str(&file_content).unwrap()
    };

    seal_json["journal_blake3_digest_bits"] = journal_bits.into();
    seal_json["pre_state_digest_bits"] = pre_state_digest_bits.into();
    seal_json["post_state_digest_bits"] = post_state_digest_bits.into();
    seal_json["id_bn254_fr_bits"] = id_bn254_fr_bits.into();
    seal_json["control_root"] = vec![a0_str, a1_str].into();
    std::fs::write(seal_path, serde_json::to_string_pretty(&seal_json).unwrap()).unwrap();

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/mnt", work_dir.to_string_lossy()))
        .arg("risc0-groth16-prover")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!(
            "docker returned failure exit code: {:?}",
            output.status.code()
        );
    }

    let contents = std::fs::read_to_string(proof_path).unwrap();
    let proof_json: ProofJson = serde_json::from_str(&contents).unwrap();
    proof_json.try_into().unwrap()
}

fn is_docker_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_x86_architecture() -> bool {
    ARCH == "x86_64" || ARCH == "x86"
}
