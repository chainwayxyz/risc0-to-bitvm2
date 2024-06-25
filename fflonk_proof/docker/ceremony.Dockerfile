# syntax=docker/dockerfile:1.4
FROM rust:1.74.0 AS dependencies

WORKDIR /src/

# APT deps
RUN apt -qq update && \
  apt install -y -q apt-transport-https build-essential clang cmake curl gnupg libgmp-dev libsodium-dev m4 nasm nlohmann-json3-dev npm

WORKDIR /src/

# Build and install circom
RUN git clone https://github.com/iden3/circom.git && \
  cd circom && \
  git checkout e60c4ab8a0b55672f0f42fbc68a74203bdb6a700 && \
  cargo install --path circom

ENV CC=clang
ENV CXX=clang++

# Cache ahead of the larger build process
FROM dependencies AS builder

WORKDIR /src/
COPY groth16/circuits/aliascheck.circom ./groth16/circuits/aliascheck.circom
COPY groth16/circuits/binsum.circom ./groth16/circuits/binsum.circom
COPY groth16/circuits/bitify.circom ./groth16/circuits/bitify.circom
COPY groth16/circuits/comparators.circom ./groth16/circuits/comparators.circom
COPY groth16/circuits/compconstant.circom ./groth16/circuits/compconstant.circom
COPY groth16/circuits/risc0.circom ./groth16/circuits/risc0.circom
COPY groth16/circuits/journal.circom ./groth16/circuits/journal.circom
COPY groth16/circuits/stark_verify.circom ./groth16/circuits/stark_verify.circom
COPY groth16/circuits/verify_for_guest.circom ./groth16/circuits/verify_for_guest.circom
COPY groth16/circuits/sha256 ./groth16/circuits/sha256


# Build the r1cs
RUN (cd groth16/circuits; circom --r1cs verify_for_guest.circom)

# Create a final clean image with all the dependencies to run the ceremony
FROM node AS ceremony

WORKDIR /ceremony

# install snarkjs globally
RUN npm install -g snarkjs@0.7.4

COPY scripts/run_ceremony.sh .
COPY --from=builder /src/groth16/circuits/verify_for_guest.r1cs /ceremony/circuits/verify_for_guest.r1cs
RUN chmod +x run_ceremony.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/ceremony/run_ceremony.sh", "/ceremony/circuits/verify_for_guest.r1cs", "/ceremony/groth16/pot23.ptau"]
