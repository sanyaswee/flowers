//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.
//!
//! This script also validates the existence of `secrets/` directory and its components

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

const SECRETS_DIR: &str = "../../secrets/";
const REQUIRED_FILES: [&str; 3] = ["ip.txt", "password.txt", "ssid.txt"];

fn main() {
    // Validate firmware/secrets/
    for file in REQUIRED_FILES {
        let path = SECRETS_DIR.to_owned() + file;
        let path = path.as_str();
        println!("cargo:rerun-if-changed={}", path);

        // Check if the file exists
        let metadata = fs::metadata(path).unwrap_or_else(|_| {
            panic!(
                "Build failed: The required configuration file '{}' does not exist.",
                path
            );
        });

        // Check if the file is empty
        if metadata.len() == 0 {
            panic!(
                "Build failed: The required configuration file '{}' is empty.",
                path
            );
        }
    }

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    // Required for `defmt`
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
