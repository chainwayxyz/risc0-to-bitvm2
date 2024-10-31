// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use risc0_build::{DockerOptions, GuestOptions};

fn main() {
    println!("cargo:rerun-if-env-changed=REPR_GUEST_BUILD");
    println!("cargo:rerun-if-env-changed=OUT_DIR");

    let mut options = HashMap::new();

    let use_docker = if std::env::var("REPR_GUEST_BUILD").is_ok() {
        let this_package_dir = std::env!("CARGO_MANIFEST_DIR");
        let root_dir = format!("{this_package_dir}/../../../");
        Some(DockerOptions {
            root_dir: Some(root_dir.into()),
        })
    } else {
        println!("cargo:warning=Guest code is not built in docker");
        None
    };

    options.insert(
        "calculate-pow",
        GuestOptions {
            features: vec![],
            use_docker,
        },
    );

    risc0_build::embed_methods_with_options(options);
}
