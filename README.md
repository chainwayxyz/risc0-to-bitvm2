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

### Testing

To test the setup, use:

```bash
cargo test -r --package core --bin core -- tests --show-output
```
