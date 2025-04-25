use risc0_binfmt::compute_image_id;
use risc0_build::{embed_methods_with_options, DockerOptionsBuilder, GuestOptionsBuilder};
use std::{collections::HashMap, env, fs, path::Path};

fn main() {
    // Build environment variables
    println!("cargo:rerun-if-env-changed=SKIP_GUEST_BUILD");
    println!("cargo:rerun-if-env-changed=REPR_GUEST_BUILD");
    println!("cargo:rerun-if-env-changed=OUT_DIR");

    // Compile time constant environment variables
    println!("cargo:rerun-if-env-changed=BITCOIN_NETWORK");
    println!("cargo:rerun-if-env-changed=TEST_SKIP_GUEST_BUILD");

    if std::env::var("CLIPPY_ARGS").is_ok() {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
        let dummy_path = Path::new(&out_dir).join("methods.rs");
        fs::write(dummy_path, "// dummy methods.rs for Clippy\n")
            .expect("Failed to write dummy methods.rs");
        println!("cargo:warning=Skipping guest build in Clippy");
        return;
    }

    // Check if we should skip the guest build for tests
    if let Ok("1" | "true") = env::var("TEST_SKIP_GUEST_BUILD").as_deref() {
        println!("cargo:warning=Skipping guest build in test. Exiting");
        return;
    }

    let network = env::var("BITCOIN_NETWORK").unwrap_or_else(|_| {
        println!("cargo:warning=BITCOIN_NETWORK not set, defaulting to 'mainnet'");
        "mainnet".to_string()
    });
    println!("cargo:warning=Building for Bitcoin network: {}", network);

    // Check if we should skip the guest build
    match env::var("SKIP_GUEST_BUILD") {
        Ok(value) => match value.as_str() {
            "1" | "true" => {
                println!("cargo:warning=Skipping guest build");
                let out_dir = env::var_os("OUT_DIR").unwrap();
                let out_dir = Path::new(&out_dir);
                let methods_path = out_dir.join("methods.rs");
                // Write empty ELF data for mock implementation
                let elf = r#"
                pub const FINAL_SPV_ELF: &[u8] = &[];
                pub const FINAL_SPV_ID: [u32; 8] = [0u32; 8];
                "#;
                return fs::write(methods_path, elf).expect("Failed to write mock final spv elf");
            }
            "0" | "false" => {
                println!("cargo:warning=Performing guest build");
            }
            _ => {
                println!("cargo:warning=Invalid value for SKIP_GUEST_BUILD: '{}'. Expected '0', '1', 'true', or 'false'. Defaulting to performing guest build.", value);
            }
        },
        Err(env::VarError::NotPresent) => {
            println!(
                "cargo:warning=SKIP_GUEST_BUILD not set. Defaulting to performing guest build."
            );
        }
        Err(env::VarError::NotUnicode(_)) => {
            println!("cargo:warning=SKIP_GUEST_BUILD contains invalid Unicode. Defaulting to performing guest build.");
        }
    }

    let is_repr_guest_build = match env::var("REPR_GUEST_BUILD") {
        Ok(value) => match value.as_str() {
            "1" | "true" => {
                println!("cargo:warning=REPR_GUEST_BUILD is set to true");
                true
            }
            "0" | "false" => {
                println!("cargo:warning=REPR_GUEST_BUILD is set to false");
                false
            }
            _ => {
                println!("cargo:warning=Invalid value for REPR_GUEST_BUILD: '{}'. Expected '0', '1', 'true', or 'false'. Defaulting to false.", value);
                false
            }
        },
        Err(env::VarError::NotPresent) => {
            println!("cargo:warning=REPR_GUEST_BUILD not set. Defaulting to false.");
            false
        }
        Err(env::VarError::NotUnicode(_)) => {
            println!(
                "cargo:warning=REPR_GUEST_BUILD contains invalid Unicode. Defaulting to false."
            );
            false
        }
    };

    // Use embed_methods_with_options with our custom options
    let guest_pkg_to_options = get_guest_options(network.clone());
    embed_methods_with_options(guest_pkg_to_options);

    // After the build is complete, copy the generated file to the elfs folder
    if is_repr_guest_build {
        println!("cargo:warning=Copying binary to elfs folder");
        copy_binary_to_elfs_folder(network);
    } else {
        println!("cargo:warning=Not copying binary to elfs folder");
    }
}

fn get_guest_options(network: String) -> HashMap<&'static str, risc0_build::GuestOptions> {
    let mut guest_pkg_to_options = HashMap::new();

    let opts = if env::var("REPR_GUEST_BUILD").is_ok() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get manifest dir");
        let root_dir = format!("{manifest_dir}/..");

        println!(
            "cargo:warning=Using Docker for guest build with root dir: {}",
            root_dir
        );

        let docker_opts = DockerOptionsBuilder::default()
            .root_dir(root_dir)
            .env(vec![("BITCOIN_NETWORK".to_string(), network.clone())])
            .build()
            .unwrap();

        GuestOptionsBuilder::default()
            // .features(features)
            .use_docker(docker_opts)
            .build()
            .unwrap()
    } else {
        println!("cargo:warning=Guest code is not built in docker");
        GuestOptionsBuilder::default()
            // .features(features)
            .build()
            .unwrap()
    };

    guest_pkg_to_options.insert("final-spv-guest", opts);
    guest_pkg_to_options
}

fn copy_binary_to_elfs_folder(network: String) {
    // Get manifest directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get manifest dir");
    let base_dir = Path::new(&manifest_dir);

    // Create elfs directory if it doesn't exist
    let elfs_dir = base_dir.join("../elfs");
    if !elfs_dir.exists() {
        fs::create_dir_all(&elfs_dir).expect("Failed to create elfs directory");
        println!("cargo:warning=Created elfs directory at {:?}", elfs_dir);
    }

    // Build source path
    let src_path = base_dir.join("../target/riscv-guest/final-spv/final-spv-guest/riscv32im-risc0-zkvm-elf/docker/final-spv-guest.bin");
    if !src_path.exists() {
        println!(
            "cargo:warning=Source binary not found at {:?}, skipping copy",
            src_path
        );
        return;
    }

    // Build destination path with network prefix
    let dest_filename = format!("{}-final-spv-guest.bin", network.to_lowercase());
    let dest_path = elfs_dir.join(&dest_filename);

    // Copy the file
    match fs::copy(&src_path, &dest_path) {
        Ok(_) => println!(
            "cargo:warning=Successfully copied binary to {:?}",
            dest_path
        ),
        Err(e) => println!("cargo:warning=Failed to copy binary: {}", e),
    }

    // Calculate and print method ID
    let elf_path = match network.as_str() {
        "mainnet" => "../elfs/mainnet-final-spv-guest.bin",
        "testnet4" => "../elfs/testnet4-final-spv-guest.bin",
        "signet" => "../elfs/signet-final-spv-guest.bin",
        "regtest" => "../elfs/regtest-final-spv-guest.bin",
        _ => {
            println!("cargo:warning=Invalid network specified, defaulting to mainnet");
            "../elfs/mainnet-final-spv-guest.bin"
        }
    };

    let elf_bytes: Vec<u8> = match fs::read(Path::new(elf_path)) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("cargo:warning=Failed to read ELF file: {}", e);
            return;
        }
    };

    let method_id = match compute_image_id(elf_bytes.as_slice()) {
        Ok(id) => id,
        Err(e) => {
            println!("cargo:warning=Failed to compute method ID: {}", e);
            return;
        }
    };

    println!("cargo:warning=Computed method ID: {:x?}", method_id);
    println!(
        "cargo:warning=Computed method ID words: {:?}",
        method_id.as_words()
    );
}
