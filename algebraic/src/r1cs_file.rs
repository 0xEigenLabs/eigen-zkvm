// some codes borrowed from https://github.com/poma/zkutil/blob/master/src/r1cs_reader.rs
// Implement of https://github.com/iden3/r1csfile/blob/master/doc/r1cs_bin_format.md
#![allow(unused_variables, dead_code, non_snake_case)]
use crate::bellman_ce::{Field, PrimeField, PrimeFieldRepr, ScalarEngine};
use crate::circom_circuit::{Constraint, CustomGates, CustomGatesUses};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom},
};

// R1CSFile's header
#[derive(Debug, Default)]
pub struct Header {
    pub field_size: u32,
    pub prime_size: Vec<u8>,
    pub n_wires: u32,
    pub n_pub_out: u32,
    pub n_pub_in: u32,
    pub n_prv_in: u32,
    pub n_labels: u64,
    pub n_constraints: u32,
    pub use_custom_gates: bool,
}

// R1CSFile parse result
#[derive(Debug, Default)]
pub struct R1CSFile<E: ScalarEngine> {
    pub version: u32,
    pub header: Header,
    pub constraints: Vec<Constraint<E>>,
    pub wire_mapping: Vec<u64>,
    pub custom_gates: Vec<CustomGates<E>>,
    pub custom_gates_uses: Vec<CustomGatesUses>,
}

fn read_field<R: Read, E: ScalarEngine>(mut reader: R) -> Result<E::Fr> {
    let mut repr = E::Fr::zero().into_repr();
    repr.read_le(&mut reader)?;
    let fr = E::Fr::from_repr(repr).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    Ok(fr)
}

const HEADER_TYPE: u32 = 1;
const CONSTRAINT_TYPE: u32 = 2;
const WIRE2LABEL_TYPE: u32 = 3;
const CUSTOM_GATES_LIST: u32 = 4;
const CUSTOM_GATES_USE: u32 = 5;

fn read_header<R: Read>(mut reader: R, size: u64) -> Result<Header> {
    let field_size = reader.read_u32::<LittleEndian>()?;
    let mut prime_size = vec![0u8; field_size as usize];
    reader.read_exact(&mut prime_size)?;
    if size != 32 + field_size as u64 {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid header section size"));
    }

    Ok(Header {
        field_size,
        prime_size,
        n_wires: reader.read_u32::<LittleEndian>()?,
        n_pub_out: reader.read_u32::<LittleEndian>()?,
        n_pub_in: reader.read_u32::<LittleEndian>()?,
        n_prv_in: reader.read_u32::<LittleEndian>()?,
        n_labels: reader.read_u64::<LittleEndian>()?,
        n_constraints: reader.read_u32::<LittleEndian>()?,
        use_custom_gates: false,
    })
}

fn read_constraint_vec<R: Read, E: ScalarEngine>(
    mut reader: R,
    header: &Header,
) -> Result<Vec<(usize, E::Fr)>> {
    let n_vec = reader.read_u32::<LittleEndian>()? as usize;
    let mut vec = Vec::with_capacity(n_vec);
    for _ in 0..n_vec {
        vec.push((
            reader.read_u32::<LittleEndian>()? as usize,
            read_field::<&mut R, E>(&mut reader)?,
        ));
        // sort by key
        vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }
    Ok(vec)
}

fn read_constraints<R: Read, E: ScalarEngine>(
    mut reader: R,
    size: u64,
    header: &Header,
) -> Result<Vec<Constraint<E>>> {
    // todo check section size
    let mut vec = Vec::with_capacity(header.n_constraints as usize);
    for _ in 0..header.n_constraints {
        vec.push((
            read_constraint_vec::<&mut R, E>(&mut reader, header)?,
            read_constraint_vec::<&mut R, E>(&mut reader, header)?,
            read_constraint_vec::<&mut R, E>(&mut reader, header)?,
        ));
    }
    Ok(vec)
}

fn read_map<R: Read>(mut reader: R, size: u64, header: &Header) -> Result<Vec<u64>> {
    if size != header.n_wires as u64 * 8 {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid map section size"));
    }
    let mut vec = Vec::with_capacity(header.n_wires as usize);
    for _ in 0..header.n_wires {
        vec.push(reader.read_u64::<LittleEndian>()?);
    }
    if vec[0] != 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Wire 0 should always be mapped to 0"));
    }
    Ok(vec)
}

// TODO: why does the `read_to_end` not work?
fn read_to_string<R: Read>(mut reader: R) -> String {
    let mut name_buf = vec![1u8; 1];
    let mut buf = vec![];
    loop {
        let name_size_res = reader.read_exact(&mut name_buf);
        if name_buf[0] != 0 {
            buf.push(name_buf[0]);
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&buf).to_string()
}

fn read_custom_gates_list<R: Read, E: ScalarEngine>(
    mut reader: R,
    size: u64,
    header: &Header,
) -> Result<Vec<CustomGates<E>>> {
    let num = reader.read_u32::<LittleEndian>()?;
    let mut custom_gates: Vec<CustomGates<E>> = vec![];
    for i in 0..num {
        let mut custom_gate =
            CustomGates::<E> { template_name: read_to_string(&mut reader), parameters: vec![] };
        let num_parameters = reader.read_u32::<LittleEndian>()?;
        for _i in 0..num_parameters {
            custom_gate.parameters.push(read_field::<&mut R, E>(&mut reader)?);
        }
        custom_gates.push(custom_gate);
    }
    Ok(custom_gates)
}

fn read_custom_gates_uses_list<R: Read>(
    mut reader: R,
    size: u64,
    header: &Header,
) -> Result<Vec<CustomGatesUses>> {
    let mut custom_gates_uses: Vec<CustomGatesUses> = vec![];

    let sz = size as usize / 4;
    let mut b_r1cs32 = Vec::with_capacity(sz);
    for _ in 0..sz {
        b_r1cs32.push(reader.read_u32::<LittleEndian>()?);
    }

    let n_custom_gate_uses = b_r1cs32[0];
    let mut b_r1cs_pos = 1;
    for i in 0..n_custom_gate_uses {
        let mut c = CustomGatesUses { id: b_r1cs32[b_r1cs_pos] as u64, ..Default::default() };
        b_r1cs_pos += 1;
        let num_signals = b_r1cs32[b_r1cs_pos];
        b_r1cs_pos += 1;
        for j in 0..num_signals {
            let LSB = b_r1cs32[b_r1cs_pos] as u64;
            b_r1cs_pos += 1;
            let MSB = b_r1cs32[b_r1cs_pos] as u64;
            b_r1cs_pos += 1;
            c.signals.push(MSB * 0x100000000u64 + LSB);
        }
        custom_gates_uses.push(c);
    }
    Ok(custom_gates_uses)
}

pub fn from_reader<R: Read + Seek, E: ScalarEngine>(mut reader: R) -> Result<R1CSFile<E>> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != [0x72, 0x31, 0x63, 0x73] {
        // magic = "r1cs"
        return Err(Error::new(ErrorKind::InvalidData, "Invalid magic number"));
    }

    let version = reader.read_u32::<LittleEndian>()?;
    if version != 1 {
        return Err(Error::new(ErrorKind::InvalidData, "Unsupported version"));
    }

    let num_sections = reader.read_u32::<LittleEndian>()?;

    // section type -> file offset
    let mut section_offsets = BTreeMap::<u32, u64>::new();
    let mut section_sizes = BTreeMap::<u32, u64>::new();

    // get file offset of each section, we donot support custom gate yet, so ignore the
    // last two sections.
    for i in 0..(num_sections) {
        let section_type = reader.read_u32::<LittleEndian>()?;
        let section_size = reader.read_u64::<LittleEndian>()?;
        let offset = reader.stream_position()?;
        section_offsets.insert(section_type, offset);
        section_sizes.insert(section_type, section_size);
        reader.seek(SeekFrom::Current(section_size as i64))?;
    }

    reader.seek(SeekFrom::Start(*section_offsets.get(&HEADER_TYPE).unwrap()))?;
    let mut header = read_header(&mut reader, *section_sizes.get(&HEADER_TYPE).unwrap())?;
    if section_offsets.contains_key(&CUSTOM_GATES_USE)
        && section_offsets.contains_key(&CUSTOM_GATES_LIST)
    {
        header.use_custom_gates = true;
    }
    if !(header.field_size == 32 || header.field_size == 8) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "This parser only supports 32-bytes or 8-bytes fields",
        ));
    }
    if header.field_size != E::Fr::NUM_BITS.div_ceil(8) {
        return Err(Error::new(ErrorKind::InvalidData, "Different prime"));
    }
    if !(header.prime_size
        == hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430")
        || header.prime_size
            == hex!("01000000fffffffffe5bfeff02a4bd5305d8a10908d83933487d9d2953a7ed73")
        || header.prime_size == hex!("01000000ffffffff"))
    {
        return Err(Error::new(ErrorKind::InvalidData, "This parser only supports bn256 or GL"));
    }
    reader.seek(SeekFrom::Start(*section_offsets.get(&CONSTRAINT_TYPE).unwrap()))?;
    let constraints = read_constraints::<&mut R, E>(
        &mut reader,
        *section_sizes.get(&CONSTRAINT_TYPE).unwrap(),
        &header,
    )?;

    reader.seek(SeekFrom::Start(*section_offsets.get(&WIRE2LABEL_TYPE).unwrap()))?;
    let wire_mapping =
        read_map(&mut reader, *section_sizes.get(&WIRE2LABEL_TYPE).unwrap(), &header)?;
    let mut custom_gates: Vec<CustomGates<E>> = vec![];
    if section_offsets.contains_key(&CUSTOM_GATES_LIST) {
        reader.seek(SeekFrom::Start(*section_offsets.get(&CUSTOM_GATES_LIST).unwrap()))?;
        custom_gates = read_custom_gates_list(
            &mut reader,
            *section_sizes.get(&CUSTOM_GATES_LIST).unwrap(),
            &header,
        )?;
    }

    let mut custom_gates_uses: Vec<CustomGatesUses> = vec![];
    if section_offsets.contains_key(&CUSTOM_GATES_USE) {
        reader.seek(SeekFrom::Start(*section_offsets.get(&CUSTOM_GATES_USE).unwrap()))?;
        custom_gates_uses = read_custom_gates_uses_list(
            &mut reader,
            *section_sizes.get(&CUSTOM_GATES_USE).unwrap(),
            &header,
        )?;
    }

    Ok(R1CSFile { version, header, constraints, wire_mapping, custom_gates, custom_gates_uses })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bellman_ce::pairing::bn256::Bn256;
    use crate::bellman_ce::pairing::ff;
    use std::io::{BufReader, Cursor};

    #[test]
    fn sample() {
        let data = hex!(
            "
        72316373
        01000000
        03000000
        01000000 40000000 00000000
        20000000
        010000f0 93f5e143 9170b979 48e83328 5d588181 b64550b8 29a031e1 724e6430
        07000000
        01000000
        02000000
        03000000
        e8030000 00000000
        03000000
        02000000 88020000 00000000
        02000000
        05000000 03000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        06000000 08000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        03000000
        00000000 02000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        02000000 14000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        03000000 0C000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        02000000
        00000000 05000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        02000000 07000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        03000000
        01000000 04000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        04000000 08000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        05000000 03000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        02000000
        03000000 2C000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        06000000 06000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        00000000
        01000000
        06000000 04000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        03000000
        00000000 06000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        02000000 0B000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        03000000 05000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        01000000
        06000000 58020000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        03000000 38000000 00000000
        00000000 00000000
        03000000 00000000
        0a000000 00000000
        0b000000 00000000
        0c000000 00000000
        0f000000 00000000
        44010000 00000000
    "
        );

        let reader = BufReader::new(Cursor::new(&data[..]));
        let file = from_reader::<_, Bn256>(reader).unwrap();
        assert_eq!(file.version, 1);

        assert_eq!(file.header.field_size, 32);
        assert_eq!(
            file.header.prime_size,
            &hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430")
        );
        assert_eq!(file.header.n_wires, 7);
        assert_eq!(file.header.n_pub_out, 1);
        assert_eq!(file.header.n_pub_in, 2);
        assert_eq!(file.header.n_prv_in, 3);
        assert_eq!(file.header.n_labels, 0x03e8);
        assert_eq!(file.header.n_constraints, 3);

        assert_eq!(file.constraints.len(), 3);
        assert_eq!(file.constraints[0].0.len(), 2);
        assert_eq!(file.constraints[0].0[0].0, 5);
        assert_eq!(file.constraints[0].0[0].1, ff::from_hex("0x03").unwrap());
        assert_eq!(file.constraints[2].1[0].0, 0);
        assert_eq!(file.constraints[2].1[0].1, ff::from_hex("0x06").unwrap());
        assert_eq!(file.constraints[1].2.len(), 0);

        assert_eq!(file.wire_mapping.len(), 7);
        assert_eq!(file.wire_mapping[1], 3);
    }

    #[test]
    fn test_reader_size_fail() {
        // fn read_header<R: Read>(mut reader: R, size: u64) -> Result<Header>
        let mut buf: Vec<u8> = 32_u32.to_le_bytes().to_vec();
        buf.resize(4 + 32, 0);
        let err = read_header(&mut buf.as_slice(), 32).err().unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidData)
    }
}
