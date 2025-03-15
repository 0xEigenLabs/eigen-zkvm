use anyhow::{bail, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek};
use std::str;

use ff::PrimeField;

use crate::circom_circuit::{CircuitJson, R1CS};

/// get universal setup file by filename
#[allow(dead_code)]
fn get_universal_setup_file_buff_reader(setup_file_name: &str) -> Result<BufReader<File>> {
    let setup_file = File::open(setup_file_name)?;
    Ok(BufReader::with_capacity(1 << 29, setup_file))
}

/// load witness file by filename with autodetect encoding (bin or json).
pub fn load_witness_from_file<E: PrimeField>(filename: &str) -> Vec<E> {
    if filename.ends_with("json") {
        load_witness_from_json_file::<E>(filename)
    } else {
        load_witness_from_bin_file::<E>(filename)
    }
}

/// load witness from json file by filename
pub fn load_witness_from_json_file<E: PrimeField>(filename: &str) -> Vec<E> {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("Unable to open {}.", filename));
    load_witness_from_json::<E, BufReader<File>>(BufReader::new(reader))
}

/// load witness from json by a reader
fn load_witness_from_json<E: PrimeField, R: Read>(reader: R) -> Vec<E> {
    let witness: Vec<String> = serde_json::from_reader(reader).expect("Unable to read.");
    witness.into_iter().map(|x| E::from_str_vartime(&x).unwrap()).collect::<Vec<E>>()
}

/// load witness from bin file by filename
pub fn load_witness_from_bin_file<E: PrimeField>(filename: &str) -> Vec<E> {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("Unable to open {}.", filename));
    load_witness_from_bin_reader::<E, BufReader<File>>(BufReader::new(reader))
        .expect("read witness failed")
}

/// load witness from u8 array
pub fn load_witness_from_array<E: PrimeField>(buffer: Vec<u8>) -> Result<Vec<E>> {
    load_witness_from_bin_reader::<E, _>(buffer.as_slice())
}

/// load witness from u8 array by a reader
pub fn load_witness_from_bin_reader<E: PrimeField, R: Read>(mut reader: R) -> Result<Vec<E>> {
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
        bail!("invalid num sections".to_string());
    }
    // read the first section
    let sec_type = reader.read_u32::<LittleEndian>()?;
    if sec_type != 1 {
        bail!("invalid section type".to_string());
    }
    let sec_size = reader.read_u64::<LittleEndian>()?;
    if sec_size != 4 + 32 + 4 {
        bail!("invalid section len".to_string());
    }
    let field_size = reader.read_u32::<LittleEndian>()?;
    if field_size != 32 {
        bail!("invalid field byte size".to_string());
    }
    let mut prime = vec![0u8; field_size as usize];
    reader.read_exact(&mut prime)?;
    if prime != hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430") {
        bail!("invalid curve prime".to_string());
    }
    let witness_len = reader.read_u32::<LittleEndian>()?;
    log::trace!("witness len {}", witness_len);
    let sec_type = reader.read_u32::<LittleEndian>()?;
    if sec_type != 2 {
        bail!("invalid section type".to_string());
    }
    let sec_size = reader.read_u64::<LittleEndian>()?;
    if sec_size != (witness_len * field_size) as u64 {
        bail!(format!("Invalid witness section size {}", sec_size));
    }
    let mut result = Vec::with_capacity(witness_len as usize);
    for _ in 0..witness_len {
        let mut repr = E::default().to_repr();
        let repr_slice = repr.as_mut();
        if reader.read_exact(repr_slice).is_err() {
            continue;
        }
        let maybe_field_elem = E::from_repr(repr);
        if maybe_field_elem.is_some().unwrap_u8() == 1 {
            result.push(maybe_field_elem.unwrap());
        } else {
            continue;
        }
        // repr.read_le(&mut reader)?;
        // result.push(E::Fr::from_repr(repr)?);
    }
    Ok(result)
}

/// load r1cs file by filename with autodetect encoding (bin or json)
pub fn load_r1cs<E: PrimeField>(filename: &str) -> R1CS<E> {
    if filename.ends_with("json") {
        load_r1cs_from_json_file(filename)
    } else {
        let (r1cs, _wire_mapping) = load_r1cs_from_bin_file(filename);
        r1cs
    }
}

/// load r1cs from json file by filename
fn load_r1cs_from_json_file<E: PrimeField>(filename: &str) -> R1CS<E> {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("Unable to open {}.", filename));
    load_r1cs_from_json(BufReader::new(reader))
}

/// load r1cs from json by a reader
fn load_r1cs_from_json<E: PrimeField, R: Read>(reader: R) -> R1CS<E> {
    let circuit_json: CircuitJson = serde_json::from_reader(reader).expect("Unable to read.");

    let num_inputs = circuit_json.num_inputs + circuit_json.num_outputs + 1;
    let num_aux = circuit_json.num_variables - num_inputs;

    let convert_constraint = |lc: &BTreeMap<String, String>| {
        lc.iter()
            .map(|(index, coeff)| (index.parse().unwrap(), E::from_str_vartime(coeff).unwrap()))
            .collect_vec()
    };

    let constraints = circuit_json
        .constraints
        .iter()
        .map(|c| (convert_constraint(&c[0]), convert_constraint(&c[1]), convert_constraint(&c[2])))
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
fn load_r1cs_from_bin_file<E: PrimeField>(filename: &str) -> (R1CS<E>, Vec<usize>) {
    let reader = OpenOptions::new()
        .read(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("Unable to open {}.", filename));
    load_r1cs_from_bin(BufReader::new(reader))
}

/// load r1cs from bin by a reader
pub fn load_r1cs_from_bin<R: Read + Seek, E: PrimeField>(reader: R) -> (R1CS<E>, Vec<usize>) {
    let file = crate::r1cs_file::from_reader::<R, E>(reader).expect("Unable to read.");
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
