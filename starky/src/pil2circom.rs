use crate::digest::ElementDigest;
use crate::starkinfo::Program;
use crate::starkinfo::StarkInfo;
use crate::types::{StarkStruct, PIL};
use anyhow::Result;
use profiler_macro::time_profiler;

/// The option to control the generation of recursive verifier
pub struct StarkOption {
    // enable the stark verifier
    pub enable_input: bool,
    // normalize the proof
    pub verkey_input: bool,
    // aggragte the proof
    pub agg_stage: bool,
    // generate the main component in Circom
    pub skip_main: bool,
}

#[time_profiler()]
pub fn pil2circom<F: ff::PrimeField + Default>(
    pil: &PIL,
    const_root: &ElementDigest<4, F>,
    stark_struct: &StarkStruct,
    starkinfo: &mut StarkInfo,
    program: &mut Program,
    options: &StarkOption,
) -> Result<String> {
    starkinfo.set_code_dimensions_first(&mut program.verifier_code)?;
    starkinfo.set_code_dimensions_first(&mut program.verifier_query_code)?;
    let res = match stark_struct.verificationHashType.as_str() {
        "GL" => crate::stark_verifier_circom::render(
            starkinfo,
            program,
            pil,
            stark_struct,
            const_root,
            options,
        ),
        "BN128" | "BLS12381" => crate::stark_verifier_circom_onchain::render(
            starkinfo,
            program,
            pil,
            stark_struct,
            const_root,
            options,
        ),
        _ => panic!("Invalid hash type: {}", stark_struct.verificationHashType),
    };
    Ok(res)
}
