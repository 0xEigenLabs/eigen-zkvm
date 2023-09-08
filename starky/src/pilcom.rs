//! Poring from https://github.com/powdr-labs/powdr.git.
// pub use compiler::*;

use crate::types::{read_json, PIL};

// compile .pil to .pil.json
pub fn compile_pil(pil_str: &String) -> PIL {
    // 1. compile pil_str to pil_json.
    // todo()!
    let pil_json = pil_str.clone();

    read_json::<PIL>(pil_json).unwrap()
}
