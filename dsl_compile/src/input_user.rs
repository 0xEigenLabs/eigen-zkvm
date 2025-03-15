use crate::errors::{bail, DslError, Result};
use ansi_term::Colour;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct Input {
    pub input_program: PathBuf,
    pub out_r1cs: PathBuf,
    pub out_json_constraints: PathBuf,
    pub out_wat_code: PathBuf,
    pub out_wasm_code: PathBuf,
    pub out_wasm_name: String,
    pub out_js_folder: PathBuf,
    pub out_c_run_name: String,
    pub out_c_folder: PathBuf,
    pub out_c_code: PathBuf,
    pub out_c_dat: PathBuf,
    pub out_sym: PathBuf,
    pub field: &'static str,
    pub c_flag: bool,
    pub wasm_flag: bool,
    pub wat_flag: bool,
    pub r1cs_flag: bool,
    pub sym_flag: bool,
    pub json_constraint_flag: bool,
    pub json_substitution_flag: bool,
    pub main_inputs_flag: bool,
    pub print_ir_flag: bool,
    pub fast_flag: bool,
    pub reduced_simplification_flag: bool,
    pub parallel_simplification_flag: bool,
    pub inspect_constraints_flag: bool,
    pub no_rounds: usize,
    pub flag_verbose: bool,
    pub prime: String,
    pub link_libraries: Vec<PathBuf>,
}

const P_0: &str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const R1CS: &str = "r1cs";
const WAT: &str = "wat";
const WASM: &str = "wasm";
const CPP: &str = "cpp";
const JS: &str = "js";
const DAT: &str = "dat";
const SYM: &str = "sym";
const JSON: &str = "json";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SimplificationStyle {
    O0,
    O1,
    O2(usize),
}
pub fn get_simplification_style(
    o_0: bool,
    o_1: bool,
    o_2: bool,
    o_2_argument: &str,
) -> Result<SimplificationStyle> {
    let no_rounds =
        if o_2_argument == "full" { Ok(usize::MAX) } else { o_2_argument.parse::<usize>() };
    match (o_0, o_1, o_2, no_rounds) {
        (true, _, _, _) => Ok(SimplificationStyle::O0),
        (_, true, _, _) => Ok(SimplificationStyle::O1),
        (_, _, true, Ok(no_rounds)) => Ok(SimplificationStyle::O2(no_rounds)),
        (false, false, false, _) => Ok(SimplificationStyle::O1),
        _ => {
            log::trace!("{}", Colour::Red.paint("invalid number of rounds"));
            bail!(DslError::CircomCompileError("invalid number of rounds".to_string(),))
        }
    }
}

impl Input {
    pub fn new(
        input: &Path,
        output_path: &Path,
        o_style: SimplificationStyle,
        prime: String,
        paths: Vec<String>,
    ) -> Result<Input> {
        let file_name = input.file_stem().unwrap().to_str().unwrap().to_string();
        let output_c_path = Input::build_folder(output_path, &file_name, CPP);
        let output_js_path = Input::build_folder(output_path, &file_name, JS);
        let mut link_libraries: Vec<PathBuf> = vec![];
        for path in paths.into_iter() {
            link_libraries.push(Path::new(&path).to_path_buf());
        }

        let input = input.to_path_buf();

        Ok(Input {
            field: P_0,
            input_program: input,
            out_r1cs: Input::build_output(output_path, &file_name, R1CS),
            out_wat_code: Input::build_output(&output_js_path, &file_name, WAT),
            out_wasm_code: Input::build_output(&output_js_path, &file_name, WASM),
            out_js_folder: output_js_path.to_path_buf(),
            out_wasm_name: file_name.clone(),
            out_c_folder: output_c_path.to_path_buf(),
            out_c_run_name: file_name.clone(),
            out_c_code: Input::build_output(&output_c_path, &file_name, CPP),
            out_c_dat: Input::build_output(&output_c_path, &file_name, DAT),
            out_sym: Input::build_output(output_path, &file_name, SYM),
            out_json_constraints: Input::build_output(
                output_path,
                &format!("{}_constraints", file_name),
                JSON,
            ),
            wat_flag: false,
            wasm_flag: true,
            c_flag: false,
            r1cs_flag: true,
            sym_flag: true,
            main_inputs_flag: false,
            json_constraint_flag: false,
            json_substitution_flag: false,
            print_ir_flag: false,
            no_rounds: if let SimplificationStyle::O2(r) = o_style { r } else { 0 },
            fast_flag: o_style == SimplificationStyle::O0,
            reduced_simplification_flag: o_style == SimplificationStyle::O1,
            parallel_simplification_flag: false, // TODO
            inspect_constraints_flag: false,
            flag_verbose: false,
            //prime: "bn128".to_string(), //goldilocks
            prime,
            link_libraries,
        })
    }

    fn build_folder(output_path: &Path, filename: &str, ext: &str) -> Box<Path> {
        let mut file = output_path.to_path_buf();
        let folder_name = format!("{}_{}", filename, ext);
        file.push(folder_name);

        file.into_boxed_path()
    }

    fn build_output(output_path: &Path, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.to_path_buf();
        file.push(format!("{}.{}", filename, ext));
        file
    }

    pub fn input_file(&self) -> &str {
        self.input_program.to_str().unwrap()
    }
    pub fn r1cs_file(&self) -> &str {
        self.out_r1cs.to_str().unwrap()
    }
    pub fn sym_file(&self) -> &str {
        self.out_sym.to_str().unwrap()
    }
    pub fn wat_file(&self) -> &str {
        self.out_wat_code.to_str().unwrap()
    }
    pub fn wasm_file(&self) -> &str {
        self.out_wasm_code.to_str().unwrap()
    }
    pub fn js_folder(&self) -> &str {
        self.out_js_folder.to_str().unwrap()
    }
    pub fn wasm_name(&self) -> String {
        self.out_wasm_name.clone()
    }

    pub fn c_folder(&self) -> &str {
        self.out_c_folder.to_str().unwrap()
    }
    pub fn c_run_name(&self) -> String {
        self.out_c_run_name.clone()
    }

    pub fn c_file(&self) -> &str {
        self.out_c_code.to_str().unwrap()
    }
    pub fn dat_file(&self) -> &str {
        self.out_c_dat.to_str().unwrap()
    }
    pub fn json_constraints_file(&self) -> &str {
        self.out_json_constraints.to_str().unwrap()
    }
    pub fn wasm_flag(&self) -> bool {
        self.wasm_flag
    }
    pub fn wat_flag(&self) -> bool {
        self.wat_flag
    }
    pub fn c_flag(&self) -> bool {
        self.c_flag
    }
    pub fn unsimplified_flag(&self) -> bool {
        self.fast_flag
    }
    pub fn r1cs_flag(&self) -> bool {
        self.r1cs_flag
    }
    pub fn json_constraints_flag(&self) -> bool {
        self.json_constraint_flag
    }
    pub fn json_substitutions_flag(&self) -> bool {
        self.json_substitution_flag
    }
    pub fn main_inputs_flag(&self) -> bool {
        self.main_inputs_flag
    }
    pub fn sym_flag(&self) -> bool {
        self.sym_flag
    }
    pub fn print_ir_flag(&self) -> bool {
        self.print_ir_flag
    }
    pub fn inspect_constraints_flag(&self) -> bool {
        self.inspect_constraints_flag
    }
    pub fn flag_verbose(&self) -> bool {
        self.flag_verbose
    }
    pub fn reduced_simplification_flag(&self) -> bool {
        self.reduced_simplification_flag
    }
    pub fn parallel_simplification_flag(&self) -> bool {
        self.parallel_simplification_flag
    }
    pub fn no_rounds(&self) -> usize {
        self.no_rounds
    }
    pub fn get_prime(&self) -> String {
        self.prime.clone()
    }
}
