FROM risczero/risc0-guest-builder:r0.1.85.0 as build

WORKDIR /src

COPY . .

ENV CARGO_MANIFEST_PATH="header-chain/guest/Cargo.toml"
ENV RUSTFLAGS="-C passes=lower-atomic -C link-arg=-Ttext=0x00200800 -C link-arg=--fatal-warnings"
ENV CARGO_TARGET_DIR="target"
ENV RISC0_FEATURE_bigint2=""
ENV CC_riscv32im_risc0_zkvm_elf="/root/.risc0/cpp/bin/riscv32-unknown-elf-gcc"
ENV CFLAGS_riscv32im_risc0_zkvm_elf="-march=rv32im -nostdlib"

RUN cargo +risc0 fetch --locked --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH
RUN cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH

# export stage
FROM scratch as export
COPY --from=build /src /output