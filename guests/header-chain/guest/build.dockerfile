
FROM risczero/risc0-guest-builder:r0.1.81.0 as build

# FromAsCasing: 'as' and 'FROM' keywords' casing do not match

WORKDIR /src
COPY . .
ENV CARGO_MANIFEST_PATH="guests/header-chain/guest/Cargo.toml"
ENV RUSTFLAGS="-C passes=loweratomic -C link-arg=-Ttext=0x00200800 -C link-arg=--fatal-warnings"
ENV CARGO_TARGET_DIR="target"
ENV CC_riscv32im_risc0_zkvm_elf="/root/.local/share/cargo-risczero/cpp/bin/riscv32-unknown-elf-gcc"
ENV CFLAGS_riscv32im_risc0_zkvm_elf="-march=rv32im -nostdlib"
RUN cargo +risc0 fetch --locked --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH
RUN cargo +risc0 build --release --locked --target riscv32im-risc0-zkvm-elf --manifest-path $CARGO_MANIFEST_PATH

# export stage
FROM scratch as export
# FromAsCasing: 'as' and 'FROM' keywords' casing do not match

COPY --from=build /src/target/riscv32im-risc0-zkvm-elf/release /header_chain_guest