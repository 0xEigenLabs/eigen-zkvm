//! A simple CLI that wraps the gnark-ffi crate. This is called using Docker in gnark-ffi when the
//! native feature is disabled.

use recursion_gnark_ffi::{ffi::test, ProofBn254};

use clap::{Args, Parser, Subcommand};
use std::{fs::File, io::Write};

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

    let result = match (args.system.as_str()) {
        "plonk" => panic!("Unsupported system: {} or mismatched proof type", args.system),
        "groth16" => test(&args.data_dir, &proof.raw_proof),
        _ => panic!("Unsupported system: {} or mismatched proof type", args.system),
    };
    // let output = match result {
    //     Ok(_) => "OK".to_string(),
    //     Err(e) => e,
    // };
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Test(args) => run_test(args),
    }
}
