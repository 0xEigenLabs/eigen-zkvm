use crate::f3g as field;
use crate::types::{StarkStruct, PIL};
pub struct StarkInfo;

impl StarkInfo {
    fn new(pil: &PIL, stark_struct: &StarkStruct) -> Self {
        let pil_deg = pil.references.values().nth(0).unwrap().polDeg;

        StarkInfo
    }
}
