use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(embed_static)");
    if std::env::var("CARGO_FEATURE_EMBED_STATIC").is_ok() {
        println!("cargo:rustc-cfg=embed_static");
        build_frontend();
    }
}

fn build_frontend() {
    let web_dir = Path::new("../../web");
    if !web_dir.exists() {
        println!("cargo:warning=web/ directory not found, skipping frontend build");
        return;
    }

    let node_modules = web_dir.join("node_modules");
    if !node_modules.exists() {
        let status = Command::new("bun")
            .arg("install")
            .current_dir(web_dir)
            .status()
            .expect("bun install failed");

        if !status.success() {
            panic!("bun install failed with exit code {:?}", status.code());
        }
    }

    let status = Command::new("bun")
        .arg("run")
        .arg("build")
        .current_dir(web_dir)
        .status()
        .expect("bun build failed");

    if !status.success() {
        panic!("bun build failed with exit code {:?}", status.code());
    }

    println!("cargo:rerun-if-changed=../../web/package.json");
    println!("cargo:rerun-if-changed=../../web/index.html");
    println!("cargo:rerun-if-changed=../../web/build.ts");
    println!("cargo:rerun-if-changed=../../web/dev.ts");
    println!("cargo:rerun-if-changed=../../web/tsconfig.json");
    println!("cargo:rerun-if-changed=../../web/app/");
    println!("cargo:rerun-if-changed=../../web/src/");
}
