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

WORKDIR /src/
RUN git clone https://github.com/iden3/circomlib.git

# Cache ahead of the larger build process
FROM dependencies AS builder

WORKDIR /src/
COPY circuits/risc0.circom ./proof/circuits/risc0.circom
COPY circuits/journal.circom ./proof/circuits/journal.circom
COPY circuits/stark_verify.circom ./proof/circuits/stark_verify.circom
COPY circuits/verify_for_guest.circom ./proof/circuits/verify_for_guest.circom


# Build the r1cs
RUN (cd proof/circuits; circom --r1cs verify_for_guest.circom)

# Create a final clean image with all the dependencies to run the ceremony
FROM node AS ceremony

WORKDIR /ceremony

# install snarkjs globally
RUN npm install -g snarkjs@0.7.4

COPY scripts/run_ceremony.sh .
COPY groth16/pot23.ptau /ceremony/proof/groth16/pot23.ptau
COPY --from=builder /src/proof/circuits/verify_for_guest.r1cs /ceremony/proof/circuits/verify_for_guest.r1cs
RUN chmod +x run_ceremony.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/ceremony/run_ceremony.sh", "/ceremony/proof/circuits/verify_for_guest.r1cs", "/ceremony/proof/groth16/pot23.ptau"]
