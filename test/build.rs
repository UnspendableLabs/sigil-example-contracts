use std::{fs, path::Path, process::Command};

fn find_first_file_with_extension(dir: &Path, extension: &str) -> Option<String> {
    let ext = extension.trim_start_matches('.').to_lowercase();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file()
            && let Some(file_ext) = path.extension()
            && file_ext.to_string_lossy().to_lowercase() == ext
        {
            return path.file_stem().map(|s| s.to_string_lossy().into_owned());
        }
    }

    None
}

fn main() {
    let mut cd = std::env::current_dir().unwrap();
    cd.pop();
    let contract_dir = cd.join("contract");
    let built_wasm = cd.join("target/wasm32-unknown-unknown/release");

    println!("cargo:rerun-if-changed={}", contract_dir.display());

    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
            &format!("{}/Cargo.toml", contract_dir.display()),
            "--release",
        ])
        .status()
        .expect("Failed to execute cargo");

    if !status.success() {
        panic!("Failed to build contract to WASM");
    }

    let file_stem = find_first_file_with_extension(&built_wasm, ".wasm").unwrap();
    let dest_wasm = built_wasm.join(format!("{}.wasm", file_stem));
    let dest_opt_wasm = built_wasm.join(format!("{}_opt.wasm", file_stem));
    let dest_brotli = built_wasm.join(format!("{}.wasm.br", file_stem));

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
        "cargo:rustc-env=CONTRACT_WASM_PATH={}",
        dest_brotli.display()
    );
}
