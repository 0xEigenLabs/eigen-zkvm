use anyhow::{anyhow, bail, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read};
use std::str;

use crate::bellman_ce::{
    kate_commitment::{Crs, CrsForLagrangeForm, CrsForMonomialForm},
    pairing::Engine,
    plonk::{
        better_cs::cs::PlonkCsWidth4WithNextStepParams,
        better_cs::keys::{Proof, VerificationKey},
    },
    Field, PrimeField, PrimeFieldRepr, ScalarEngine,
};

#[cfg(not(feature = "wasm"))]
use crate::aggregation::{AggregatedProof, AggregationVerificationKey};

/// load proof by filename
pub fn load_proof<E: Engine>(filename: &str) -> Proof<E, PlonkCsWidth4WithNextStepParams> {
    Proof::<E, PlonkCsWidth4WithNextStepParams>::read(
        File::open(filename).unwrap_or_else(|_| panic!("read proof file err, {}", filename)),
    )
    .expect("read proof err")
}

/// load multiple proofs form a list
pub fn load_proofs_from_list<E: Engine>(
    list: &str,
) -> Vec<Proof<E, PlonkCsWidth4WithNextStepParams>> {
    let file = File::open(list).unwrap_or_else(|_| panic!("read proof list file err, {}", list));
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("could not parse line"))
        .collect();
    let proofs: Vec<Proof<E, PlonkCsWidth4WithNextStepParams>> = lines
        .iter()
        .map(|l| {
            log::trace!("reading {:?}", l);
            load_proof::<E>(l)
        })
        .collect();

    assert!(!proofs.is_empty(), "no proof file found!");

    let num_inputs = proofs[0].num_inputs;
    for p in &proofs {
        assert_eq!(p.num_inputs, num_inputs, "proofs num_inputs mismatch!");
    }

    proofs
}

/// load verification key file by filename
pub fn load_verification_key<E: Engine>(
    filename: &str,
) -> VerificationKey<E, PlonkCsWidth4WithNextStepParams> {
    let mut reader = BufReader::with_capacity(
        1 << 24,
        File::open(filename).unwrap_or_else(|_| panic!("read vk file err, {}", filename)),
    );
    VerificationKey::<E, PlonkCsWidth4WithNextStepParams>::read(&mut reader).expect("read vk err")
}

/// get universal setup file by filename
fn get_universal_setup_file_buff_reader(setup_file_name: &str) -> Result<BufReader<File>> {
    let setup_file = File::open(setup_file_name).map_err(|e| {
        anyhow!(format!(
            "Failed to open universal setup file {}, err: {}",
            setup_file_name, e
        ))
    })?;
    Ok(BufReader::with_capacity(1 << 29, setup_file))
}

/// load monomial form SRS by filename
pub fn load_key_monomial_form<E: Engine>(filename: &str) -> Crs<E, CrsForMonomialForm> {
    let mut buf_reader = get_universal_setup_file_buff_reader(filename)
        .unwrap_or_else(|_| panic!("read key_monomial_form file err, {}", filename));
    Crs::<E, CrsForMonomialForm>::read(&mut buf_reader).expect("read key_monomial_form err")
}

/// load optional lagrange form SRS by filename
pub fn maybe_load_key_lagrange_form<E: Engine>(
    option_filename: Option<String>,
) -> Option<Crs<E, CrsForLagrangeForm>> {
    match option_filename {
        None => None,
        Some(filename) => {
            let mut buf_reader = get_universal_setup_file_buff_reader(&filename)
                .unwrap_or_else(|_| panic!("read key_lagrange_form file err, {}", filename));
            let key_lagrange_form = Crs::<E, CrsForLagrangeForm>::read(&mut buf_reader)
                .expect("read key_lagrange_form err");
            Some(key_lagrange_form)
        }
    }
}

/// load witness file by filename with autodetect encoding (bin or json).
pub fn load_witness_from_file<E: ScalarEngine>(filename: &str) -> Vec<E::Fr> {
    if filename.ends_with("json") {
        load_witness_from_json_file::<E>(filename)
    } else {
        load_witness_from_bin_file::<E>(filename)
    }
}

/// load witness from json file by filename
pub fn load_witness_from_json_file<E: ScalarEngine>(filename: &str) -> Vec<E::Fr> {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("unable to open {}.", filename));
    load_witness_from_json::<E, BufReader<File>>(BufReader::new(reader))
}

/// load witness from json by a reader
fn load_witness_from_json<E: ScalarEngine, R: Read>(reader: R) -> Vec<E::Fr> {
    let witness: Vec<String> = serde_json::from_reader(reader).expect("unable to read.");
    witness
        .into_iter()
        .map(|x| E::Fr::from_str(&x).unwrap())
        .collect::<Vec<E::Fr>>()
}

/// load witness from bin file by filename
pub fn load_witness_from_bin_file<E: ScalarEngine>(filename: &str) -> Vec<E::Fr> {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("unable to open {}.", filename));
    load_witness_from_bin_reader::<E, BufReader<File>>(BufReader::new(reader))
        .expect("read witness failed")
}

/// load witness from u8 array
pub fn load_witness_from_array<E: ScalarEngine>(buffer: Vec<u8>) -> Result<Vec<E::Fr>> {
    load_witness_from_bin_reader::<E, _>(buffer.as_slice())
}

/// load witness from u8 array by a reader
pub fn load_witness_from_bin_reader<E: ScalarEngine, R: Read>(mut reader: R) -> Result<Vec<E::Fr>> {
    let mut wtns_header = [0u8; 4];
    reader.read_exact(&mut wtns_header)?;
    if wtns_header != [119, 116, 110, 115] {
        // python -c 'print([ord(c) for c in "wtns"])' => [119, 116, 110, 115]
        bail!("Invalid file header");
    }
    let version = reader.read_u32::<LittleEndian>()?;
    log::trace!("wtns version {}", version);
    if version > 2 {
        bail!("unsupported file version");
    }
    let num_sections = reader.read_u32::<LittleEndian>()?;
    if num_sections != 2 {
        bail!("invalid num sections");
    }
    // read the first section
    let sec_type = reader.read_u32::<LittleEndian>()?;
    if sec_type != 1 {
        bail!("invalid section type");
    }
    let sec_size = reader.read_u64::<LittleEndian>()?;
    if sec_size != 4 + 32 + 4 {
        bail!("invalid section len");
    }
    let field_size = reader.read_u32::<LittleEndian>()?;
    if field_size != 32 {
        bail!("invalid field byte size");
    }
    let mut prime = vec![0u8; field_size as usize];
    reader.read_exact(&mut prime)?;
    if prime != hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430") {
        bail!("invalid curve prime");
    }
    let witness_len = reader.read_u32::<LittleEndian>()?;
    log::trace!("witness len {}", witness_len);
    let sec_type = reader.read_u32::<LittleEndian>()?;
    if sec_type != 2 {
        bail!("invalid section type");
    }
    let sec_size = reader.read_u64::<LittleEndian>()?;
    if sec_size != (witness_len * field_size) as u64 {
        bail!(format!("Invalid witness section size {}", sec_size));
    }
    let mut result = Vec::with_capacity(witness_len as usize);
    for _ in 0..witness_len {
        let mut repr = E::Fr::zero().into_repr();
        repr.read_le(&mut reader)?;
        result.push(E::Fr::from_repr(repr)?);
    }
    Ok(result)
}

/// load aggregation proof file by filename
#[cfg(not(feature = "wasm"))]
pub fn load_aggregated_proof(filename: &str) -> AggregatedProof {
    AggregatedProof::read(File::open(filename).expect("read aggregated proof file err"))
        .expect("read aggregated proof err")
}

/// load aggregation verification key file by filename
#[cfg(not(feature = "wasm"))]
pub fn load_aggregation_verification_key(filename: &str) -> AggregationVerificationKey<'static> {
    let mut reader = BufReader::with_capacity(
        1 << 24,
        File::open(filename)
            .unwrap_or_else(|_| panic!("read aggregation vk file err, {}", filename)),
    );
    AggregationVerificationKey::read(&mut reader).expect("read aggregation vk err")
}
