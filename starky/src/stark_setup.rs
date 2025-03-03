#![allow(non_snake_case, dead_code)]
use crate::fft_p::interpolate;
use crate::polsarray::PolsArray;
use crate::starkinfo::{self, Program, StarkInfo};
use crate::traits::{FieldExtension, MerkleTree};
use crate::types::{StarkStruct, PIL};
use anyhow::Result;
use fields::field_gl::Fr as FGL;
use profiler_macro::time_profiler;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct StarkSetup<M: MerkleTree> {
    pub const_tree: M,
    pub const_root: M::MTNode,
    pub starkinfo: StarkInfo,
    pub program: Program,
}

/// STARK SETUP
///
///  calculate the trace polynomial over extended field, return the new polynomial's coefficient.
impl<M: MerkleTree> StarkSetup<M> {
    // global_l1: https://github.com/0xEigenLabs/eigen-zkvm/pull/91
    #[time_profiler("stark_setup")]
    pub fn new(
        const_pol: &PolsArray,
        pil: &mut PIL,
        stark_struct: &StarkStruct,
        global_l1: Option<String>,
    ) -> Result<StarkSetup<M>> {
        let nBits = stark_struct.nBits;
        let nBitsExt = stark_struct.nBitsExt;
        assert_eq!(const_pol.nPols, pil.nConstants);

        log::trace!("Write const pol buff and interpolate");
        let const_buff = const_pol.write_buff();
        //extend and merkelize
        let mut const_pols_array_e = vec![M::ExtendField::ZERO; (1 << nBitsExt) * pil.nConstants];
        let mut const_pols_array_e_be = vec![FGL::ZERO; (1 << nBitsExt) * pil.nConstants];

        interpolate(&const_buff, pil.nConstants, nBits, &mut const_pols_array_e, nBitsExt);

        const_pols_array_e_be.par_iter_mut().zip(const_pols_array_e).for_each(
            |(be_out, f3g_in)| {
                *be_out = f3g_in.to_be();
            },
        );

        let mut const_tree = M::new();
        log::trace!("Merkelize const tree");
        const_tree.merkelize(
            const_pols_array_e_be,
            const_pol.nPols,
            const_pol.n << (nBitsExt - nBits),
        )?;

        let starkinfo = starkinfo::StarkInfo::new(pil, stark_struct, global_l1)?;
        Ok(StarkSetup {
            const_root: const_tree.root(),
            const_tree,
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
    use crate::traits::MTNodeType;
    use crate::ElementDigest;
    use ff::*;
    use fields::field_gl::Fr as FGL;

    #[test]
    fn test_stark_setup() {
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

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
        let setup =
            StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();

        let expect_root = ElementDigest::<4, FGL>::new(&[
            FGL::from(15302509084042343527u64),
            FGL::from(985081440042889555u64),
            FGL::from(14692153289195851822u64),
            FGL::from(1611894784155222896u64),
        ]);
        assert_eq!(expect_root, setup.const_root);
    }

    #[test]
    fn test_stark_setup_serialize_and_deserialize() {
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let data =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();

        let serialized = serde_json::to_string(&data).unwrap();
        println!("Serialized: {}", serialized);

        let expect: StarkSetup<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();
        let root: Fr = Fr(expect.const_root.as_scalar::<Fr>());

        let expect_root =
            "4658128321472362347225942316135505030498162093259225938328465623672244875764";
        assert_eq!(Fr::from_str(expect_root).unwrap(), root);
    }
}
