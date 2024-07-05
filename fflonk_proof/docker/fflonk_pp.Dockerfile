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
COPY groth16/circuits/aliascheck.circom ./fflonk/circuits/aliascheck.circom
COPY groth16/circuits/binsum.circom ./fflonk/circuits/binsum.circom
COPY groth16/circuits/bitify.circom ./fflonk/circuits/bitify.circom
COPY groth16/circuits/comparators.circom ./fflonk/circuits/comparators.circom
COPY groth16/circuits/compconstant.circom ./fflonk/circuits/compconstant.circom
COPY groth16/circuits/risc0.circom ./fflonk/circuits/risc0.circom
COPY groth16/circuits/test_journal.circom ./fflonk/circuits/test_journal.circom
COPY groth16/circuits/test_stark_verify.circom ./fflonk/circuits/test_stark_verify.circom
COPY groth16/circuits/test_verify_for_guest.circom ./fflonk/circuits/test_verify_for_guest.circom
COPY groth16/circuits/sha256 ./fflonk/circuits/sha256


# Build the r1cs
RUN (cd fflonk/circuits; circom --r1cs test_verify_for_guest.circom)

# Create a final clean image with all the dependencies to run the preprocess
FROM node AS test_preprocess

WORKDIR /test_preprocess

# install snarkjs globally
RUN npm install -g snarkjs@0.7.4

COPY scripts/run_preprocess.sh .
COPY --from=builder /src/fflonk/circuits/test_verify_for_guest.r1cs /test_preprocess/circuits/test_verify_for_guest.r1cs
RUN chmod +x run_preprocess.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/test_preprocess/run_preprocess.sh", "/test_preprocess/circuits/test_verify_for_guest.r1cs", "/test_preprocess/fflonk/pot24.ptau"]
