// build script :3

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Rerun the script if the binary gets re-built
    println!("cargo:rerun-if-changed=src/main.rs");

    let _out_dir = env::var("OUT_DIR").unwrap();
    let bin = if cfg!(windows) { "crabby.exe" } else { "crabby" };

    let bin_target = PathBuf::from(format!("./target/debug/{bin}"));
    let bin_custom = PathBuf::from(format!("./bin/{bin}"));

    if let Err(e) = fs::copy(&bin_target, &bin_custom) {
        eprintln!("WARNING: failed to COPY binary to bin:/{e}");
    }
}
