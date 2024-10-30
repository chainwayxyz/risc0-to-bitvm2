#![doc = include_str!("../README.md")]

use core::panic;

use bitcoin::{block, BlockHash};
use bitcoin::{block::Header, consensus::deserialize, secp256k1};
pub use bitcoin_pow_methods::CALCULATE_POW_ELF;
pub use bitcoin_pow_methods::CALCULATE_POW_ID;
use bitcoincore_rpc::RpcApi;
use risc0_zkvm::guest::env;
use risc0_zkvm::Journal;
use risc0_zkvm::ProverOpts;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use secp256k1::hashes::Hash;

pub fn calculate_pow(
    last_proven_blockhash: Option<BlockHash>,
    last_receipt: Option<Receipt>,
    chunk_size: u32,
    target_block_height: Option<u32>,
    k_depth: Option<u32>,
    output_type: u32,
) -> (Option<Receipt>, Option<Journal>, [u32; 8]) {
    let auth = bitcoincore_rpc::Auth::UserPass("admin".to_string(), "admin".to_string());
    let rpc = bitcoincore_rpc::Client::new("http://127.0.0.1:48332", auth).unwrap();
    if last_proven_blockhash.is_some() != last_receipt.clone().is_some() {
        panic!("Both last_proven_blockhash and last_receipt must be provided");
    }
    let mut blockhashes_to_prove = Vec::new(); // Each subvector is a chunk of blockhashes to be proven
    let mut blockheaders_to_prove = Vec::new(); // Each subvector is a chunk of blockheaders to be proven

    let start_block_height = last_proven_blockhash.map_or(0, |hash| {
        rpc.get_block(&hash).unwrap().bip34_block_height().unwrap() as u32 + 1
    });
    let end_block_height =
        target_block_height.unwrap_or_else(|| rpc.get_block_count().unwrap() as u32);

    if start_block_height > end_block_height {
        panic!("No blocks to prove");
    }
    println!("START BLOCK HEIGHT: {:?}", start_block_height);
    println!("END BLOCK HEIGHT: {:?}", end_block_height);

    let num_chunks = (end_block_height - start_block_height + 1) / chunk_size;
    let remainder = (end_block_height - start_block_height + 1) % chunk_size;

    for j in 0..num_chunks {
        blockhashes_to_prove.push(vec![]);
        blockheaders_to_prove.push(vec![]);
        for i in 0..chunk_size {
            let block_height = start_block_height + j * chunk_size + i;
            let blockhash = rpc.get_block_hash(block_height as u64).unwrap();
            let blockheader = rpc.get_block_header(&blockhash).unwrap();
            blockhashes_to_prove.last_mut().unwrap().push(blockhash);
            blockheaders_to_prove.last_mut().unwrap().push(blockheader);
        }
    }

    if remainder > 0 {
        blockhashes_to_prove.push(vec![]);
        blockheaders_to_prove.push(vec![]);
        for j in 0..remainder {
            let block_height = start_block_height + num_chunks * chunk_size + j;
            let blockhash = rpc.get_block_hash(block_height as u64).unwrap();
            let blockheader = rpc.get_block_header(&blockhash).unwrap();
            blockhashes_to_prove.last_mut().unwrap().push(blockhash);
            blockheaders_to_prove.last_mut().unwrap().push(blockheader);
        }
    }

    let mut prev_receipt: Option<Receipt> = last_receipt.clone();

    for (chunk_index, chunk) in blockheaders_to_prove.iter().enumerate() {
        let mut env = ExecutorEnv::builder();
        env.segment_limit_po2(21);

        if chunk_index == 0 && last_proven_blockhash.is_none() {
            env.write(&0).unwrap();
            println!("WRITE 0");
        } else {
            env.write(&1).unwrap();
            println!("WRITE 1");
            env.write(&CALCULATE_POW_ID).unwrap();
            println!("WRITE CALCULATE_POW_ID");
            env.add_assumption(prev_receipt.clone().unwrap());
            println!("ADD ASSUMPTION");
            env.write(&prev_receipt.clone().unwrap().journal.bytes).unwrap();
            println!("{:?}", prev_receipt.clone().unwrap());
            println!("WRITE JOURNAL BYTES: {:?}", prev_receipt.unwrap().journal.bytes);
        }

        // Write the current chunk headers to the environment
        env.write(&(chunk.len() as u32)).unwrap();
        println!("WRITE CHUNK LEN: {:?}", chunk.len());
        for header in chunk.iter() {
            env.write(&header.version).unwrap();
            println!("WRITE VERSION: {:?}", header.version);
            env.write(&header.merkle_root.as_byte_array()).unwrap();
            println!("WRITE MERKLE ROOT: {:?}", header.merkle_root);
            env.write(&header.time).unwrap();
            println!("WRITE TIME: {:?}", header.time);
            env.write(&header.bits).unwrap();
            println!("WRITE BITS: {:?}", header.bits);
            env.write(&header.nonce).unwrap();
            println!("WRITE NONCE: {:?}", header.nonce);
        }

        if chunk_index == blockheaders_to_prove.len() - 1 {
            if k_depth.is_some() {
                if output_type != 2 {
                    panic!("Output type incompatible");
                } else {
                    env.write(&2).unwrap();
                    println!("WRITE MODE2");
                    env.write(&k_depth.unwrap()).unwrap();
                    println!("WRITE K_DEPTH {:?}", k_depth.unwrap());
                    for i in 0..k_depth.unwrap() {
                        if i == 0 {
                            let blockhash = rpc.get_block_hash((end_block_height - k_depth.unwrap() + i + 1) as u64).unwrap();
                            env.write(blockhash.as_byte_array()).unwrap();   
                            println!("WRITE BLOCKHASH: {:?}", blockhash);                     
                        }
                        else {
                            let blockhash = rpc.get_block_hash((end_block_height - k_depth.unwrap() + i + 1) as u64).unwrap();
                            let header = rpc.get_block_header(&blockhash).unwrap();
                            env.write(&header.version).unwrap();
                            println!("WRITE VERSION: {:?}", header.version);
                            env.write(&header.merkle_root.as_byte_array()).unwrap();
                            println!("WRITE MERKLE ROOT: {:?}", header.merkle_root);
                            env.write(&header.time).unwrap();
                            println!("WRITE TIME: {:?}", header.time);
                            env.write(&header.bits).unwrap();
                            println!("WRITE BITS: {:?}", header.bits);
                            env.write(&header.nonce).unwrap();
                            println!("WRITE NONCE: {:?}", header.nonce);
                        }
                    }
                }
            } else {
                if output_type == 0 {
                    env.write(&0).unwrap();
                    println!("WRITE MODE0");
                } else if output_type == 1 {
                    env.write(&1).unwrap();
                    println!("WRITE MODE1");
                } else {
                    panic!("Output type invalid");
                }
            }
        } else {
            env.write(&0).unwrap();
            println!("WRITE MODE0");
        }

        let env = env.build().unwrap();
    
        // Obtain the default prover and prove the current chunk
        let prover = default_prover();
        let prover_opts = ProverOpts::succinct();
        let start_time = std::time::Instant::now();
        prev_receipt = Some(prover.prove_with_opts(env, CALCULATE_POW_ELF, &prover_opts).unwrap().receipt);
        let end_time = std::time::Instant::now();
        println!("PROOF TIME: {:?}", end_time - start_time);
        std::fs::write(format!("block_header_proofs/{}.json", chunk_index), serde_json::to_string(&prev_receipt.clone().unwrap()).unwrap()).unwrap(); // TODO: Write these receipts with the blockhash as the filename
        println!("CHUNK INDEX: {:?} PROVEN", chunk_index);
    }
    let final_journal = prev_receipt.clone().unwrap().journal;
    (prev_receipt.clone(), Some(final_journal), CALCULATE_POW_ID)
}