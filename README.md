# Risc0 to BitVM

**Warning: This software is experimental and should not be used in production.**

## Building Risc0 Guests

To build Risc0 guests deterministically, run the following command:

```bash
REPR_GUEST_BUILD=1 cargo build --release
```

## Proving Bitcoin Headers

To prove Bitcoin headers, first download the Bitcoin headers:

```bash
wget https://zerosync.org/chaindata/headers.bin
```

### Usage

```bash
./target/release/core None first_10.bin 10
```

- The first argument is the previous proof file path (`None` if starting from genesis).
- The second argument is the output proof file path.
- The third argument is the number of headers to prove.

Example: To verify the previous proof and prove the next 90 Bitcoin headers, run the following command:

```bash
./target/release/core first_10.bin first_100.bin 90
```

## Risc0 to Succinct Proofs

BitVM requires a Groth16 proof with one public input. We have implemented the necessary functionalities to support this.

## Setup

First download Powers of Tau ceremony files:

```
wget -O ./proof/groth16/pot23.ptau https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_23.ptau
```

To run the ceremony for `groth16` proof:
```
cd groth16_proof
docker build -f docker/ceremony.Dockerfile . -t snark-ceremony
docker run --rm -v $(pwd)/groth16:/ceremony/proof/groth16 snark-ceremony
```

To build the prover:
```
docker build -f docker/prover.Dockerfile . -t risc0-groth16-prover
```


### Testing

To test the setup, use:

```bash
cargo test -r --package core --bin core -- tests --show-output
```

## Our Approach
### Goal
Our goal is to be able to (optimistically) prove any computation inside BitVM. Overall system is as follows:
Any computation -> ZK Proof 1 (Succinct STARK Proof - via Risc0 ZKVM) -> ZK Proof 2 with constant sized (32 bytes) output (Succinct STARK Proof - via Risc0 ZKVM) -> Groth16 Proof with a single public output binding the previous circuits with Blake3 hashing (via Risc0's STARK Verifier Circom circuit + Our Circom circuit for end-to-end binding) -> BitVM.
Here, the journals with non-constant sizes of the general purpose circuits will be verified in BitVM to ensure the correctness of the claims.
### Bitcoin
 In the case of Bitcoin, it is the bridge operations (PegIn/PegOut). This requires the proving of the Bitcoin block headers. With `header-chain-circuit`, one can prove the current state of the Bitcoin given the block headers. It does not necessarily prevent the malicious actors to generate proofs for their private forks, but the calculation of the `ChainState` is the basis for the conflict resolution. For more, see:
 [Proof of work](https://en.bitcoin.it/wiki/Proof_of_work).
