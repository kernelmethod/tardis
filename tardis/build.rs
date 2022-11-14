use std::process::Command;

fn main() {
    cargo_build("../loader");
}

fn cargo_build(loader_path: &str) {
    let target_dir = format!("{}/embeds", std::env::var("OUT_DIR").unwrap());

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target-dir")
        .arg(target_dir)
        .arg("--profile")
        .arg("release");

    let output = cmd
        .current_dir(loader_path)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "Building {} failed.\nstdout: {}\nstderr: {}",
            loader_path,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}
