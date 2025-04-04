use std::{collections::HashMap, env, fs, path::Path};
use risc0_build::{embed_methods_with_options, DockerOptionsBuilder, GuestOptionsBuilder};

fn main() {
    // Build environment variables
    println!("cargo:rerun-if-env-changed=SKIP_GUEST_BUILD");
    println!("cargo:rerun-if-env-changed=REPR_GUEST_BUILD");
    println!("cargo:rerun-if-changed=../header_chain_build.dockerfile");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    
    // Compile time constant environment variables
    println!("cargo:rerun-if-env-changed=BITCOIN_NETWORK");
    println!("cargo:rerun-if-env-changed=TEST_SKIP_GUEST_BUILD");
    
    // Check if we should skip the guest build for tests
    if let Ok("1" | "true") = env::var("TEST_SKIP_GUEST_BUILD").as_deref() {
        println!("cargo:warning=Skipping guest build in test. Exiting");
        return;
    }
    
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
                pub const HEADER_CHAIN_ELF: &[u8] = &[];
                pub const HEADER_CHAIN_ID: [u32; 8] = [0u32; 8];
                "#;
                return fs::write(methods_path, elf).expect("Failed to write mock header chain elf");
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
    
    // Use embed_methods_with_options with our custom options
    let guest_pkg_to_options = get_guest_options();
    embed_methods_with_options(guest_pkg_to_options);
}

fn get_guest_options() -> HashMap<&'static str, risc0_build::GuestOptions> {
    let mut guest_pkg_to_options = HashMap::new();
    let mut features = Vec::new();
    
    // Add Bitcoin network feature if specified
    if let Ok(network) = env::var("BITCOIN_NETWORK") {
        println!("cargo:warning=Building for Bitcoin network: {}", network);
        features.push(format!("network-{}", network.to_lowercase()));
    }
    
    let opts = if env::var("REPR_GUEST_BUILD").is_ok() {
        let this_package_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get manifest dir");
        let root_dir = format!("{this_package_dir}/../../../");
        
        println!("cargo:warning=Using Docker for guest build with root dir: {}", root_dir);
        
        let docker_opts = DockerOptionsBuilder::default()
            .root_dir(root_dir)
            .build()
            .unwrap();
            
        GuestOptionsBuilder::default()
            .features(features)
            .use_docker(docker_opts)
            .build()
            .unwrap()
    } else {
        println!("cargo:warning=Guest code is not built in docker");
        GuestOptionsBuilder::default()
            .features(features)
            .build()
            .unwrap()
    };
    
    guest_pkg_to_options.insert("header-chain-guest", opts);
    guest_pkg_to_options
}