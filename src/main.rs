use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use bitcoin_pow::calculate_pow;
use crypto_bigint::U256;
use hello_world::multiply;
use num_bigint::BigUint;
use num_traits::Num;
use risc0_groth16::{to_json, Fr, ProofJson, Seal};
use risc0_zkvm::{default_prover, get_prover_server, Groth16Receipt, ProverOpts, VerifierContext};
use serde_json::Value;
use std::env;
use std::str::FromStr;
use tempfile::tempdir;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub fn stark_to_fflonk(identity_p254_seal_bytes: &[u8], journal: &[u8], pre_state_bits: &[u8]) {
    let tmp_dir = tempdir().unwrap();
    let work_dir = std::env::var("RISC0_WORK_DIR");
    let work_dir = work_dir.as_ref().map(Path::new).unwrap_or(tmp_dir.path());

    std::fs::write(work_dir.join("seal.r0"), identity_p254_seal_bytes).unwrap();
    let seal_path = work_dir.join("input.json");
    let proof_path = work_dir.join("proof.json");
    let mut seal_json = Vec::new();
    // println!("Seal: {:?}", seal_json);
    to_json(identity_p254_seal_bytes, &mut seal_json).unwrap();
    let journal_hex = hex::encode(journal);
    println!("Journal hex: {:?}", journal_hex);
    let mut journal_bits = Vec::new();
    for byte in journal {
        for i in 0..8 {
            journal_bits.push((byte >> (7 - i)) & 1);
        }
    }
    println!("Journal bits: {:?}", journal_bits);
    let q = journal_bits.len() / 252;
    let r = journal_bits.len() % 252;
    let mut journal_chunks: Vec<U256> = Vec::new();
    for i in 0..q {
        journal_chunks.push(bits_to_num(
            252,
            &journal_bits[i * 252..(i + 1) * 252].to_vec(),
        ));
    }
    if r > 0 {
        journal_chunks.push(bits_to_num(r, &journal_bits[q * 252..].to_vec()));
    }
    println!("Journal chunks: {:?}", journal_chunks);
    // let journal_json = serde_json::json!({ "journal": journal_bits });
    // std::fs::write(seal_path.clone(), seal_json).unwrap();
    // read the seal file
    // let seal_contents = std::fs::read_to_string(seal_path.clone()).unwrap();

    // println!("Seal contents: {:?}", seal_contents);
    let mut seal_json: Value = {
        let file_content = fs::read_to_string(&seal_path).unwrap();
        serde_json::from_str(&file_content).unwrap()
    };
    // Now extend the seal json by adding journal
    let journal_str_vec = journal_chunks
        .iter()
        .map(|s| to_decimal(&s.to_string()).unwrap())
        .collect::<Vec<String>>();
    println!("Journal str vec: {:?}", journal_str_vec);
    seal_json["journal"] = journal_str_vec.into();
    println!("pre_state_bits: {:?}", pre_state_bits);
    seal_json["pre_state_digest_bits"] = pre_state_bits.to_vec().into();
    std::fs::write(seal_path, serde_json::to_string_pretty(&seal_json).unwrap()).unwrap();

    println!("Starting proving");

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(&format!("{}:/mnt", work_dir.to_string_lossy()))
        .arg("risc0-fflonk-prover")
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

    let contents = std::fs::read_to_string(proof_path).unwrap();
    let proof_json: ProofJson = serde_json::from_str(&contents).unwrap();
    // let proof = proof_json.try_into().unwrap();
    println!("proof: {:?}", proof_json);
}

pub fn bits_to_num(len: usize, bits: &Vec<u8>) -> U256 {
    assert!(len <= 252);
    assert!(bits.len() == len as usize);
    let mut num_lo: u128 = 0;
    if len > 128 {
        let mut num_hi: u128 = 0;
        for i in 0..128 {
            num_lo = num_lo + ((bits[i] as u128) << i);
        }
        for i in 128..len {
            num_hi = num_hi + ((bits[i] as u128) << (i - 128));
        }
        let u256_lo = U256::from(num_lo);
        let u256_hi = U256::from(num_hi) << 128;
        return u256_hi.wrapping_add(&u256_lo);
    } else {
        for i in 0..len {
            num_lo = num_lo + ((bits[i] as u128) << i);
        }
        let num_lo: U256 = U256::from(num_lo);
        num_lo
    }
}

pub fn to_decimal(s: &str) -> Option<String> {
    let int = BigUint::from_str_radix(s, 16).ok();
    int.map(|n| n.to_str_radix(10))
}

fn main() {
    // initialize_logging();
    let (receipt, (last_block_hash, pow), IMAGE_ID) = calculate_pow();
    println!("IMAGE_ID: {:?}", IMAGE_ID);
    let mut pre_state_bits: Vec<u8> = Vec::new();
    for i in 0..8 {
        for j in 0..4 {
            for k in 0..8 {
                pre_state_bits.push((IMAGE_ID[i] >> (8 * j + 7 - k)) as u8 & 1);
            }
        }
    }
    println!("Pre state bits: {:?}", pre_state_bits);
    // pre_state_bits.reverse();
    // println!("Receipt: {:?}", receipt);
    // println!("Result: {:?}", multiplication)
    println!("Seal claim: {:?}", receipt.inner.claim());
    let journal_bytes = receipt.journal.bytes;
    let composite_receipt = receipt.inner.composite().unwrap();
    let prover = get_prover_server(&ProverOpts::default()).unwrap();
    let succinct_receipt = prover.composite_to_succinct(composite_receipt).unwrap();
    let pre_state_digest = receipt.inner.claim().ok().unwrap();
    println!("Succinct receipt claim: {:?}", receipt.inner.claim());
    let groth16_proof = prover.succinct_to_groth16(&succinct_receipt).unwrap();
    let res = groth16_proof.verify_integrity_with_context(&VerifierContext::default());
    println!("Verification result: {:?}", res);
    let ident_receipt = prover.identity_p254(&succinct_receipt).unwrap();
    println!(
        "Identity receipt control_id: {:?}",
        ident_receipt.control_id
    );

    let identity_p254_seal_bytes = ident_receipt.get_seal_bytes();
    let fflonk_proof = stark_to_fflonk(&identity_p254_seal_bytes, &journal_bytes, &pre_state_bits);
    // let groth16_proof = stark_to_snark(&identity_p254_seal_bytes).unwrap();

    // let compressed_proof = prover.compress(&ProverOpts::groth16(), &receipt).unwrap();
    // println!("Compressed proof: {:?}", compressed_proof);
}

pub fn initialize_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::from_str(
                &env::var("RUST_LOG").unwrap_or_else(|_| "debug,bitcoincore_rpc=info".to_string()),
            )
            .unwrap(),
        )
        .init();
}
