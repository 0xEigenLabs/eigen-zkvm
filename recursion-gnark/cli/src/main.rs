//! A simple CLI that wraps the gnark-ffi crate. This is called using Docker in gnark-ffi when the
//! native feature is disabled.

use recursion_gnark_ffi::ffi::test;

use clap::{Args, Parser, Subcommand};
use std::fs::File;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Test(TestArgs),
}

#[derive(Debug, Args)]
struct TestArgs {
    data_dir: String,
    proof_path: String,
    #[arg(short, long)]
    system: String,
}

fn run_test(args: TestArgs) {
    let mut file = File::open(&args.proof_path).unwrap();
    let proof: recursion_gnark_ffi::Groth16Bn254Proof =
        bincode::deserialize_from(&mut file).expect("Failed to deserialize proof");

    match args.system.as_str() {
        "plonk" => panic!("Unsupported system: {} or mismatched proof type", args.system),
        "groth16" => test(&args.data_dir, &proof.raw_proof),
        _ => panic!("Unsupported system: {} or mismatched proof type", args.system),
    };
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Test(args) => run_test(args),
    }
}
