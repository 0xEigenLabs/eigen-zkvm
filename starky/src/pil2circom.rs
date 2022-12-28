use crate::digest::ElementDigest;
use crate::errors::Result;
use crate::f3g::F3G;
use crate::polsarray::PolsArray;
use crate::starkinfo::StarkInfo;
use crate::types::{StarkStruct, PIL};
use std::collections::HashMap;
use crate::starkinfo::Program;

pub fn pil2circom(
    pil: &PIL,
    const_root: &ElementDigest,
    stark_struct: &StarkStruct,
    starkinfo: &mut StarkInfo,
    program: &mut Program,
    options: HashMap<String, String>,
) -> Result<()> {

    starkinfo.set_code_dimensions_first(&mut program.verifier_code);
    starkinfo.set_code_dimensions_first(&mut program.verifier_query_code);


    return Ok(());
}
