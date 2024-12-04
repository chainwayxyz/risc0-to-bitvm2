use std::process::Command;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../../build.dockerfile");

    if env::var("REPR_GUEST_BUILD").is_ok() {
        // Get the absolute path to the project root
        let current_dir = env::current_dir().expect("Failed to get current directory");
        let project_root = current_dir.parent().unwrap().parent().unwrap();
        let output_dir = project_root.join("target/riscv-guest/riscv32im-risc0-zkvm-elf/docker");

        // Ensure the output directory exists
        std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        let output = Command::new("docker")
            .args([
                "buildx", "build",
                "--platform", "linux/amd64",
                "-f", "build.dockerfile",
                "--output", &format!("type=local,dest={}", output_dir.display()),
                ".", // Use current directory as context
            ])
            .current_dir(project_root) // Set working directory to project root
            .output()
            .expect("Failed to execute Docker command");

        if !output.status.success() {
            eprintln!("Docker build failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Docker build failed");
        }
    }

    risc0_build::embed_methods();
}