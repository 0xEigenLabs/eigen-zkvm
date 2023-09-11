//! Poring from https://github.com/powdr-labs/powdr.git.
pub use backend::*;
pub use compiler::*;

use crate::types::{read_json, PIL};

pub fn compile_pil(pil_str: &String) -> PIL {
    // 1. compile pil_str to pil_json.
    // todo()!
    let pil_json = pil_str.clone();
}

// todo
pub fn compile_pil_from_str(pil_str: &String) -> PIL {
    let compilationResult = compiler::compile_pil();

    // backend::export()
}
