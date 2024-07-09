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
COPY circuits/test_journal.circom ./proof/circuits/test_journal.circom
COPY circuits/test_stark_verify.circom ./proof/circuits/test_stark_verify.circom
COPY circuits/test_verify_for_guest.circom ./proof/circuits/test_verify_for_guest.circom


# Build the r1cs
RUN (cd proof/circuits; circom --r1cs test_verify_for_guest.circom)

# Create a final clean image with all the dependencies to run the preprocess
FROM node AS test_preprocess

WORKDIR /test_preprocess

# install snarkjs globally
RUN npm install -g snarkjs@0.7.4

COPY scripts/run_preprocess.sh .
COPY fflonk/pot24.ptau /test_preprocess/proof/fflonk/pot24.ptau
COPY --from=builder /src/proof/circuits/test_verify_for_guest.r1cs /test_preprocess/proof/circuits/test_verify_for_guest.r1cs
RUN chmod +x test_run_preprocess.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/test_preprocess/test_run_preprocess.sh", "/test_preprocess/proof/circuits/test_verify_for_guest.r1cs", "/test_preprocess/proof/fflonk/pot24.ptau"]
