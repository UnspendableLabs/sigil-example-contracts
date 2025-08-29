use std::path::Path;
use std::process::Command;

fn main() {
    let contract_dir = "../contract"; // Path to contract crate
    let built_wasm = "../target/wasm32-unknown-unknown/release";
    let dest_wasm = Path::new(&built_wasm).join("contract.wasm");
    let dest_opt_wasm = Path::new(&built_wasm).join("contract_opt.wasm");
    let dest_brotli = Path::new(&built_wasm).join("contract.wasm.br");

    println!("cargo:rerun-if-changed={}", contract_dir);

    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
            &format!("{}/Cargo.toml", contract_dir),
            "--release",
        ])
        .status()
        .expect("Failed to execute cargo");

    if !status.success() {
        panic!("Failed to build contract to WASM");
    }

    if !Command::new("wasm-opt")
        .args([
            "-Oz",
            "--enable-bulk-memory",
            "--enable-sign-ext",
            &dest_wasm.display().to_string(),
            "-o",
            &dest_opt_wasm.display().to_string(),
        ])
        .status()
        .expect(
            "Failed to execute wasm-opt; ensure it’s installed (e.g., 'cargo install wasm-opt')",
        )
        .success()
    {
        panic!("wasm-opt failed for {}", dest_wasm.display());
    }

    if !Command::new("brotli")
        .args(["-Zf", &dest_opt_wasm.display().to_string(), "-o", &dest_brotli.display().to_string()])
        .status()
        .expect("Failed to execute brotli; ensure it’s installed (e.g., 'brew install brotli' on macOS or 'apt install brotli' on Ubuntu)")
        .success()
    {
        panic!("brotli failed for {}", dest_opt_wasm.display());
    }

    println!(
        "cargo:rustc-env=CONTRACT_WASM_PATH=../{}",
        dest_brotli.display()
    );
}
