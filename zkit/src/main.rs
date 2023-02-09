extern crate clap;
use clap::Clap;
use plonky::api::{
    aggregation_prove, aggregation_verify, export_aggregation_verification_key,
    export_verification_key, generate_aggregation_verifier, generate_verifier, prove, setup,
    verify,
};
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
#[derive(Debug, Clap)]
pub struct SetupOpt {
    #[clap(short, required = true, default_value = "20")]
    power: u32,
    #[clap(short, required = true)]
    srs_monomial_form: String,
}

#[derive(Debug, Clap)]
pub struct CompilierOpt {
    #[clap(short = "i", required = true)]
    input: String,

    ///Set no simplification
    #[clap(long = "O0", hidden = false)]
    no_simplification: bool,

    /// prime field, like goldilocks
    #[clap(short = "p", default_value = "bn128")]
    prime: String,

    ///Set reduced simplification
    #[clap(long = "O1", hidden = false)]
    reduced_simplification: bool,

    ///Set full simplification with rounds to optimize
    #[clap(long = "O2", hidden = false, default_value = "full")]
    full_simplification: String,

    #[clap(short = "o")]
    output: String,

    #[clap(short = "l")]
    link_directories: Vec<String>,
}

/// Prove by Plonk
#[derive(Debug, Clap)]
struct ProveOpt {
    #[clap(short, required = true)]
    circuit_file: String,
    #[clap(short)]
    witness: String,
    /// SRS monomial form
    #[clap(short)]
    srs_monomial_form: String,

    #[clap(short = "l")]
    srs_lagrange_form: Option<String>,

    #[clap(short, default_value = "keccak")]
    transcript: String,

    #[clap(short = "b", default_value = "proof.bin")]
    proof_bin: String,

    #[clap(short = "j", default_value = "proof.json")]
    proof_json: String,

    #[clap(short = "p", default_value = "public.json")]
    public_json: String,
}

/// Verify the Plonk proof
#[derive(Debug, Clap)]
struct VerifyOpt {
    #[clap(short, default_value = "vk.bin")]
    vk_file: String,
    #[clap(short)]
    proof_bin: String,
    /// Transcript can be keccak or rescue, keccak default
    #[clap(short, default_value = "keccak")]
    transcript: String,
}

/// Generate solidity verifier
#[derive(Debug, Clap)]
struct GenerateVerifierOpt {
    #[clap(short, default_value = "vk.bin")]
    vk_file: String,
    #[clap(short = "s", long = "sol", default_value = "verifier.sol")]
    sol: String,
}

/// Export proof's verification key
#[derive(Debug, Clap)]
struct ExportVerificationKeyOpt {
    #[clap(short)]
    srs_monomial_form: String,
    #[clap(short)]
    circuit_file: String,
    #[clap(short = "v", default_value = "vk.bin")]
    output_vk: String,
}

/// Export aggregation proof's verification key
#[derive(Debug, Clap)]
struct ExportAggregationVerificationKeyOpt {
    #[clap(short = "c")]
    num_proofs_to_check: usize,
    #[clap(short = "i")]
    num_inputs: usize,
    #[clap(short)]
    srs_monomial_form: String,
    #[clap(short = "v", long = "vk", default_value = "aggregation_vk.bin")]
    output_vk: String,
}

/// Proof aggregation for plonk
#[derive(Debug, Clap)]
struct AggregationProveOpt {
    /// SRS monomial form
    #[clap(short)]
    srs_monomial_form: String,

    #[clap(short = "f")]
    old_proof_list: String,

    #[clap(short = "v", default_value = "vk.bin")]
    old_vk: String,

    #[clap(short = "n", default_value = "aggregation_proof.bin")]
    new_proof: String,

    #[clap(short = "j", default_value = "proof.json")]
    proof_json: String,
}

/// Verify aggregation proof
#[derive(Debug, Clap)]
struct AggregationVerifyOpt {
    #[clap(short = "p", default_value = "aggregation_proof.bin")]
    proof: String,
    #[clap(short = "v", default_value = "aggregation_vk.bin")]
    vk: String,
}

/// A subcommand for generating a Solidity aggregation verifier smart contract
#[derive(Debug, Clap)]
struct GenerateAggregationVerifierOpt {
    /// Original individual verification key file
    #[clap(short = "o", long = "old_vk", default_value = "vk.bin")]
    old_vk: String,
    /// Aggregated verification key file
    #[clap(short = "n", long = "new_vk", default_value = "aggregation_vk.bin")]
    new_vk: String,
    /// Num of inputs
    #[clap(short = "i", long = "num_inputs")]
    num_inputs: usize,
    /// Output solidity file
    #[clap(short = "s", long = "sol", default_value = "verifier.sol")]
    sol: String,
}

/// Stark proving and verifying all in one
#[derive(Debug, Clap)]
struct StarkProveOpt {
    #[clap(short = "s", long = "stark_stuct", default_value = "stark_struct.json")]
    stark_struct: String,
    #[clap(short = "p", long = "piljson", default_value = "pil.json")]
    piljson: String,
    #[clap(short = "o", long = "const_pols", default_value = "pols.const")]
    const_pols: String,
    #[clap(short = "m", long = "cm_pols", default_value = "pols.cm")]
    cm_pols: String,
    #[clap(short = "c", long = "circom", default_value = "stark_verfier.circom")]
    circom_file: String,
    #[clap(short = "i", long = "zkin", default_value = "zkin.json")]
    zkin: String,
}

#[derive(Debug, Clap)]
enum Command {
    #[clap(name = "setup")]
    Setup(SetupOpt),
    /// Compile circom circuits to r1cs, and generate witness
    #[clap(name = "compile")]
    Compile(CompilierOpt),
    #[clap(name = "prove")]
    Prove(ProveOpt),
    #[clap(name = "verify")]
    Verify(VerifyOpt),
    #[clap(name = "export_verification_key")]
    ExportVerificationKey(ExportVerificationKeyOpt),
    #[clap(name = "generate_verifier")]
    GenerateVerifier(GenerateVerifierOpt),
    #[clap(name = "export_aggregation_verification_key")]
    ExportAggregationVerificationKey(ExportAggregationVerificationKeyOpt),
    #[clap(name = "aggregation_prove")]
    AggregationProve(AggregationProveOpt),
    #[clap(name = "aggregation_verify")]
    AggregationVerify(AggregationVerifyOpt),
    #[clap(name = "generate_aggregation_verifier")]
    GenerateAggregationVerifier(GenerateAggregationVerifierOpt),

    #[clap(name = "stark_prove")]
    StarkProve(StarkProveOpt),
}

#[derive(Debug, Clap)]
#[clap(version = "0.1.6")]
struct Cli {
    #[clap(subcommand)]
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
        Command::Compile(args) => compile(args).map_err(|_| anyhow::anyhow!("compile error")),
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

        Command::StarkProve(args) => stark::prove(
            &args.stark_struct,
            &args.piljson,
            &args.const_pols,
            &args.cm_pols,
            &args.circom_file,
            &args.zkin,
        ),
    };
    match exec_result {
        Err(x) => println!("execute error: {}", x),
        _ => println!("time cost: {}", start.elapsed().as_secs_f64()),
    };
}
