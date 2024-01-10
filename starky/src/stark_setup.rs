#![allow(non_snake_case, dead_code)]
use rayon::prelude::*;
use std::fs;
use std::path;

use crate::errors::Result;
use crate::fft_p::interpolate;
use crate::polsarray::PolsArray;
use crate::starkinfo::{self, Program, StarkInfo};
use crate::traits::MTNodeType;
use crate::traits::{FieldExtension, MerkleTree};
use crate::types::{StarkStruct, PIL};
use plonky::field_gl::Fr as FGL;
use profiler_macro::time_profiler;

#[derive(Default)]
pub struct StarkSetup<M: MerkleTree> {
    pub const_tree: M,
    pub const_root: M::MTNode,
    pub starkinfo: StarkInfo,
    pub program: Program,
}

impl<M: MerkleTree> StarkSetup<M> {
    pub fn save(&self, base_dir: &str, overwrite: bool) -> Result<()> {
        if overwrite && path::Path::new(base_dir).exists() {
            fs::remove_dir_all(base_dir)?;
        }
        std::fs::create_dir_all(base_dir)?;
        let base_dir = path::Path::new(base_dir);
        let ct = base_dir.join("const_tree");
        let mut writer = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(ct)?;
        self.const_tree.save(&mut writer)?;
        self.const_root.save(&mut writer)?;

        let si = base_dir.join("starkinfo");
        let si = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(si)?;
        serde_json::to_writer(si, &self.starkinfo)?;

        let pg = base_dir.join("program");
        let pg = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(pg)?;
        serde_json::to_writer(pg, &self.program)?;
        Ok(())
    }

    pub fn load(base_dir: &str) -> Result<Self> {
        let base_dir = path::Path::new(base_dir);
        let ct = base_dir.join("const_tree");
        let mut reader = fs::File::open(ct)?;
        let const_tree = M::load(&mut reader)?;
        let const_root = M::MTNode::load(&mut reader)?;

        let si = base_dir.join("starkinfo");
        let si = fs::File::open(si)?;
        let starkinfo: StarkInfo = serde_json::from_reader(si)?;

        let pg = base_dir.join("program");
        let pg = fs::File::open(pg)?;
        let program: Program = serde_json::from_reader(pg)?;
        Ok(StarkSetup {
            const_tree,
            const_root,
            starkinfo,
            program,
        })
    }
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

        let mut p: Vec<Vec<FGL>> = vec![Vec::new(); const_pol.nPols];
        for i in 0..const_pol.nPols {
            p[i] = vec![FGL::ZERO; const_pol.n];
            p[i].par_iter_mut().enumerate().for_each(|(j, out)| {
                *out = const_pol.array[i][j];
            });
        }

        log::trace!("Write const pol buff and interpolate");
        let const_buff = const_pol.write_buff();
        //extend and merkelize
        let mut const_pols_array_e = vec![M::ExtendField::ZERO; (1 << nBitsExt) * pil.nConstants];
        let mut const_pols_array_e_be = vec![FGL::ZERO; (1 << nBitsExt) * pil.nConstants];

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
    use plonky::field_gl::Fr as FGL;

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

        let expect_root = ElementDigest::<4>::new(&[
            FGL::from(15302509084042343527u64),
            FGL::from(985081440042889555u64),
            FGL::from(14692153289195851822u64),
            FGL::from(1611894784155222896u64),
        ]);
        assert_eq!(expect_root, setup.const_root);
    }
}
