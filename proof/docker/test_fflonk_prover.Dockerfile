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

# Build the witness generation
RUN (cd proof/circuits; circom --c --r1cs test_verify_for_guest.circom) && \
  sed -i 's/g++/clang++/' proof/circuits/test_verify_for_guest_cpp/Makefile && \
  sed -i 's/O3/O0/' proof/circuits/test_verify_for_guest_cpp/Makefile && \
  (cd proof/circuits/test_verify_for_guest_cpp; make)

# Create a final clean image with all the dependencies to perform stark->snark
FROM ubuntu:jammy-20231211.1@sha256:bbf3d1baa208b7649d1d0264ef7d522e1dc0deeeaaf6085bf8e4618867f03494 AS prover

RUN apt update -qq && \
  apt install -y libsodium23 nodejs npm && \
  npm install -g snarkjs@0.7.3

COPY scripts/test_fflonk_prover.sh /app/test_fflonk_prover.sh
COPY --from=builder /src/proof/circuits/test_verify_for_guest_cpp/test_verify_for_guest /app/test_verify_for_guest
COPY --from=builder /src/proof/circuits/test_verify_for_guest_cpp/test_verify_for_guest.dat /app/test_verify_for_guest.dat
COPY fflonk/test_fflonk.zkey /app/test_fflonk.zkey

WORKDIR /app
RUN chmod +x test_fflonk_prover.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/app/test_fflonk_prover.sh"]