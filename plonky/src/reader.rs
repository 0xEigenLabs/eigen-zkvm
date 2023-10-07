use crate::errors::{EigenError, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek};
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

use crate::circom_circuit::{CircuitJson, R1CS};

#[cfg(not(feature = "wasm"))]
use crate::aggregation::{AggregatedProof, AggregationVerificationKey};

/// load proof by filename
pub fn load_proof<E: Engine>(filename: &str) -> Proof<E, PlonkCsWidth4WithNextStepParams> {
    Proof::<E, PlonkCsWidth4WithNextStepParams>::read(
        File::open(filename).expect("read proof file err"),
    )
    .expect("read proof err")
}

/// load multiple proofs form a list
pub fn load_proofs_from_list<E: Engine>(
    list: &str,
) -> Vec<Proof<E, PlonkCsWidth4WithNextStepParams>> {
    let file = File::open(list).expect("read proof list file err");
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("could not parse line"))
        .collect();
    let proofs: Vec<Proof<E, PlonkCsWidth4WithNextStepParams>> = lines
        .iter()
        .map(|l| {
            log::debug!("reading {:?}", l);
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
    let mut reader =
        BufReader::with_capacity(1 << 24, File::open(filename).expect("read vk file err"));
    VerificationKey::<E, PlonkCsWidth4WithNextStepParams>::read(&mut reader).expect("read vk err")
}

/// get universal setup file by filename
fn get_universal_setup_file_buff_reader(setup_file_name: &str) -> Result<BufReader<File>> {
    let setup_file = File::open(setup_file_name).map_err(|e| {
        EigenError::from(format!(
            "Failed to open universal setup file {}, err: {}",
            setup_file_name, e
        ))
    })?;
    Ok(BufReader::with_capacity(1 << 29, setup_file))
}

/// load monomial form SRS by filename
pub fn load_key_monomial_form<E: Engine>(filename: &str) -> Crs<E, CrsForMonomialForm> {
    let mut buf_reader =
        get_universal_setup_file_buff_reader(filename).expect("read key_monomial_form file err");
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
                .expect("read key_lagrange_form file err");
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
        .expect("unable to open.");
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
        .expect("unable to open.");
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
        return Err(EigenError::from("Invalid file header".to_string()));
    }
    let version = reader.read_u32::<LittleEndian>()?;
    log::debug!("wtns version {}", version);
    if version > 2 {
        return Err(EigenError::from("unsupported file version".to_string()));
    }
    let num_sections = reader.read_u32::<LittleEndian>()?;
    if num_sections != 2 {
        return Err(EigenError::from("invalid num sections".to_string()));
    }
    // read the first section
    let sec_type = reader.read_u32::<LittleEndian>()?;
    if sec_type != 1 {
        return Err(EigenError::from("invalid section type".to_string()));
    }
    let sec_size = reader.read_u64::<LittleEndian>()?;
    if sec_size != 4 + 32 + 4 {
        return Err(EigenError::from("invalid section len".to_string()));
    }
    let field_size = reader.read_u32::<LittleEndian>()?;
    if field_size != 32 {
        return Err(EigenError::from("invalid field byte size".to_string()));
    }
    let mut prime = vec![0u8; field_size as usize];
    reader.read_exact(&mut prime)?;
    if prime != hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430") {
        return Err(EigenError::from("invalid curve prime".to_string()));
    }
    let witness_len = reader.read_u32::<LittleEndian>()?;
    log::debug!("witness len {}", witness_len);
    let sec_type = reader.read_u32::<LittleEndian>()?;
    if sec_type != 2 {
        return Err(EigenError::from("invalid section type".to_string()));
    }
    let sec_size = reader.read_u64::<LittleEndian>()?;
    if sec_size != (witness_len * field_size) as u64 {
        return Err(EigenError::from(format!(
            "Invalid witness section size {}",
            sec_size
        )));
    }
    let mut result = Vec::with_capacity(witness_len as usize);
    for _ in 0..witness_len {
        let mut repr = E::Fr::zero().into_repr();
        repr.read_le(&mut reader)?;
        result.push(E::Fr::from_repr(repr)?);
    }
    Ok(result)
}

/// load r1cs file by filename with autodetect encoding (bin or json)
pub fn load_r1cs<E: ScalarEngine>(filename: &str) -> R1CS<E> {
    if filename.ends_with("json") {
        load_r1cs_from_json_file(filename)
    } else {
        let (r1cs, _wire_mapping) = load_r1cs_from_bin_file(filename);
        r1cs
    }
}

/// load r1cs from json file by filename
fn load_r1cs_from_json_file<E: ScalarEngine>(filename: &str) -> R1CS<E> {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .expect("unable to open.");
    load_r1cs_from_json(BufReader::new(reader))
}

/// load r1cs from json by a reader
fn load_r1cs_from_json<E: ScalarEngine, R: Read>(reader: R) -> R1CS<E> {
    let circuit_json: CircuitJson = serde_json::from_reader(reader).expect("unable to read.");

    let num_inputs = circuit_json.num_inputs + circuit_json.num_outputs + 1;
    let num_aux = circuit_json.num_variables - num_inputs;

    let convert_constraint = |lc: &BTreeMap<String, String>| {
        lc.iter()
            .map(|(index, coeff)| (index.parse().unwrap(), E::Fr::from_str(coeff).unwrap()))
            .collect_vec()
    };

    let constraints = circuit_json
        .constraints
        .iter()
        .map(|c| {
            (
                convert_constraint(&c[0]),
                convert_constraint(&c[1]),
                convert_constraint(&c[2]),
            )
        })
        .collect_vec();

    R1CS {
        num_inputs,
        num_aux,
        num_variables: circuit_json.num_variables,
        num_outputs: circuit_json.num_outputs,
        constraints,
        custom_gates: vec![],
        custom_gates_uses: vec![],
    }
}

/// load r1cs from bin file by filename
fn load_r1cs_from_bin_file<E: ScalarEngine>(filename: &str) -> (R1CS<E>, Vec<usize>) {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .expect(&format!("unable to open {}.", filename));
    load_r1cs_from_bin(BufReader::new(reader))
}

/// load r1cs from bin by a reader
pub fn load_r1cs_from_bin<R: Read + Seek, E: ScalarEngine>(reader: R) -> (R1CS<E>, Vec<usize>) {
    let file = crate::r1cs_file::from_reader::<R, E>(reader).expect("unable to read.");
    let num_inputs = (1 + file.header.n_pub_in + file.header.n_pub_out) as usize;
    let num_variables = file.header.n_wires as usize;
    let num_aux = num_variables - num_inputs;
    (
        R1CS {
            num_aux,
            num_inputs,
            num_variables,
            num_outputs: file.header.n_pub_out as usize,
            constraints: file.constraints,
            custom_gates: file.custom_gates,
            custom_gates_uses: file.custom_gates_uses,
        },
        file.wire_mapping.iter().map(|e| *e as usize).collect_vec(),
    )
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
        File::open(filename).expect("read aggregation vk file err"),
    );
    AggregationVerificationKey::read(&mut reader).expect("read aggregation vk err")
}
