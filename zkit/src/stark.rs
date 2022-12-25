use starky::{
    merklehash_bn128::MerkleTreeBN128,
    polsarray::{PolKind, PolsArray},
    stark_gen::StarkProof,
    stark_setup::StarkSetup,
    stark_verify::stark_verify,
    transcript_bn128::TranscriptBN128,
    types::*,
};

pub fn prove(
    stark_struct: &String,
    pil_file: &String,
    const_pol_file: &String,
    cm_pol_file: &String,
) -> Result<(), anyhow::Error> {
    let mut pil = load_json::<PIL>(pil_file.as_str()).unwrap();
    let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
    const_pol.load(const_pol_file.as_str()).unwrap();

    let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
    cm_pol.load(cm_pol_file.as_str()).unwrap();

    let stark_struct = load_json::<StarkStruct>(stark_struct.as_str()).unwrap();
    let mut setup =
        StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct).unwrap();
    let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
        &cm_pol,
        &const_pol,
        &setup.const_tree,
        &setup.starkinfo,
        &setup.program,
        &pil,
        &stark_struct,
    )
    .unwrap();

    println!("verify the proof...");

    let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
        &starkproof,
        &setup.const_root,
        &setup.starkinfo,
        &stark_struct,
        &mut setup.program,
    )
    .unwrap();
    assert_eq!(result, true);
    println!("verify the proof successfully");
    Ok(())
}
