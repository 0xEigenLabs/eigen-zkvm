use crate::{
    merklehash::MerkleTreeGL,
    merklehash_bls12381::MerkleTreeBLS12381,
    merklehash_bn128::MerkleTreeBN128,
    pil2circom,
    polsarray::{PolKind, PolsArray},
    stark_gen::StarkProof,
    stark_setup::StarkSetup,
    stark_verify::stark_verify,
    traits::{MerkleTree, Transcript},
    transcript::TranscriptGL,
    transcript_bls12381::TranscriptBLS128,
    transcript_bn128::TranscriptBN128,
    types::*,
    ElementDigest,
};

use crate::field_bls12381::Fr as Fr_BLS12381;
use crate::field_bn128::Fr as Fr_BN128;
use ff::PrimeField;
use fields::field_gl::Fr as FGL;

use anyhow::Result;
use profiler_macro::time_profiler;
use std::fs::File;
use std::io::Write;

#[allow(clippy::too_many_arguments)]
#[time_profiler()]
pub fn stark_prove(
    stark_struct: &str,
    pil_file: &str,
    norm_stage: bool,
    skip_main: bool,
    agg_stage: bool,
    const_pol_file: &str,
    cm_pol_file: &str,
    circom_file: &str,
    zkin: &str,
    prover_addr: &str,
) -> Result<()> {
    let mut pil = load_json::<PIL>(pil_file)?;
    let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
    const_pol.load(const_pol_file)?;

    let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
    cm_pol.load(cm_pol_file)?;

    let stark_struct = load_json::<StarkStruct>(stark_struct)?;
    match stark_struct.verificationHashType.as_str() {
        "BN128" => prove::<Fr_BN128, MerkleTreeBN128, TranscriptBN128>(
            &mut pil,
            const_pol,
            cm_pol,
            &stark_struct,
            false,
            norm_stage,
            skip_main,
            circom_file,
            zkin,
            prover_addr,
        ),
        "BLS12381" => prove::<Fr_BLS12381, MerkleTreeBLS12381, TranscriptBLS128>(
            &mut pil,
            const_pol,
            cm_pol,
            &stark_struct,
            false,
            norm_stage,
            skip_main,
            circom_file,
            zkin,
            prover_addr,
        ),
        "GL" => prove::<FGL, MerkleTreeGL, TranscriptGL>(
            &mut pil,
            const_pol,
            cm_pol,
            &stark_struct,
            agg_stage,
            norm_stage,
            skip_main,
            circom_file,
            zkin,
            prover_addr,
        ),
        _ => panic!("Invalid hashtype {}", stark_struct.verificationHashType),
    }
}

// Adopt with different curve, eg: BN128, BLS12381, Goldilocks
#[allow(clippy::too_many_arguments)]
fn prove<
    F: PrimeField + Default,
    M: MerkleTree<MTNode = ElementDigest<4, F>> + Default,
    T: Transcript,
>(
    pil: &mut PIL,
    const_pol: PolsArray,
    cm_pol: PolsArray,
    stark_struct: &StarkStruct,
    agg_stage: bool,
    norm_stage: bool,
    skip_main: bool,
    circom_file: &str,
    zkin: &str,
    prover_addr: &str,
) -> Result<()> {
    let mut setup = StarkSetup::<M>::new(&const_pol, pil, stark_struct, None)?;
    let starkproof = StarkProof::<M>::stark_gen::<T>(
        cm_pol,
        const_pol,
        &setup.const_tree,
        &setup.starkinfo,
        &setup.program,
        pil,
        stark_struct,
        prover_addr,
    )?;
    log::debug!("generate the proof done");

    let result = stark_verify::<M, T>(
        &starkproof,
        &setup.const_root,
        &setup.starkinfo,
        stark_struct,
        &mut setup.program,
    )?;

    assert!(result);
    log::debug!("verify the proof done");

    let opt = pil2circom::StarkOption {
        enable_input: false,
        verkey_input: norm_stage,
        skip_main,
        agg_stage,
    };

    let str_ver = pil2circom::pil2circom::<F>(
        pil,
        &setup.const_root,
        stark_struct,
        &mut setup.starkinfo,
        &mut setup.program,
        &opt,
    )?;
    let mut file = File::create(circom_file)?;
    write!(file, "{}", str_ver)?;
    log::debug!("generate circom done");

    // if agg_stage {
    //     starkproof.rootC = None;
    // }

    let input = serde_json::to_string(&starkproof)?;
    let mut file = File::create(zkin)?;
    write!(file, "{}", input)?;
    log::debug!("generate zkin done");
    Ok(())
}
