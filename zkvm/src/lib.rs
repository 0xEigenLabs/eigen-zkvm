#![feature(asm_experimental_arch)]
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use mktemp::Temp;
use serde_json::Value as JsonValue;
use std::fs;

macro_rules! as_ref [
    ($t:ty; $($x:expr),* $(,)?) => {
        [$(AsRef::<$t>::as_ref(&$x)),+]
    };
];

#[allow(dead_code)]
pub const TARGET_RV32: &str = "riscv32imac-unknown-none-elf";

#[allow(dead_code)]
pub const TARGET_MIPS32: &str = "mips-unknown-linux-gnu";

pub fn compile_rust_crate_to_target_asm(
    input_dir: &str,
    output_dir: &Path,
    target: &str,
) -> BTreeMap<String, String> {
    // We call cargo twice, once to get the build plan json, so we know exactly
    // which object file to use, and once to perform the actual building.

    // Real build run.
    let target_dir = output_dir.join("cargo_target");
    let build_status = build_cargo_command(input_dir, &target_dir, target, false)
        .status()
        .unwrap();
    log::debug!("build status: {:?}", build_status);
    assert!(build_status.success());

    // Build plan run. We must set the target dir to a temporary directory,
    // otherwise cargo will screw up the build done previously.
    let tmp_dir = Temp::new_dir().unwrap();
    let output = build_cargo_command(input_dir, &tmp_dir, target, true)
        .output()
        .unwrap();
    assert!(output.status.success());

    let output_files = output_files_from_cargo_build_plan(target, &output.stdout, &tmp_dir);
    drop(tmp_dir);

    // Load all the expected assembly files:
    let mut assemblies = BTreeMap::new();
    for (name, filename) in output_files {
        let filename = target_dir.join(filename);
        assert!(
            assemblies
                .insert(name, fs::read_to_string(&filename).unwrap())
                .is_none(),
            "Duplicate assembly file name: {}",
            filename.to_string_lossy()
        );
    }
    assemblies
}


fn build_cargo_command(input_dir: &str, target_dir: &Path, target: &str, produce_build_plan: bool) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.env("RUSTFLAGS", "--emit=asm -g");

    let args = as_ref![
        OsStr;
        "+nightly-2023-01-03",
        "build",
        "--release",
        "-Z",
        "build-std=core,alloc",
        "--target",
        target,
        "--lib",
        "--target-dir",
        target_dir,
        "--manifest-path",
        input_dir,
    ];

    if produce_build_plan {
        let extra_args = as_ref![
            OsStr;
            "-Z",
            "unstable-options",
            "--build-plan"
        ];
        cmd.args(itertools::chain(args.iter(), extra_args.iter()));
    } else {
        cmd.args(args.iter());
    }
    cmd
}

fn output_files_from_cargo_build_plan(
    target: &str,
    build_plan_bytes: &[u8],
    target_dir: &Path,
) -> Vec<(String, PathBuf)> {
    let json: JsonValue = serde_json::from_slice(build_plan_bytes).unwrap();

    let mut assemblies = Vec::new();

    let JsonValue::Array(invocations) = &json["invocations"] else {
        panic!("no invocations in cargo build plan");
    };

    log::debug!("{target} assembly files of this build:");
    for i in invocations {
        let JsonValue::Array(outputs) = &i["outputs"] else {
            panic!("no outputs in cargo build plan");
        };
        for output in outputs {
            let output = Path::new(output.as_str().unwrap());
            // Strip the target_dir, so that the path becomes relative.
            let parent = output.parent().unwrap().strip_prefix(target_dir).unwrap();
            log::debug!("parent: {:?}, output: {:?}", parent.to_str(), output.to_str());
            if Some(OsStr::new("rmeta")) == output.extension()
                && parent.ends_with(format!("{TARGET_MIPS32}/release/deps"))
            {
                // Have to convert to string to remove the "lib" prefix:
                let name_stem = output
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .strip_prefix("lib")
                    .unwrap();

                let mut asm_name = parent.join(name_stem);
                asm_name.set_extension("s");

                log::debug!(" - {}", asm_name.to_string_lossy());
                assemblies.push((name_stem.to_string(), asm_name));
            }
        }
    }
    assemblies
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn compile_rust_riscv() {
        env_logger::try_init().unwrap_or_default();
        let result = compile_rust_crate_to_target_asm("tests/evm/Cargo.toml", &Path::new("/tmp/abc"), TARGET_RV32);
        log::trace!("result: {:?}", result);
    }
    #[test]
    fn compile_rust_mips() {
        env_logger::try_init().unwrap_or_default();
        let result = compile_rust_crate_to_target_asm("tests/helloworld/Cargo.toml", &Path::new("/tmp/helloworld"), TARGET_MIPS32);
        log::trace!("result: {:?}", result);
    }
}
