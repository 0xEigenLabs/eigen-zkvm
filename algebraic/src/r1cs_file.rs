// some codes borrowed from https://github.com/poma/zkutil/blob/master/src/r1cs_reader.rs
// Implement of https://github.com/iden3/r1csfile/blob/master/doc/r1cs_bin_format.md
#![allow(unused_variables, dead_code, non_snake_case)]

pub mod constraint;
pub mod custom_gate;
pub mod header;

use crate::bellman_ce::{Field, PrimeField, PrimeFieldRepr, ScalarEngine};
use crate::r1cs_file::{
    constraint::{read_constraint_vec, read_constraints, Constraint},
    custom_gate::{CustomGates, CustomGatesUses},
    header::Header,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom},
};

const HEADER_TYPE: u32 = 1;
const CONSTRAINT_TYPE: u32 = 2;
const WIRE2LABEL_TYPE: u32 = 3;
const CUSTOM_GATES_LIST: u32 = 4;
const CUSTOM_GATES_USE: u32 = 5;

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

impl<E: ScalarEngine> R1CSFile<E> {
    fn read_map<R: Read>(mut reader: R, size: u64, header: &Header) -> Result<Vec<u64>> {
        if size != header.n_wires as u64 * 8 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid map section size",
            ));
        }
        let mut vec = Vec::with_capacity(header.n_wires as usize);
        for _ in 0..header.n_wires {
            vec.push(reader.read_u64::<LittleEndian>()?);
        }
        if vec[0] != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Wire 0 should always be mapped to 0",
            ));
        }
        Ok(vec)
    }

    pub fn from_reader<R: Read + Seek, E: ScalarEngine>(mut reader: R) -> Result<R1CSFile<E>> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != [0x72, 0x31, 0x63, 0x73] {
            // magic = "r1cs_file"
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
        let mut header =
            Header::read_header(&mut reader, *section_sizes.get(&HEADER_TYPE).unwrap())?;
        if section_offsets.get(&CUSTOM_GATES_USE).is_some()
            && section_offsets.get(&CUSTOM_GATES_LIST).is_some()
        {
            header.use_custom_gates = true;
        }
        if !(header.field_size == 32 || header.field_size == 8) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "This parser only supports 32-bytes or 8-bytes fields",
            ));
        }
        if header.field_size != (E::Fr::NUM_BITS + 7) / 8 {
            return Err(Error::new(ErrorKind::InvalidData, "Different prime"));
        }
        if !(header.prime_size
            == hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430")
            || header.prime_size
                == hex!("01000000fffffffffe5bfeff02a4bd5305d8a10908d83933487d9d2953a7ed73")
            || header.prime_size == hex!("01000000ffffffff"))
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "This parser only supports bn256 or GL",
            ));
        }
        reader.seek(SeekFrom::Start(
            *section_offsets.get(&CONSTRAINT_TYPE).unwrap(),
        ))?;
        let constraints = read_constraints::<&mut R, E>(
            &mut reader,
            *section_sizes.get(&CONSTRAINT_TYPE).unwrap(),
            &header,
        )?;

        reader.seek(SeekFrom::Start(
            *section_offsets.get(&WIRE2LABEL_TYPE).unwrap(),
        ))?;
        let wire_mapping = Self::read_map(
            &mut reader,
            *section_sizes.get(&WIRE2LABEL_TYPE).unwrap(),
            &header,
        )?;
        let mut custom_gates: Vec<CustomGates<E>> = vec![];
        if section_offsets.get(&CUSTOM_GATES_LIST).is_some() {
            reader.seek(SeekFrom::Start(
                *section_offsets.get(&CUSTOM_GATES_LIST).unwrap(),
            ))?;
            custom_gates = CustomGates::read_custom_gates_list(
                &mut reader,
                *section_sizes.get(&CUSTOM_GATES_LIST).unwrap(),
                &header,
            )?;
        }

        let mut custom_gates_uses: Vec<CustomGatesUses> = vec![];
        if section_offsets.get(&CUSTOM_GATES_USE).is_some() {
            reader.seek(SeekFrom::Start(
                *section_offsets.get(&CUSTOM_GATES_USE).unwrap(),
            ))?;
            custom_gates_uses = CustomGatesUses::read_custom_gates_uses_list(
                &mut reader,
                *section_sizes.get(&CUSTOM_GATES_USE).unwrap(),
                &header,
            )?;
        }

        Ok(R1CSFile {
            version,
            header,
            constraints,
            wire_mapping,
            custom_gates,
            custom_gates_uses,
        })
    }
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
        let file = R1CSFile::from_reader::<_, Bn256>(reader).unwrap();
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
        let err = Header::read_header(&mut buf.as_slice(), 32).err().unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidData)
    }
}
