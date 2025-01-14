use std::time::{Duration, Instant};

use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::{utils::parse_ether, Address};
use anyhow::{bail, ensure};
use borsh::BorshDeserialize;
use boundless_market::{
    client::ClientBuilder,
    contracts::{Input, Offer, Predicate, ProofRequest, Requirements},
    storage::StorageProviderConfig,
};
use clap::Parser;
use header_chain::header_chain::{CircuitBlockHeader, HeaderChainCircuitInput};
use risc0_zkvm::sha::Digestible;
use risc0_zkvm::{compute_image_id, default_executor, ExecutorEnv};
use url::Url;

const ELF: &[u8] = include_bytes!("../../elfs/mainnet-header-chain-guest");

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// URL of the Ethereum RPC endpoint.
    #[clap(short, long, env)]
    rpc_url: Url,
    /// Private key used to interact with the EvenNumber contract.
    #[clap(short, long, env)]
    wallet_private_key: PrivateKeySigner,
    /// Submit the request offchain via the provided order stream service url.
    #[clap(short, long, requires = "order_stream_url")]
    offchain: bool,
    /// Offchain order stream service URL to submit offchain requests to.
    #[clap(long, env)]
    order_stream_url: Option<Url>,
    /// Storage provider to use
    #[clap(flatten)]
    storage_config: Option<StorageProviderConfig>,
    /// Address of the RiscZeroSetVerifier contract.
    #[clap(short, long, env)]
    set_verifier_address: Address,
    /// Address of the BoundlessfMarket contract.
    #[clap(short, long, env)]
    boundless_market_address: Address,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match dotenvy::dotenv() {
        Ok(path) => tracing::debug!("Loaded environment variables from {:?}", path),
        Err(e) if e.not_found() => tracing::debug!("No .env file found"),
        Err(e) => bail!("failed to load .env file: {}", e),
    }
    let args = Args::parse();

    let image_id = compute_image_id(ELF).unwrap();

    let headers = include_bytes!("../../mainnet-headers.bin");
    let headers = headers
        .chunks(80)
        .map(|header| CircuitBlockHeader::try_from_slice(header).unwrap())
        .collect::<Vec<CircuitBlockHeader>>();

    let input = HeaderChainCircuitInput {
        method_id: [0; 8],
        prev_proof: header_chain::header_chain::HeaderChainPrevProofType::GenesisBlock,
        block_headers: headers[0..50].to_vec(),
    };

    // Create a Boundless client from the provided parameters.
    let boundless_client = ClientBuilder::default()
        .with_rpc_url(args.rpc_url)
        .with_boundless_market_address(args.boundless_market_address)
        .with_set_verifier_address(args.set_verifier_address)
        .with_order_stream_url(args.offchain.then_some(args.order_stream_url).flatten())
        .with_storage_provider_config(args.storage_config)
        .with_private_key(args.wallet_private_key)
        .build()
        .await?;

    ensure!(
        boundless_client.storage_provider.is_some(),
        "a storage provider is required to upload the zkVM guest ELF"
    );
    let image_url = boundless_client.upload_image(ELF).await?;
    tracing::info!("Uploaded image to {}", image_url);

    let input_bytes = borsh::to_vec(&input).unwrap();

    // If the input exceeds 2 kB, upload the input and provide its URL instead, as a rule of thumb.
    let input_url = boundless_client.upload_input(&input_bytes).await?;
    tracing::info!("Uploaded input to {}", input_url);

    let env = ExecutorEnv::builder().write_slice(&input_bytes).build()?;
    let session_info = default_executor().execute(env, ELF)?;
    let mcycles_count = session_info
        .segments
        .iter()
        .map(|segment| 1 << segment.po2)
        .sum::<u64>()
        .div_ceil(1_000_000);
    println!("{} mcycles", mcycles_count);
    let journal = session_info.journal;
    println!("Journal: {:#?}", journal);

    let request = ProofRequest::default()
        .with_image_url(&image_url)
        .with_input(Input::url(&input_url))
        .with_requirements(Requirements::new(
            image_id,
            Predicate::digest_match(journal.digest()),
        ))
        .with_offer(
            Offer::default()
                // The market uses a reverse Dutch auction mechanism to match requests with provers.
                // Each request has a price range that a prover can bid on. One way to set the price
                // is to choose a desired (min and max) price per million cycles and multiply it
                // by the number of cycles. Alternatively, you can use the `with_min_price` and
                // `with_max_price` methods to set the price directly.
                .with_min_price_per_mcycle(parse_ether("0.001")?, mcycles_count)
                // NOTE: If your offer is not being accepted, try increasing the max price.
                .with_max_price_per_mcycle(parse_ether("0.002")?, mcycles_count)
                // The timeout is the maximum number of blocks the request can stay
                // unfulfilled in the market before it expires. If a prover locks in
                // the request and does not fulfill it before the timeout, the prover can be
                // slashed.
                .with_timeout(1000),
        );
    println!("Request: {:#?}", request);

    // Send the request and wait for it to be completed.
    let start_time = Instant::now();
    let (request_id, expires_at) = boundless_client.submit_request(&request).await?;
    tracing::info!("Request 0x{request_id:x} submitted at {:?}", start_time);

    // Wait for the request to be fulfilled by the market, returning the journal and seal.
    tracing::info!("Waiting for 0x{request_id:x} to be fulfilled");
    let (journal, seal) = boundless_client
        .wait_for_request_fulfillment(request_id, Duration::from_secs(10), expires_at)
        .await?;

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);

    tracing::info!("Request 0x{request_id:x} fulfilled");
    tracing::info!("End time: {:?}", end_time);
    tracing::info!("Time taken: {:?}", duration);
    tracing::info!("Request 0x{request_id:x} fulfilled");
    tracing::info!("Journal: {:#?}", journal);
    tracing::info!("Seal: {:#?}", seal);

    Ok(())
}
