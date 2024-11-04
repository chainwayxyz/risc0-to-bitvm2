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
