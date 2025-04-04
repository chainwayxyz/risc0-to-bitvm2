FROM risczero/risc0-guest-builder:r0.1.85.0 as build

WORKDIR /src

COPY header-chain header-chain
COPY core core

ENV CARGO_MANIFEST_PATH="header-chain/guest/Cargo.toml"
ENV CARGO_ENCODED_RUSTFLAGS="-Cpasses=lower-atomic-Clink-arg=-Ttext=0x00200800-Clink-arg=--fatal-warnings-Cpanic=abort"
ENV CARGO_TARGET_DIR="header-chain/guest/target"
ENV RISC0_FEATURE_bigint2=""
ENV CC_riscv32im_risc0_zkvm_elf="/root/.risc0/cpp/bin/riscv32-unknown-elf-gcc"
ENV CFLAGS_riscv32im_risc0_zkvm_elf="-march=rv32im -nostdlib"

# Set network environment variable
ARG BITCOIN_NETWORK=mainnet
ENV BITCOIN_NETWORK=${BITCOIN_NETWORK}

RUN cargo +risc0 fetch --locked --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH
RUN cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH

# export stage
FROM scratch AS export
ARG BITCOIN_NETWORK
# COPY --from=build /src/header-chain/guest ../output
COPY --from=build /src ../output
# COPY --from=build /src/header-chain/guest/target/riscv32im-risc0-zkvm-elf/release/header-chain-guest.bin ../elfs/${BITCOIN_NETWORK}-header-chain-guest.bin

# WORKDIR /src

# # Copy the entire project structure
# COPY header-chain header-chain
# COPY core core
# # Set compile-time environment variables
# ENV CARGO_MANIFEST_PATH="header-chain/guest/Cargo.toml"
# # ENV CARGO_ENCODED_RUSTFLAGS="-C passes=lower-atomic -C link-arg=-Ttext=0x00200800 -C link-arg=--fatal-warnings -C panic=abort"
# ENV CARGO_TARGET_DIR="header-chain/guest/target"
# ENV CC_riscv32im_risc0_zkvm_elf="/root/.risc0/cpp/bin/riscv32-unknown-elf-gcc"
# ENV CFLAGS_riscv32im_risc0_zkvm_elf="-march=rv32im -nostdlib"

# RUN rzup show
# RUN cargo +risc0 --version

# # Set network environment variable
# ARG BITCOIN_NETWORK=mainnet
# ENV BITCOIN_NETWORK=${BITCOIN_NETWORK}

# RUN RUST_FLAGS=$(echo -e "-C\x1fpasses=lower-atomic\x1f-C\x1flink-arg=-Ttext=0x00200800\x1f-C\x1flink-arg=--fatal-warnings\x1f-C\x1fpanic=abort") && \
#     export CARGO_ENCODED_RUSTFLAGS="$RUST_FLAGS" && \
#     cargo +risc0 fetch --locked --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH && \
#     cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH
