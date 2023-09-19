extern crate clap;
use clap::{command, Parser};
use plonky::api::{
    aggregation_check, aggregation_prove, aggregation_verify, analyse, calculate_witness,
    export_aggregation_verification_key, export_verification_key, generate_aggregation_verifier,
    generate_verifier, prove, setup, verify,
};
use plonky::errors::EigenError;
use std::time::Instant;

mod compilation_user;
mod execution_user;
mod input_user;
mod parser_user;
mod stark;
mod type_analysis_user;

/// Align with https://github.com/iden3/circom/blob/master/circom/Cargo.toml#L3
const VERSION: &'static str = "2.1.2";

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
    #[arg(short, default_value = "bn128")]
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

/// Prove by Plonk
#[derive(Debug, Parser)]
struct ProveOpt {
    #[arg(short, required = true)]
    circuit_file: String,
    #[arg(short)]
    witness: String,
    /// SRS monomial form
    #[arg(short)]
    srs_monomial_form: String,

    #[arg(long = "l")]
    srs_lagrange_form: Option<String>,

    #[arg(short, default_value = "keccak")]
    transcript: String,

    #[arg(long = "b", default_value = "proof.bin")]
    proof_bin: String,

    #[arg(long = "j", default_value = "proof.json")]
    proof_json: String,

    #[arg(long = "p", default_value = "public.json")]
    public_json: String,
}

/// Verify the Plonk proof
#[derive(Debug, Parser)]
struct VerifyOpt {
    #[arg(short, default_value = "vk.bin")]
    vk_file: String,
    #[arg(short)]
    proof_bin: String,
    /// Transcript can be keccak or rescue, keccak default
    #[arg(short, default_value = "keccak")]
    transcript: String,
}

/// Generate solidity verifier
#[derive(Debug, Parser)]
struct GenerateVerifierOpt {
    #[arg(short, default_value = "vk.bin")]
    vk_file: String,
    #[arg(long = "s", default_value = "verifier.sol")]
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

/// Analyse circuits
#[derive(Parser, Debug)]

struct AnalyseOpt {
    #[arg(short)]
    circuit_file: String,
    #[arg(short, default_value = "analyse.json")]
    output: String,
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

/// Proof aggregation for plonk
#[derive(Parser, Debug)]

struct AggregationProveOpt {
    /// SRS monomial form
    #[arg(short)]
    srs_monomial_form: String,

    #[arg(long = "f")]
    old_proof_list: String,

    #[arg(long = "v", default_value = "vk.bin")]
    old_vk: String,

    #[arg(long = "n", default_value = "aggregation_proof.bin")]
    new_proof: String,

    #[arg(long = "j", default_value = "proof.json")]
    proof_json: String,
}

/// Verify aggregation proof
#[derive(Parser, Debug)]

struct AggregationVerifyOpt {
    #[arg(long = "p", default_value = "aggregation_proof.bin")]
    proof: String,
    #[arg(long = "v", default_value = "aggregation_vk.bin")]
    vk: String,
}

/// A subcommand for generating a Solidity aggregation verifier smart contract
#[derive(Parser, Debug)]

struct GenerateAggregationVerifierOpt {
    /// Original individual verification key file
    #[arg(short, long = "old_vk", default_value = "vk.bin")]
    old_vk: String,
    /// Aggregated verification key file
    #[arg(long = "n", default_value = "aggregation_vk.bin")]
    new_vk: String,
    /// Num of inputs
    #[arg(long = "i", long = "num_inputs")]
    num_inputs: usize,
    /// Output solidity file
    #[arg(short, long = "sol", default_value = "verifier.sol")]
    sol: String,
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

// .usage("node main_compressor12_setup.js -r <verifier.c12.r1cs> -p <verifier.c12.pil> -c <verifier.c12.const> -e <verifier.c12.exec> [--forceNBits=23]")
#[derive(Parser, Debug)]
struct Compresor12SetupOpt {
    #[arg(long = "r", default_value = "mycircuit.verifier.r1cs")]
    r1cs_file: String,
    #[arg(long = "c", default_value = "mycircuit.c12.const")]
    const_file: String, // Output file required to build the constants
    #[arg(long = "p", default_value = "mycircuit.c12.pil")]
    pil_file: String, // Proposed PIL
    #[arg(long = "e", default_value = "mycircuit.c12.exec")]
    exec_file: String, // File required to execute
    #[arg(short, default_value = "23")]
    force_n_bits: usize,
}

// generate the pil files and  const polynomicals files
// .usage("node main_compressor12_exec.js -r <r1cs.circom> -p <pil.json> [-P <pilconfig.json] -v <verification_key.json>")
#[derive(Parser, Debug)]
struct Compresor12ExecOpt {
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

#[derive(Parser, Debug)]
enum Command {
    #[command(name = "setup")]
    Setup(SetupOpt),
    #[command(name = "calculate_witness")]
    CalculateWitness(CalculateWitnessOpt),
    /// Compile circom circuits to r1cs, and generate witness
    #[command(name = "compile")]
    Compile(CompilierOpt),
    #[command(name = "prove")]
    Prove(ProveOpt),
    #[command(name = "verify")]
    Verify(VerifyOpt),
    #[command(name = "export_verification_key")]
    ExportVerificationKey(ExportVerificationKeyOpt),
    #[command(name = "generate_verifier")]
    GenerateVerifier(GenerateVerifierOpt),
    #[command(name = "export_aggregation_verification_key")]
    ExportAggregationVerificationKey(ExportAggregationVerificationKeyOpt),
    #[command(name = "aggregation_prove")]
    AggregationProve(AggregationProveOpt),
    #[command(name = "aggregation_verify")]
    AggregationVerify(AggregationVerifyOpt),
    #[command(name = "generate_aggregation_verifier")]
    GenerateAggregationVerifier(GenerateAggregationVerifierOpt),
    #[command(name = "aggregation_check")]
    AggregationCheck(AggregationCheckOpt),

    #[command(name = "stark_prove")]
    StarkProve(StarkProveOpt),

    #[command(name = "analyse")]
    Analyse(AnalyseOpt),

    // todo opti command name.
    #[command(name = "compressor12_setup")]
    Compresor12Setup(Compresor12SetupOpt),
    #[command(name = "compressor12_exec")]
    Compresor12Exec(Compresor12ExecOpt),
}

#[derive(Parser, Debug)]
#[command(author, version = "0.1.6", about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub fn compile(opt: CompilierOpt) -> Result<(), ()> {
    use compilation_user::CompilerConfig;
    use execution_user::ExecutionConfig;
    let fullopt = opt.full_simplification.len() > 0;
    let o2_arg = opt.full_simplification.as_str();
    let o_style = input_user::get_simplification_style(
        opt.no_simplification,
        opt.reduced_simplification,
        fullopt,
        &o2_arg,
    )?;
    let input = std::path::PathBuf::from(opt.input);
    let output = std::path::PathBuf::from(opt.output);

    let user_input =
        input_user::Input::new(input, output, o_style, opt.prime, opt.link_directories)?;
    let mut program_archive = parser_user::parse_project(&user_input)?;

    type_analysis_user::analyse_project(&mut program_archive)?;

    let config = ExecutionConfig {
        no_rounds: user_input.no_rounds(),
        flag_p: user_input.parallel_simplification_flag(),
        flag_s: user_input.reduced_simplification_flag(),
        flag_f: user_input.unsimplified_flag(),
        flag_verbose: user_input.flag_verbose(),
        inspect_constraints_flag: user_input.inspect_constraints_flag(),
        r1cs_flag: user_input.r1cs_flag(),
        json_constraint_flag: user_input.json_constraints_flag(),
        json_substitution_flag: user_input.json_substitutions_flag(),
        sym_flag: user_input.sym_flag(),
        sym: user_input.sym_file().to_string(),
        r1cs: user_input.r1cs_file().to_string(),
        json_constraints: user_input.json_constraints_file().to_string(),
        prime: user_input.get_prime(),
    };
    let circuit = execution_user::execute_project(program_archive, config)?;
    let compilation_config = CompilerConfig {
        vcp: circuit,
        debug_output: user_input.print_ir_flag(),
        c_flag: user_input.c_flag(),
        wasm_flag: user_input.wasm_flag(),
        wat_flag: user_input.wat_flag(),
        js_folder: user_input.js_folder().to_string(),
        wasm_name: user_input.wasm_name().to_string(),
        c_folder: user_input.c_folder().to_string(),
        c_run_name: user_input.c_run_name().to_string(),
        c_file: user_input.c_file().to_string(),
        dat_file: user_input.dat_file().to_string(),
        wat_file: user_input.wat_file().to_string(),
        wasm_file: user_input.wasm_file().to_string(),
        produce_input_log: user_input.main_inputs_flag(),
    };
    compilation_user::compile(compilation_config)?;
    Result::Ok(())
}

fn main() {
    let args = Cli::parse();
    env_logger::init();
    let start = Instant::now();
    let exec_result = match args.command {
        Command::Setup(args) => setup(args.power, &args.srs_monomial_form),
        Command::Compile(args) => {
            compile(args).map_err(|_| EigenError::from("compile error".to_string()))
        }
        Command::CalculateWitness(args) => {
            calculate_witness(&args.wasm_file, &args.input_json, &args.output)
        }
        Command::Prove(args) => prove(
            &args.circuit_file,
            &args.witness,
            &args.srs_monomial_form,
            args.srs_lagrange_form,
            &args.transcript,
            &args.proof_bin,
            &args.proof_json,
            &args.public_json,
        ),
        Command::Verify(args) => verify(&args.vk_file, &args.proof_bin, &args.transcript),
        Command::GenerateVerifier(args) => generate_verifier(&args.vk_file, &args.sol),
        Command::ExportVerificationKey(args) => {
            export_verification_key(&args.srs_monomial_form, &args.circuit_file, &args.output_vk)
        }

        Command::ExportAggregationVerificationKey(args) => export_aggregation_verification_key(
            args.num_proofs_to_check,
            args.num_inputs,
            &args.srs_monomial_form,
            &args.output_vk,
        ),
        Command::AggregationProve(args) => aggregation_prove(
            &args.srs_monomial_form,
            &args.old_proof_list,
            &args.old_vk,
            &args.new_proof,
            &args.proof_json,
        ),
        Command::AggregationVerify(args) => aggregation_verify(&args.proof, &args.vk),
        Command::GenerateAggregationVerifier(args) => {
            generate_aggregation_verifier(&args.old_vk, &args.new_vk, args.num_inputs, &args.sol)
        }
        Command::AggregationCheck(args) => {
            aggregation_check(&args.old_proof_list, &args.old_vk, &args.new_proof)
        }
        Command::StarkProve(args) => stark::prove(
            &args.stark_struct,
            &args.piljson,
            args.norm_stage,
            args.agg_stage,
            &args.const_pols,
            &args.cm_pols,
            &args.circom_file,
            &args.zkin,
        )
        .map_err(|e| EigenError::from(format!("stark prove error {:?}", e))),

        Command::Analyse(args) => analyse(&args.circuit_file, &args.output),
        Command::Compresor12Setup(args) => starky::compressor12_setup::setup(
            &args.r1cs_file,
            &args.pil_file,
            &args.const_file,
            &args.exec_file,
            args.force_n_bits,
        )
        .map_err(|_| EigenError::from("compreesor12 setup error".to_string())),
        Command::Compresor12Exec(args) => starky::compressor12_exec::exec(
            &args.input_file,
            &args.wasm_file,
            &args.pil_file,
            &args.exec_file,
            &args.commit_file,
        )
        .map_err(|_| EigenError::from("compreesor12 exec error".to_string())),
    };
    match exec_result {
        Err(x) => {
            println!("execute error: {}", x);
            std::process::exit(400)
        }
        _ => println!("time cost: {}", start.elapsed().as_secs_f64()),
    };
}
