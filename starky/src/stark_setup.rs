#![allow(non_snake_case, dead_code)]
use crate::errors::Result;
use crate::f3g::F3G;
use crate::fft_p::interpolate;
use crate::polsarray::PolsArray;
use crate::starkinfo::{self, Program, StarkInfo};
use crate::traits::MerkleTree;
use crate::types::{StarkStruct, PIL};
use crate::ElementDigest;
use rayon::prelude::*;
use winter_math::{fields::f64::BaseElement, FieldElement};

#[derive(Default)]
pub struct StarkSetup<M: MerkleTree> {
    pub const_tree: M,
    pub const_root: ElementDigest,
    pub starkinfo: StarkInfo,
    pub program: Program,
}

/// STARK SETUP
///
///  calculate the trace polynomial over extended field, return the new polynomial's coefficient.
impl<M: MerkleTree> StarkSetup<M> {
    pub fn new(
        const_pol: &PolsArray,
        pil: &mut PIL,
        stark_struct: &StarkStruct,
    ) -> Result<StarkSetup<M>> {
        log::debug!("StarkSetup: {:?}", stark_struct);
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
        let mut const_pols_array_e = vec![F3G::ZERO; (1 << nBitsExt) * pil.nConstants];
        let mut const_pols_array_e_be = vec![BaseElement::ZERO; (1 << nBitsExt) * pil.nConstants];

        log::debug!("before interpolate, const");
        crate::helper::pretty_print_array(&const_buff);
        interpolate(
            &const_buff,
            pil.nConstants,
            nBits,
            &mut const_pols_array_e,
            nBitsExt,
        );

        const_pols_array_e_be
            .par_iter_mut()
            .zip(const_pols_array_e)
            .for_each(|(be_out, f3g_in)| {
                *be_out = f3g_in.to_be();
            });
        log::debug!("before merklize, const");
        crate::helper::pretty_print_array(&const_pols_array_e_be);

        let mut const_tree = M::new();
        const_tree.merkelize(
            const_pols_array_e_be,
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

    use crate::field_bn128::Fr;
    use crate::merklehash::MerkleTreeGL;
    use crate::merklehash_bn128::MerkleTreeBN128;
    use crate::ElementDigest;
    use ff::*;
    use winter_math::fields::f64::BaseElement;

    #[test]
    fn test_stark_setup() {
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct).unwrap();
        let root: Fr = setup.const_root.into();

        let expect_root =
            "4658128321472362347225942316135505030498162093259225938328465623672244875764";
        assert_eq!(Fr::from_str(expect_root).unwrap(), root);
    }

    #[test]
    fn test_stark_setup_gl() {
        let mut pil = load_json::<PIL>("data/fib.pil.json.gl").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const.gl").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.gl").unwrap();
        let setup = StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct).unwrap();

        let expect_root = ElementDigest::from([
            BaseElement::from(15302509084042343527u64),
            BaseElement::from(985081440042889555u64),
            BaseElement::from(14692153289195851822u64),
            BaseElement::from(1611894784155222896u64),
        ]);
        assert_eq!(expect_root, setup.const_root);
    }
}
