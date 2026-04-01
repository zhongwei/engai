fn main() {
    if std::env::var("FRONTEND_BUILD").is_ok() || cfg!(not(debug_assertions)) {
        let web_dir = std::path::Path::new("../../web");
        if web_dir.exists() {
            let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
            let status = std::process::Command::new(npm)
                .args(["install"])
                .current_dir(web_dir)
                .status()
                .expect("Failed to run npm install");
            assert!(status.success());
            let status = std::process::Command::new(npm)
                .args(["run", "build"])
                .current_dir(web_dir)
                .status()
                .expect("Failed to run npm build");
            assert!(status.success());
        }
    }
    println!("cargo:rerun-if-changed=static/");
}
