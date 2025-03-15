use crate::errors::{bail, DslError, Result};
use crate::input_user::Input;
use crate::CIRCOM_VERSION;
use program_structure::error_definition::Report;
use program_structure::program_archive::ProgramArchive;

pub fn parse_project(input_info: &Input) -> Result<ProgramArchive> {
    let initial_file = input_info.input_file().to_string();
    let result_program_archive =
        parser::run_parser(initial_file, CIRCOM_VERSION, input_info.link_libraries.clone());
    match result_program_archive {
        Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            bail!(DslError::CircomCompileError("parser::run_parser error".to_string(),))
        }
        Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Ok(program_archive)
        }
    }
}
