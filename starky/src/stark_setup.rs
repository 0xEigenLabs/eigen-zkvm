#![allow(non_snake_case)]
use crate::errors::Result;
use crate::f3g::F3G;
use crate::fft_p::interpolate;
use crate::merklehash_bn128::MerkleTree;
use crate::polsarray::PolsArray;
use crate::starkinfo::{self, Program, StarkInfo};
use crate::types::{StarkStruct, PIL};
use crate::ElementDigest;

use winter_math::{fields::f64::BaseElement, FieldElement, StarkField};

use winter_utils::{iter, transpose_slice};

#[derive(Default)]
pub struct StarkSetup {
    pub const_tree: MerkleTree,
    pub const_root: ElementDigest,
    pub starkinfo: StarkInfo,
    pub program: Program,
}

/// STARK SETUP
///
///  calculate the trace polynomial over extended field, return the new polynomial's coefficient.
impl StarkSetup {
    pub fn new(
        const_pol: &PolsArray,
        pil: &mut PIL,
        stark_struct: &StarkStruct,
    ) -> Result<StarkSetup> {
        let nBits = stark_struct.nBits;
        let nBitsExt = stark_struct.nBitsExt;
        assert_eq!(const_pol.nPols, pil.nConstants);

        let mut p: Vec<Vec<BaseElement>> = vec![Vec::new(); const_pol.nPols];
        for i in 0..const_pol.nPols {
            for j in 0..const_pol.n {
                p[i].push(const_pol.array[i][j])
            }
        }

        let const_buff = const_pol.write_buff();
        //extend and merkelize
        let mut constPolsArrayE = vec![F3G::ZERO; (1 << nBitsExt) * pil.nConstants];

        interpolate(
            &const_buff,
            pil.nConstants,
            nBits,
            &mut constPolsArrayE,
            nBitsExt,
        );

        let const_tree = MerkleTree::merkelize(
            constPolsArrayE,
            const_pol.nPols,
            const_pol.n << (nBitsExt - nBits),
        )?;

        let starkinfo = starkinfo::StarkInfo::new(pil, stark_struct)?;
        Ok(StarkSetup {
            const_root: const_tree.root(),
            const_tree: const_tree,
            starkinfo: starkinfo.0,
            program: starkinfo.1,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::polsarray::{PolKind, PolsArray};
    use crate::stark_setup::StarkSetup;
    use crate::types::{load_json, StarkStruct, PIL};
    use winter_math::fft::{self, get_inv_twiddles};
    use winter_math::{fields::f64::BaseElement, FieldElement, StarkField};

    //use super::interpolate_in_pil;
    use crate::poseidon_bn128::Fr;
    use ff::*;

    #[test]
    fn test_stark_setup() {
        let mut pil = load_json::<PIL>("data/fib.pil.json.2").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant, 32);
        const_pol.load("data/fib.const.2").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.2").unwrap();
        let setup = StarkSetup::new(&const_pol, &mut pil, &stark_struct).unwrap();
        let root: Fr = setup.const_root.into();

        let expect_root =
            "4658128321472362347225942316135505030498162093259225938328465623672244875764";
        assert_eq!(Fr::from_str(expect_root).unwrap(), root);
        //crate::helper::pretty_print_matrix(&setup.const_tree.elements);
    }
}
