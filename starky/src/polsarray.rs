use crate::types::PIL;
use std::collections::HashMap;

pub struct PolsArray {
    pub nPols: u32,
    pub def: HashMap<i32, i32>,
    pub defArray: Vec<i32>,
    pub array: Vec<i32>,
    // nameSpace, namePol,
    inner: HashMap<String, HashMap<String, Vec<i32>>>,
}

pub enum PolKind {
    Commit,
    Constant,
}

impl PolsArray {
    /*
    pub fn new(pil: PIL, kind: PolKind) -> Self
    {
        let nPols = match kind {
            PolKind::Commit => pil.nCommitments,
            PolKind::Constant => pil.nConstants,
        };

        for (name, ref_) in &pil.references {
            if (ref_.type_ == "cmP" && kind == PolKind::Commit) ||
                (ref_.type_ == "constP" && kind == PolKind::Constant) {
                    let name_vec Vec<&str> = name.split('.').collect();

            }
        }

    }
    */
}
