use std::collections::HashMap;
use risc0_build::{DockerOptions, GuestOptions};

fn main() {
   println!("cargo:rerun-if-env-changed=REPR_GUEST_BUILD");
   println!("cargo:rerun-if-env-changed=OUT_DIR");

   let mut options = HashMap::new();

   let bitcoin_network = std::env::var("BITCOIN_NETWORK")
       .unwrap_or_else(|_| "mainnet".to_string());
   
//    println!("cargo:rustc-cfg=bitcoin_network=\"{}\"", bitcoin_network);

   let use_docker = if std::env::var("REPR_GUEST_BUILD").is_ok() {
       let this_package_dir = std::env::var("CARGO_MANIFEST_DIR")
           .expect("Failed to get CARGO_MANIFEST_DIR");
       let root_dir = format!("{}/../../", this_package_dir);
       
       Some(DockerOptions {
           root_dir: Some(root_dir.into()),
       })
   } else {
       println!("cargo:warning=Guest code is not built in docker");
       None
   };

//    println!("cargo:rustc-env=CARGO_FEATURE_{}", bitcoin_network.to_uppercase());
   println!("cargo:rustc-cfg=feature=\"{}\"", bitcoin_network);

   options.insert(
       "header-chain-guest",
       GuestOptions {
           use_docker,
           ..Default::default()
       },
   );

   risc0_build::embed_methods_with_options(options);
}