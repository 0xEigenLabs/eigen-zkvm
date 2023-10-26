use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Error, ErrorKind, Read, Result};

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

impl Header {
    pub(crate) fn read_header<R: Read>(mut reader: R, size: u64) -> Result<Header> {
        let field_size = reader.read_u32::<LittleEndian>()?;
        let mut prime_size = vec![0u8; field_size as usize];
        reader.read_exact(&mut prime_size)?;
        if size != 32 + field_size as u64 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid header section size",
            ));
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
}
