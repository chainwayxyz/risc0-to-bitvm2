use std::{
    path::Path,
    process::{Command, Stdio},
};

use hello_world::multiply;
use risc0_groth16::{to_json, ProofJson};
use risc0_zkvm::{default_prover, get_prover_server, stark_to_snark, ProverOpts};
use tempfile::tempdir;

pub fn stark_to_fflonk(identity_p254_seal_bytes: &[u8]) {
    let tmp_dir = tempdir().unwrap();
    let work_dir = std::env::var("RISC0_WORK_DIR");
    let work_dir = work_dir.as_ref().map(Path::new).unwrap_or(tmp_dir.path());

    std::fs::write(work_dir.join("seal.r0"), identity_p254_seal_bytes).unwrap();
    let seal_path = work_dir.join("input.json");
    let proof_path = work_dir.join("proof.json");
    let mut seal_json = Vec::new();
    to_json(identity_p254_seal_bytes, &mut seal_json).unwrap();
    std::fs::write(seal_path, seal_json).unwrap();

    println!("Starting proving");

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(&format!("{}:/mnt", work_dir.to_string_lossy()))
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

    let contents = std::fs::read_to_string(proof_path).unwrap();
    let proof_json: ProofJson = serde_json::from_str(&contents).unwrap();
    // let proof = proof_json.try_into().unwrap();
    println!("proof: {:?}", proof_json);
}

fn main() {
    let (receipt, multiplication, image_id) = multiply(1923, 2024);
    // println!("Receipt: {:?}", receipt);
    // println!("Result: {:?}", multiplication);
    // println!("Journal: {:?}", receipt.journal);
    let composite_receipt = receipt.inner.composite().unwrap();
    let prover = get_prover_server(&ProverOpts::default()).unwrap();
    let succinct_receipt = prover.composite_to_succinct(composite_receipt).unwrap();
    // println!("Succinct receipt: {:?}", succinct_receipt);
    let ident_receipt = prover.identity_p254(&succinct_receipt).unwrap();

    let identity_p254_seal_bytes = ident_receipt.get_seal_bytes();

    let fflonk_proof = stark_to_fflonk(&identity_p254_seal_bytes);
    // let groth16_proof = stark_to_snark(&identity_p254_seal_bytes).unwrap();

    // let compressed_proof = prover.compress(&ProverOpts::groth16(), &receipt).unwrap();
    // println!("Compressed proof: {:?}", compressed_proof);
}
