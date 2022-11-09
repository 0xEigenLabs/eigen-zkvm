use crate::f3g as field;
use std::collections::HashMap;
use crate::types::{StarkStruct, PIL};
use crate::errors::{EigenError, Result};

pub struct StarkInfo {
    var_pol_map: usize,
    pu_ctx: usize,
    pe_ctx: usize,
    ci_ctx: usize,
    n_constants: usize,
    n_publics: usize,
}

impl StarkInfo {

    fn new(pil: &PIL, stark_struct: &StarkStruct) -> Result<Self> {
        let pil_deg = pil.references.values().nth(0).unwrap().polDeg as i32;

        let stark_deg = 2i32.pow(stark_struct.nBits as u32);

        if stark_deg != pil_deg {
            return Err(EigenError::MustEqualDegreeError(stark_deg, pil_deg));
        }

        if stark_struct.nBitsExt != stark_struct.steps[0]["nBits"] {
            return Err(EigenError::MustEqualDegreeError(stark_struct.nBitsExt, stark_struct.steps[0]["nBits"]));
        }


        Ok(StarkInfo)
    }

    pub fn generate_pubulic_calculators(&mut self, pil: &PIL) -> Result<()> {

        Ok(())
    }

}
