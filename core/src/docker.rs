use crypto_bigint::U256;
use hex::ToHex;
use num_bigint::BigUint;
use risc0_groth16::to_json;
use risc0_zkvm::{ReceiptClaim, SuccinctReceipt, SuccinctReceiptVerifierParameters};
use serde_json::Value;
use num_traits::Num;

use std::{fs, path::Path, process::{Command, Stdio}};

use tempfile::tempdir;


pub fn stark_to_succinct(
    ident_receipt: SuccinctReceipt<ReceiptClaim>,
    journal: &[u8],
    verify_stark_method_id: &[u32; 8],
) {
    let mut pre_state_bits: Vec<u8> = Vec::new();
    for item in verify_stark_method_id.iter().take(8) {
        for j in 0..4 {
            for k in 0..8 {
                pre_state_bits.push((item >> (8 * j + 7 - k)) as u8 & 1);
            }
        }
    }
    let tmp_dir = tempdir().unwrap();
    let work_dir = std::env::var("RISC0_WORK_DIR");
    let work_dir = work_dir.as_ref().map(Path::new).unwrap_or(tmp_dir.path());
    let identity_p254_seal_bytes = ident_receipt.get_seal_bytes();
    std::fs::write(
        work_dir.join("seal.r0"),
        identity_p254_seal_bytes.as_slice(),
    )
    .unwrap();
    let seal_path = work_dir.join("input.json");
    let _proof_path = work_dir.join("proof.json");
    let mut seal_json = Vec::new();
    to_json(identity_p254_seal_bytes.as_slice(), &mut seal_json).unwrap();
    std::fs::write(seal_path.clone(), seal_json).unwrap();
    let journal_hex = hex::encode(journal);
    println!("Journal hex: {:?}", journal_hex);

    let mut journal_bits = Vec::new();
    for byte in journal {
        for i in 0..8 {
            journal_bits.push((byte >> (7 - i)) & 1);
        }
    }

    let succinct_verifier_params = SuccinctReceiptVerifierParameters::default();
    println!("Succinct verifier params: {:?}", succinct_verifier_params);
    let succinct_control_root = succinct_verifier_params.control_root;
    println!("Succinct control root: {:?}", succinct_control_root);
    let mut succinct_control_root_bytes: [u8; 32] =
        succinct_control_root.as_bytes().try_into().unwrap();
    succinct_control_root_bytes.reverse();
    let succinct_control_root_bytes: String = succinct_control_root_bytes.encode_hex();
    let a1_str = succinct_control_root_bytes[0..32].to_string();
    let a0_str = succinct_control_root_bytes[32..64].to_string();
    println!("Succinct control root a0: {:?}", a0_str);
    println!("Succinct control root a1: {:?}", a1_str);
    let a0_dec = to_decimal(&a0_str).unwrap();
    let a1_dec = to_decimal(&a1_str).unwrap();
    println!("Succinct control root a0 dec: {:?}", a0_dec);
    println!("Succinct control root a1 dec: {:?}", a1_dec);

    let id_bn254_fr_bits = ident_receipt
        .control_id
        .as_bytes()
        .iter()
        .flat_map(|&byte| (0..8).rev().map(move |i| (byte >> i) & 1));

    let mut seal_json: Value = {
        let file_content = fs::read_to_string(&seal_path).unwrap();
        serde_json::from_str(&file_content).unwrap()
    };

    let journal_bits_str_vec: Vec<String> = journal_bits
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let pre_state_bits_str_vec = pre_state_bits
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let id_bn254_fr_bits_str_vec = id_bn254_fr_bits
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    seal_json["journal_blake3_digest_bits"] = journal_bits_str_vec.into();
    seal_json["pre_state_digest_bits"] = pre_state_bits_str_vec.into();
    seal_json["id_bn254_fr_bits"] = id_bn254_fr_bits_str_vec.into();
    seal_json["control_root"] = vec![a0_dec, a1_dec].into();
    std::fs::write(seal_path, serde_json::to_string_pretty(&seal_json).unwrap()).unwrap();

    println!("Starting proving");

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
    println!("Output: {:?}", output);

    if !output.status.success() {
        eprintln!(
            "docker returned failure exit code: {:?}",
            output.status.code()
        );
    }
}

pub fn bits_to_num(len: usize, bits: &[u8]) -> U256 {
    assert!(len <= 252);
    assert!(bits.len() == len);

    let mut num_lo: u128 = 0;
    let mut num_hi: u128 = 0;

    for (i, &bit) in bits.iter().enumerate() {
        if i < 128 {
            num_lo |= (bit as u128) << i;
        } else {
            num_hi |= (bit as u128) << (i - 128);
        }
    }

    let u256_lo = U256::from(num_lo);
    if len > 128 {
        let u256_hi = U256::from(num_hi) << 128;
        u256_hi.wrapping_add(&u256_lo)
    } else {
        u256_lo
    }
}

pub fn to_decimal(s: &str) -> Option<String> {
    let int = BigUint::from_str_radix(s, 16).ok();
    int.map(|n| n.to_str_radix(10))
}
