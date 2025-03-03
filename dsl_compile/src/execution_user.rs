use crate::errors::{bail, DslError, Result};
use ansi_term::Colour;
use compiler::hir::very_concrete_program::VCP;
use constraint_writers::debug_writer::DebugWriter;
use constraint_writers::ConstraintExporter;
use program_structure::program_archive::ProgramArchive;

pub struct ExecutionConfig {
    pub r1cs: String,
    pub sym: String,
    pub json_constraints: String,
    pub no_rounds: usize,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_verbose: bool,
    pub inspect_constraints_flag: bool,
    pub sym_flag: bool,
    pub r1cs_flag: bool,
    pub json_substitution_flag: bool,
    pub json_constraint_flag: bool,
    pub prime: String,
}

pub fn execute_project(program_archive: ProgramArchive, config: ExecutionConfig) -> Result<VCP> {
    use constraint_generation::{build_circuit, BuildConfig};
    let debug = DebugWriter::new(config.json_constraints).unwrap();
    let build_config = BuildConfig {
        // https://github.com/iden3/circom/commit/a43f93135a9dc22bd374d29ba57722e9fe1d4646
        json_substitutions: String::new(),
        no_rounds: config.no_rounds,
        flag_json_sub: config.json_substitution_flag,
        flag_s: config.flag_s,
        flag_f: config.flag_f,
        flag_p: config.flag_p,
        flag_verbose: config.flag_verbose,
        inspect_constraints: config.inspect_constraints_flag,
        prime: config.prime,
        // https://github.com/iden3/circom/commit/8f140c1dec7975b339bfe17c1f08d8081b913560
        flag_old_heuristics: false,
    };
    match build_circuit(program_archive, build_config) {
        Ok((exporter, vcp)) => {
            if config.r1cs_flag {
                generate_output_r1cs(&config.r1cs, exporter.as_ref())?;
            }
            if config.sym_flag {
                generate_output_sym(&config.sym, exporter.as_ref())?;
            }
            if config.json_constraint_flag {
                generate_json_constraints(&debug, exporter.as_ref())?;
            }
            Result::Ok(vcp)
        }
        Err(..) => bail!(DslError::CircomCompileError("execute_project error".to_string(),)),
    }
}

fn generate_output_r1cs(file: &str, exporter: &dyn ConstraintExporter) -> Result<()> {
    if let Result::Ok(()) = exporter.r1cs(file, true) {
        log::trace!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        log::trace!("{}", Colour::Red.paint("Could not write the output in the given path"));
        bail!(DslError::CircomCompileError("generate_output_r1cs error".to_string(),))
    }
}

fn generate_output_sym(file: &str, exporter: &dyn ConstraintExporter) -> Result<()> {
    if let Result::Ok(()) = exporter.sym(file) {
        log::trace!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        log::error!("{}", Colour::Red.paint("Could not write the output in the given path"));
        bail!(DslError::CircomCompileError("generate_output_sym error".to_string(),))
    }
}

fn generate_json_constraints(debug: &DebugWriter, exporter: &dyn ConstraintExporter) -> Result<()> {
    if let Ok(()) = exporter.json_constraints(debug) {
        log::trace!(
            "{} {}",
            Colour::Green.paint("Constraints written in:"),
            debug.json_constraints
        );
        Result::Ok(())
    } else {
        log::error!("{}", Colour::Red.paint("Could not write the output in the given path"));
        bail!(DslError::CircomCompileError("generate_json_constraints error".to_string(),))
    }
}
