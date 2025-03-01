# Risc0 to BitVM2

**Warning: This software is experimental and should not be used in production.**

## Building Risc0 Guests
First, install `Risc0` toolchain. You can refer to [here](https://dev.risczero.com/api/zkvm/install).

To build Risc0 guests deterministically, run the following command:

```bash
BITCOIN_NETWORK=mainnet REPR_GUEST_BUILD=1 cargo build --release
```

## Proving Bitcoin Headers

To prove Bitcoin headers, first download the Bitcoin headers and rename it to "mainnet-headers.bin":

```bash
wget https://static.testnet.citrea.xyz/conf/headers.bin -O data/headers/mainnet-headers.bin
```

### Usage

```bash
./target/release/host None data/proofs/mainnet/mainnet_first_9.bin 10
```

- The first argument is the previous proof file path (`None` if starting from genesis).
- The second argument is the output proof file path.
- The third argument is the number of headers to prove.

Example: To verify the previous proof and prove the next 90 Bitcoin headers, run the following command:

```bash
./target/release/host data/proofs/mainnet/mainnet_first_9.bin data/proofs/mainnet/mainnet_first_99.bin 90
```

## Risc0 to Succinct Proofs

BitVM requires a Groth16 proof (preferably with one public input). We have implemented the necessary functionalities to support this.

## Setup

First download the STARK Verify Circom circuit:

```
git lfs pull
```

To build the prover:
```
cd groth16_proof
docker build -f docker/prover.Dockerfile . -t risc0-groth16-prover
cd ..
```

### Testing

To test the setup, use:

```bash
cargo test -r --package host --bin core -- tests --show-output
```

## Boundless
Boundless is a decentralized proving network built by Risc0. You can learn more from [here](https://docs.beboundless.xyz/). To use it,
```
RUST_LOG=info PINATA_JWT=<your-pinata-jwt> cargo run --package boundless-client --bin boundless-client -- --rpc-url <your-sepolia-rpc-url> --wallet-private-key <your-private-key> --boundless-market-address 0x01e4130C977b39aaa28A744b8D3dEB23a5297654 --set-verifier-address 0xea6a0Ca4BfD0A6C43081D57672b3B6D43B69265F --storage-provider pinata  --offchain --order-stream-url https://order-stream.beboundless.xyz
```

## Our Approach
### Goal
Our goal is to be able to (optimistically) prove any computation inside BitVM. Overall system is as follows:
<div align="center">
  <b>Any computation</b> <br> 
  ⬇️ <br>
  <b>ZK Proof 1</b>: Succinct STARK Proof via Risc0 ZKVM <br>
  ⬇️ <br>
  <b>ZK Proof 2</b>: With constant-sized (32 bytes) digest of the journal of the previous proof (Succinct STARK Proof via Risc0 ZKVM) <br>
  ⬇️ <br>
  <b>Groth16 Proof</b>: Single public output binding the previous circuits with Blake3 hashing <br>
  <i>(via Risc0's STARK Verifier Circom circuit + our Circom circuit for end-to-end binding)</i> <br>
  ⬇️ <br>
  <b>BitVM</b>
</div>

> Here, the journals with non-constant sizes of the general-purpose circuits will be digested (etc. hashing using Blake3) with the circuit constants (`general_purpose_circuit_method_id`, `final_circuit_method_id`, `pre_state`, `post_state`, etc.) in BitVM to ensure the correctness of the claims.

### Bitcoin
 In the case of Bitcoin, the main computations we want to prove are the bridge operations (PegIn/PegOut). This requires the proving of the Bitcoin block headers. With `header-chain-circuit`, one can prove the current state of the Bitcoin given the block headers. It does not necessarily prevent the malicious actors to generate proofs for their private forks, but the calculation of the `ChainState` is the basis for the conflict resolution. For more, see:
 [Proof of work](https://en.bitcoin.it/wiki/Proof_of_work).
 With a small modification to `final-circuit`, anyone can prove the inclusion of a transaction inside a block on the header chain.


## Acknowledgments
- [Risc0](https://github.com/risc0/risc0): This repository is built using the technology stack of Risc0, more specifically Risc0 ZKVM and their STARK verifier Circom circuit.
- [Blake3 Circom](https://github.com/banyancomputer/hot-proofs-blake3-circom): The Circom circuit of the Blake3 hash function implementation is taken from here.
- [Bitcoin Header Chain Proof](https://github.com/ZeroSync/header_chain/tree/master/program/src/block_header): Our Bitcoin header chain proof circuit implementation is inspired by the Bitcoin Core implementation and ZeroSync's header chain proof circuit.
