# syntax=docker/dockerfile:1.4
FROM rust:1.85.0 AS dependencies

WORKDIR /src/

# APT deps
RUN apt -qq update && \
  apt install -y -q apt-transport-https build-essential clang cmake curl gnupg libgmp-dev libsodium-dev m4 nasm nlohmann-json3-dev npm protobuf-compiler

WORKDIR /src/

# Build and install circom
RUN git clone https://github.com/iden3/circom.git && \
  cd circom && \
  git checkout e60c4ab8a0b55672f0f42fbc68a74203bdb6a700 && \
  cargo install --path circom

ENV CC=clang
ENV CXX=clang++

# Build rapidsnark
RUN git clone https://github.com/iden3/rapidsnark.git && \
  cd rapidsnark && \
  git checkout 998383787ee86bcb6bfb8741e9a638d203c08eae

WORKDIR /src/rapidsnark/

# Copied from: https://github.com/iden3/rapidsnark/blob/main/README.md
RUN git submodule init && \
  git submodule update && \
  ./build_gmp.sh aarch64 && \
  make host_arm64 && \
  cp ./package_arm64/bin/prover /usr/local/sbin/rapidsnark

WORKDIR /src/
RUN git clone https://github.com/iden3/circomlib.git

RUN git clone https://github.com/iden3/circom-witnesscalc.git && \
  cd circom-witnesscalc && \
  git checkout 14c05e8779dab64f1a338a31edd2b98f7fba4f33 && \
  cargo install --path ./extensions/build-circuit && \
  cargo install --path .

  
# Cache ahead of the larger build process
FROM dependencies AS builder

WORKDIR /src/
COPY circuits/stark_verify.circom ./groth16_proof/circuits/stark_verify.circom
COPY circuits/verify_for_guest.circom ./groth16_proof/circuits/verify_for_guest.circom
COPY circuits/blake3_compression.circom ./groth16_proof/circuits/blake3_compression.circom
COPY circuits/blake3_common.circom ./groth16_proof/circuits/blake3_common.circom
COPY circuits/risc0.circom ./groth16_proof/circuits/risc0.circom

# Delete the last line of stark_verify.circom so that we only use its template
RUN sed -i '$d' ./groth16_proof/circuits/stark_verify.circom

# Build the circuit graph to be used in calc-witness
RUN (cd groth16_proof/circuits; /usr/local/cargo/bin/build-circuit verify_for_guest.circom /verify_for_guest.bin)

# Create a final clean image with all the dependencies to perform stark->snark
FROM ubuntu:jammy-20231211.1@sha256:bbf3d1baa208b7649d1d0264ef7d522e1dc0deeeaaf6085bf8e4618867f03494 AS prover

RUN apt update -qq && \
  apt install -y libsodium23 nodejs npm wget && \
  npm install -g snarkjs@0.7.3

COPY scripts/prover.sh /app/prover.sh
COPY --from=dependencies /usr/local/cargo/bin/calc-witness /app/
COPY --from=builder /verify_for_guest.r1cs /app/
COPY --from=builder /usr/local/sbin/rapidsnark /usr/local/sbin/rapidsnark
RUN wget -O /app/verify_for_guest_final.zkey https://static.testnet.citrea.xyz/conf/verify_for_guest_final.zkey

WORKDIR /app
RUN chmod +x prover.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/app/prover.sh"]
