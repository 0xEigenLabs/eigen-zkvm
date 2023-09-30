use crate::digest::ElementDigest;
use crate::errors::Result;
use crate::starkinfo::Program;
use crate::starkinfo::StarkInfo;
use crate::types::{StarkStruct, PIL};

pub struct StarkOption {
    pub enable_input: bool,
    pub verkey_input: bool,
    pub agg_stage: bool,
    pub skip_main: bool,
}

pub fn pil2circom(
    pil: &PIL,
    const_root: &ElementDigest<4>,
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
        "BN128" | "BLS12381" => crate::stark_verifier_circom_bn128::render(
            starkinfo,
            program,
            pil,
            stark_struct,
            const_root,
            options,
        ),
        _ => panic!("Invalid hash type: {}", stark_struct.verificationHashType),
    };
    return Ok(res);
}
