use starky::errors::Result;
use starky::{
    merklehash::MerkleTreeGL,
    merklehash_bn128::MerkleTreeBN128,
    pil2circom,
    polsarray::{PolKind, PolsArray},
    stark_gen::StarkProof,
    stark_setup::StarkSetup,
    stark_verify::stark_verify,
    transcript::TranscriptGL,
    transcript_bn128::TranscriptBN128,
    types::*,
};
use std::fs::File;
use std::io::Write;

pub fn prove(
    stark_struct: &String,
    pil_file: &String,
    const_pol_file: &String,
    cm_pol_file: &String,
    circom_file: &String,
    zkin: &String,
) -> Result<()> {
    let mut pil = load_json::<PIL>(pil_file.as_str()).unwrap();
    // load the const polynomials which writen in pil and compiled by 'pil-com'
    let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
    const_pol.load(const_pol_file.as_str()).unwrap();

    // load the commit polynomials which writen in pil and compiled by 'pil-com'
    let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
    cm_pol.load(cm_pol_file.as_str()).unwrap();

    let stark_struct = load_json::<StarkStruct>(stark_struct.as_str()).unwrap();
    match stark_struct.verificationHashType.as_str() {
        "BN128" => {

            // only the const polynomials will engage in the setup stage 
            let mut setup =
                StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct).unwrap();

            // generate the stark proof
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
            println!("verify the proof done");

            // translate the .pil to circom
            let opt = pil2circom::StarkOption {
                enable_input: false,
                verkey_input: false,
                skip_main: false,
            };

            println!("generate circom");
            let str_ver = pil2circom::pil2circom(
                &pil,
                &setup.const_root,
                &stark_struct,
                &mut setup.starkinfo,
                &mut setup.program,
                &opt,
            )?;
            let mut file = File::create(&circom_file)?;
            write!(file, "{}", str_ver)?;
            println!("generate circom done");

            let input = serde_json::to_string(&starkproof)?;
            let mut file = File::create(&zkin)?;
            write!(file, "{}", input)?;
            println!("generate zkin done");
        }
        "GL" => {
            let mut setup =
                StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct).unwrap();
            let starkproof = StarkProof::<MerkleTreeGL>::stark_gen::<TranscriptGL>(
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
            let result = stark_verify::<MerkleTreeGL, TranscriptGL>(
                &starkproof,
                &setup.const_root,
                &setup.starkinfo,
                &stark_struct,
                &mut setup.program,
            )
            .unwrap();

            assert_eq!(result, true);
            println!("verify the proof done");

            let opt = pil2circom::StarkOption {
                enable_input: false,
                verkey_input: false,
                skip_main: false,
            };

            println!("generate circom");
            let str_ver = pil2circom::pil2circom(
                &pil,
                &setup.const_root,
                &stark_struct,
                &mut setup.starkinfo,
                &mut setup.program,
                &opt,
            )?;
            let mut file = File::create(&circom_file)?;
            write!(file, "{}", str_ver)?;
            println!("generate circom done");

            let input = serde_json::to_string(&starkproof)?;
            let mut file = File::create(&zkin)?;
            write!(file, "{}", input)?;
            println!("generate zkin done");
        }
        _ => panic!("Invalid hashtype {}", stark_struct.verificationHashType),
    };
    Ok(())
}
