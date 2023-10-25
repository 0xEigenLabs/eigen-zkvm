use crate::bellman_ce::ScalarEngine;
use crate::io_utils::read_field;
use crate::r1cs_file::header::Header;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Result};

// R1CSfile's CustomGates
#[derive(Debug, Default, Clone)]
pub struct CustomGates<E: ScalarEngine> {
    pub template_name: String,
    pub parameters: Vec<E::Fr>,
}

impl<E: ScalarEngine> CustomGates<E> {
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

    pub fn read_custom_gates_list<R: Read>(
        mut reader: R,
        size: u64,
        header: &Header,
    ) -> Result<Vec<CustomGates<E>>> {
        let num = reader.read_u32::<LittleEndian>()?;
        let mut custom_gates: Vec<CustomGates<E>> = vec![];
        for i in 0..num {
            let mut custom_gate = CustomGates::<E> {
                template_name: Self::read_to_string(&mut reader),
                parameters: vec![],
            };
            let num_parameters = reader.read_u32::<LittleEndian>()?;
            for _i in 0..num_parameters {
                custom_gate
                    .parameters
                    .push(read_field::<&mut R, E>(&mut reader)?);
            }
            custom_gates.push(custom_gate);
        }
        Ok(custom_gates)
    }
}

// R1CSfile's CustomGatesUses
#[derive(Debug, Default, Clone)]
pub struct CustomGatesUses {
    pub id: u64,
    pub signals: Vec<u64>,
}
impl CustomGatesUses {
    pub fn read_custom_gates_uses_list<R: Read>(
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
            let mut c = CustomGatesUses {
                id: b_r1cs32[b_r1cs_pos] as u64,
                ..Default::default()
            };
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
}
