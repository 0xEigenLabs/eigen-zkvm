extern crate clap;
use clap::{command, Parser};
use dsl_compile::circom_compiler;
use groth16::api::*;
use starky::prove::stark_prove;
use std::time::Instant;

/// Trust setup for Plonk
#[derive(Parser, Debug)]
pub struct SetupOpt {
    #[arg(short, required = true, default_value = "20")]
    power: u32,
    #[arg(short, required = true)]
    srs_monomial_form: String,
}

#[derive(Debug, Parser)]
pub struct CompilierOpt {
    #[arg(short, required = true)]
    input: String,

    ///Set no simplification
    #[arg(long = "O0", hide = false)]
    no_simplification: bool,

    /// prime field, like goldilocks
    #[arg(short, default_value = "BN128")]
    prime: String,

    ///Set reduced simplification
    #[arg(long = "O1", hide = false)]
    reduced_simplification: bool,

    ///Set full simplification with rounds to optimize
    #[arg(long = "O2", hide = false, default_value = "full")]
    full_simplification: String,

    /// setup output path
    #[arg(short)]
    output: String,

    /// setup the library path
    #[arg(short)]
    link_directories: Vec<String>,
}

/// Calculate witness and save to output file
#[derive(Debug, Parser)]

struct CalculateWitnessOpt {
    /// wasm circuit
    #[arg(short, required = true)]
    wasm_file: String,
    /// [input] input json
    #[arg(short, required = true)]
    input_json: String,
    /// [output] witness filename
    #[arg(short, default_value = "witness.wtns")]
    output: String,
}

/// Generate solidity verifier
#[derive(Debug, Parser)]
struct GenerateVerifierOpt {
    #[arg(short, default_value = "vk.bin")]
    vk_file: String,
    #[arg(short, default_value = "groth16")]
    protocal: String,
    #[arg(short, default_value = "verifier.sol")]
    sol: String,
}

/// Export proof's verification key
#[derive(Debug, Parser)]
struct ExportVerificationKeyOpt {
    #[arg(short)]
    srs_monomial_form: String,
    #[arg(short)]
    circuit_file: String,
    #[arg(long = "v", default_value = "vk.bin")]
    output_vk: String,
}

/// Export aggregation proof's verification key
#[derive(Parser, Debug)]
struct ExportAggregationVerificationKeyOpt {
    #[arg(long = "c")]
    num_proofs_to_check: usize,
    #[arg(long = "i")]
    num_inputs: usize,
    #[arg(short)]
    srs_monomial_form: String,
    #[arg(long = "v", default_value = "aggregation_vk.bin")]
    output_vk: String,
}

/// Stark proving and verifying all in one
#[derive(Parser, Debug)]
struct StarkProveOpt {
    #[arg(short, long = "stark_stuct", default_value = "stark_struct.json")]
    stark_struct: String,
    #[arg(short, long = "piljson", default_value = "pil.json")]
    piljson: String,
    #[arg(short, long = "norm_stage", action= clap::ArgAction::SetTrue)]
    norm_stage: bool,
    #[arg(short, long = "skip_main", action= clap::ArgAction::SetTrue)]
    skip_main: bool,
    #[arg(short, long = "agg_stage", action= clap::ArgAction::SetTrue)]
    agg_stage: bool,
    #[arg(long = "o", default_value = "pols.const")]
    const_pols: String,
    #[arg(long = "m", default_value = "pols.cm")]
    cm_pols: String,
    #[arg(short, long = "circom", default_value = "stark_verfier.circom")]
    circom_file: String,
    #[arg(long = "i", default_value = "zkin.json")]
    zkin: String,
    #[arg(
        long = "prover_addr",
        default_value = "273030697313060285579891744179749754319274977764"
    )]
    prover_addr: String,
}

/// Check aggregation proof
#[derive(Parser, Debug)]
struct AggregationCheckOpt {
    #[arg(long = "f")]
    old_proof_list: String,

    #[arg(long = "v", default_value = "vk.bin")]
    old_vk: String,

    #[arg(short, default_value = "aggregation_proof.bin")]
    new_proof: String,
}

/// Setup compressor12 for converting R1CS to PIL
#[derive(Parser, Debug)]
struct Compressor12SetupOpt {
    #[arg(long = "r", default_value = "mycircuit.verifier.r1cs")]
    r1cs_file: String,
    #[arg(long = "c", default_value = "mycircuit.c12.const")]
    const_file: String, // Output file required to build the constants
    #[arg(long = "p", default_value = "mycircuit.c12.pil")]
    pil_file: String, // Proposed PIL
    #[arg(long = "e", default_value = "mycircuit.c12.exec")]
    exec_file: String, // File required to execute
    #[arg(long, default_value = "0")]
    force_n_bits: usize,
}

/// Exec compressor12 for converting R1CS to PIL
#[derive(Parser, Debug)]
struct Compressor12ExecOpt {
    // input files :  $C12_VERIFIER.r1cs  $C12_VERIFIER.const  $C12_VERIFIER.pil
    #[arg(long = "i", default_value = "mycircuit.proof.zkin.json")]
    input_file: String,
    #[arg(long = "w", default_value = "mycircuit.verifier.wasm")]
    wasm_file: String,
    #[arg(long = "p", default_value = "mycircuit.c12.pil")]
    pil_file: String,
    // output files :  $C12_VERIFIER.exec
    #[arg(long = "e", default_value = "mycircuit.c12.exec")]
    exec_file: String,
    #[arg(long = "m", default_value = "mycircuit.c12.cm")]
    commit_file: String,
}

/// generate the input1.zkin.json and input2.zkin.json into out.zkin.json
#[derive(Parser, Debug)]
struct JoinZkinExecOpt {
    // #[arg(long = "starksetup", default_value = "starksetup.json")]
    // starksetup: String,
    #[arg(long = "zkin1", default_value = "input1.zkin.json")]
    zkin1: String,
    #[arg(long = "zkin2", default_value = "input2.zkin.json")]
    zkin2: String,
    #[arg(long = "zkinout", default_value = "out.zkin.json")]
    zkinout: String,
}

/// Setup groth16
#[derive(Parser, Debug)]
pub struct Groth16SetupOpt {
    #[arg(short, required = true, default_value = "BN128")]
    curve_type: String,
    #[arg(long = "r1cs", required = true)]
    circuit_file: String,
    #[arg(short, required = true, default_value = "g16.zkey")]
    pk_file: String,
    #[arg(short, required = true, default_value = "verification_key.json")]
    vk_file: String,
    #[arg(short, action= clap::ArgAction::SetTrue)]
    to_hex: bool,
}

/// Prove with groth16
#[derive(Parser, Debug)]
pub struct Groth16ProveOpt {
    #[arg(short, required = true, default_value = "BN128")]
    curve_type: String,
    #[arg(long = "r1cs", required = true)]
    circuit_file: String,
    #[arg(short, required = true)]
    wasm_file: String,
    #[arg(short, required = true, default_value = "g16.zkey")]
    pk_file: String,
    #[arg(short, required = true)]
    input_file: String,
    #[arg(long = "public-input", required = true, default_value = "public_input.json")]
    public_input_file: String,
    #[arg(long = "proof", required = true, default_value = "proof.json")]
    proof_file: String,
    #[arg(short, action= clap::ArgAction::SetTrue)]
    to_hex: bool,
}

/// Verify with groth16
#[derive(Parser, Debug)]
pub struct Groth16VerifyOpt {
    #[arg(short, required = true, default_value = "BN128")]
    curve_type: String,
    #[arg(short, required = true, default_value = "verification_key.json")]
    vk_file: String,
    #[arg(long = "public-input", required = true, default_value = "public_input.json")]
    public_input_file: String,
    #[arg(long = "proof", required = true, default_value = "proof.json")]
    proof_file: String,
}

#[derive(Parser, Debug)]
enum Command {
    /// Compile circom circuits to r1cs, and generate witness
    #[command(name = "compile")]
    Compile(CompilierOpt),
    #[command(name = "generate_verifier")]
    GenerateVerifier(GenerateVerifierOpt),

    #[command(name = "stark_prove")]
    StarkProve(StarkProveOpt),
    #[command(name = "compressor12_setup")]
    Compressor12Setup(Compressor12SetupOpt),
    #[command(name = "compressor12_exec")]
    Compressor12Exec(Compressor12ExecOpt),
    #[command(name = "join_zkin")]
    JoinZkin(JoinZkinExecOpt),

    #[command(name = "groth16_setup")]
    Groth16Setup(Groth16SetupOpt),
    #[command(name = "groth16_prove")]
    Groth16Prove(Groth16ProveOpt),
    #[command(name = "groth16_verify")]
    Groth16Verify(Groth16VerifyOpt),
}

#[derive(Parser, Debug)]
#[command(author, version = "0.1.6", about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let args = Cli::parse();
    env_logger::init();
    let start = Instant::now();
    let exec_result = match args.command {
        Command::Compile(args) => circom_compiler(
            args.input,
            args.prime.to_lowercase(),
            args.full_simplification,
            args.link_directories,
            args.output,
            args.no_simplification,
            args.reduced_simplification,
        ),
        Command::GenerateVerifier(args) => match args.protocal.as_str() {
            "groth16" => groth16::api::generate_verifier(&args.vk_file, &args.sol),
            _ => {
                panic!("unknown protocol")
            }
        },

        Command::StarkProve(args) => stark_prove(
            &args.stark_struct,
            &args.piljson,
            args.norm_stage,
            args.skip_main,
            args.agg_stage,
            &args.const_pols,
            &args.cm_pols,
            &args.circom_file,
            &args.zkin,
            &args.prover_addr,
        ),
        Command::Compressor12Setup(args) => recursion::compressor12_setup::setup(
            &args.r1cs_file,
            &args.pil_file,
            &args.const_file,
            &args.exec_file,
            args.force_n_bits,
        ),
        Command::Compressor12Exec(args) => recursion::compressor12_exec::exec(
            &args.input_file,
            &args.wasm_file,
            &args.pil_file,
            &args.exec_file,
            &args.commit_file,
        ),
        Command::JoinZkin(args) => {
            starky::zkin_join::join_zkin(&args.zkin1, &args.zkin2, &args.zkinout)
        }
        Command::Groth16Setup(args) => groth16_setup(
            &args.curve_type,
            &args.circuit_file,
            &args.pk_file,
            &args.vk_file,
            args.to_hex,
        ),
        Command::Groth16Prove(args) => groth16_prove(
            &args.curve_type,
            &args.circuit_file,
            &args.wasm_file,
            &args.pk_file,
            &args.input_file,
            &args.public_input_file,
            &args.proof_file,
            args.to_hex,
        ),
        Command::Groth16Verify(args) => groth16_verify(
            &args.curve_type,
            &args.vk_file,
            &args.public_input_file,
            &args.proof_file,
        ),
    };
    match exec_result {
        Err(x) => {
            println!("execute error: {}", x);
            std::process::exit(400)
        }
        _ => println!("time cost: {}", start.elapsed().as_secs_f64()),
    };
}
