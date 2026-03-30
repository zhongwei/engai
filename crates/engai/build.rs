use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../frontend/dist");
    println!("cargo:rerun-if-changed=../../frontend/package.json");
    println!("cargo:rerun-if-changed=../../frontend/src");
    let frontend_dist = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../frontend/dist");
    if !frontend_dist.exists() {
        let frontend_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../frontend");
        println!("cargo:warning=frontend/dist not found, running npm build...");
        let output = Command::new("npm")
            .args(["run", "build"])
            .current_dir(&frontend_dir)
            .output();
        match output {
            Ok(o) if o.status.success() => println!("cargo:warning=frontend build succeeded"),
            Ok(o) => println!(
                "cargo:warning=frontend build failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => println!("cargo:warning=failed to run npm build: {}", e),
        }
    }
}
