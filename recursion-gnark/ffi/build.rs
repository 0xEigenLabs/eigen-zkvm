#![allow(unused)]

use cfg_if::cfg_if;
use std::{env, path::PathBuf, process::Command};

#[allow(deprecated)]
use bindgen::CargoCallbacks;

/// Build the go library, generate Rust bindings for the exposed functions, and link the library.
fn main() {
    println!("cargo:rerun-if-changed=go");
    // Define the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("OUT_DIR is: {}", out_dir);
    let dest_path = PathBuf::from(&out_dir);
    let lib_name = "eigengnark";
    let dest = dest_path.join(format!("lib{}.a", lib_name));

    println!("Building Go library at {}", dest.display());

    // Run the go build command
    let status = Command::new("go")
        .current_dir("go")
        .env("CGO_ENABLED", "1")
        .args(["build", "-tags=debug", "-o", dest.to_str().unwrap(), "-buildmode=c-archive", "."])
        .status()
        .expect("Failed to build Go library");
    if !status.success() {
        panic!("Go build failed");
    }

    // Generate bindings using bindgen
    let header_path = dest_path.join(format!("lib{}.h", lib_name));
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(dest_path.join("bindings.rs")).expect("Couldn't write bindings!");

    println!("Go library built");

    // Link the Go library
    println!("cargo:rustc-link-search=native={}", dest_path.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);

    // Static linking doesn't really work on macos, so we need to link some system libs
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}
