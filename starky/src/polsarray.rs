#![allow(non_snake_case)]
use crate::types::PIL;
use std::collections::HashMap;
use std::fs::File;

use std::io::{Read, Write};
use winter_math::StarkField;

use crate::errors::Result;
use winter_math::fields::f64::BaseElement;

#[derive(Default, Debug)]
pub struct PolsArray {
    pub nPols: usize,
    // nameSpace, namePol, defArray's index,
    pub def: HashMap<String, HashMap<String, (bool, Vec<usize>)>>,
    pub defArray: Vec<Pol>,
    pub array: Vec<Vec<BaseElement>>,
    pub n: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Pol {
    pub name: String,
    pub id: usize,
    pub idx: Option<i32>,
    pub polDeg: usize,
    pub elementType: Option<String>, // "field, s8, s16, s32, s64, u16, u8"
}

#[derive(Eq, PartialEq)]
pub enum PolKind {
    Commit,
    Constant,
}

pub type ArrayPol = (bool, Vec<usize>);

impl PolsArray {
    pub fn new(pil: &PIL, kind: PolKind, defSize: usize) -> Self {
        let nPols = match kind {
            PolKind::Commit => pil.nCommitments,
            PolKind::Constant => pil.nConstants,
        };

        let mut def: HashMap<String, HashMap<String, ArrayPol>> = HashMap::new();
        let mut defArray: Vec<Pol> = vec![Pol::default(); nPols as usize];
        let mut array: Vec<Vec<BaseElement>> = vec![Vec::new(); nPols as usize];
        for i in 0..array.len() {
            array[i] = vec![BaseElement::default(); nPols as usize];
        }
        println!("reference {:?}", pil.references);
        for (refName, ref_) in pil.references.iter() {
            if (ref_.type_ == "cmP" && kind == PolKind::Commit)
                || (ref_.type_ == "constP" && kind == PolKind::Constant)
            {
                let name_vec: Vec<&str> = refName.split('.').collect();
                println!("name_vec {:?}", name_vec);
                let nameSpace = name_vec[0].to_string();
                let namePols = name_vec[1].to_string();

                if ref_.isArray {
                    let mut ns: HashMap<String, ArrayPol> = HashMap::new();
                    let mut arrayPols: ArrayPol = (true, vec![0usize; ref_.len.unwrap()]);
                    if def.contains_key(&nameSpace) {
                        ns = def.get(&nameSpace).unwrap().clone();
                        if ns.contains_key(&namePols) {
                            arrayPols = ns.get(&namePols).unwrap().clone();
                        }
                    }

                    for i in 0..ref_.len.unwrap() {
                        defArray[ref_.id + i] = Pol {
                            name: refName.clone(),
                            id: ref_.id + i,
                            idx: Some(i as i32),
                            elementType: match &ref_.elementType {
                                Some(x) => Some(x.clone()),
                                None => None,
                            },
                            polDeg: ref_.polDeg,
                        };
                        arrayPols.1[i] = ref_.id + i;
                        array[ref_.id + i] = vec![BaseElement::default(); ref_.polDeg];
                    }
                    ns.insert(namePols, arrayPols);
                    def.insert(nameSpace, ns);
                } else {
                    defArray[ref_.id] = Pol {
                        name: refName.clone(),
                        id: ref_.id,
                        idx: None,
                        elementType: match &ref_.elementType {
                            Some(x) => Some(x.clone()),
                            None => None,
                        },
                        polDeg: ref_.polDeg,
                    };
                    let arrayPols: ArrayPol = (false, vec![ref_.id]);
                    let mut ns: HashMap<String, ArrayPol> = HashMap::new();
                    ns.insert(namePols, arrayPols);
                    def.insert(nameSpace, ns);
                    array[ref_.id] = vec![BaseElement::default(); ref_.polDeg];
                }
            }
        }

        for i in 0..nPols {
            if defArray[i as usize].name.len() == 0 {
                panic!("Invalid pils sequence");
            }
        }

        PolsArray {
            nPols: defArray.len(),
            n: defArray[0].polDeg,
            defArray: defArray,
            array: array,
            def: def,
        }
    }

    pub fn load(&mut self, fileName: &str) -> Result<()> {
        let mut f = File::open(fileName)?;
        let maxBufferSize = 1024 * 1024 * 32;
        let totalSize = self.nPols * self.n;
        let mut buff8: Vec<u8> = vec![0u8; std::cmp::min(totalSize, maxBufferSize) * 8];

        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        while k < totalSize {
            println!(
                "loading {:?}.. {:?} of {}",
                fileName,
                k / 1024 / 1024,
                totalSize / 1024 / 1204
            );
            let mut n = std::cmp::min(buff8.len() / 8, totalSize - k);
            let rs = f.read(&mut buff8[..(n * 8)])?;
            println!("read size: {} {} totalSize {}", rs, n, totalSize);
            let buff: &[u64] = unsafe {
                std::slice::from_raw_parts(
                    buff8.as_ptr() as *const u64,
                    buff8.len() / std::mem::size_of::<u64>(),
                )
            };
            n = rs / 8;
            for l in 0..n {
                self.array[i][j] = BaseElement::from(buff[l]);
                i += 1;
                if i == self.nPols {
                    i = 0;
                    j += 1;
                }
            }
            k += n;
        }

        Ok(())
    }

    pub fn save(&self, fileName: &str) -> Result<()> {
        let mut writer = File::create(fileName)?;
        let maxBufferSize = 1024 * 1024 * 32;
        let totalSize = self.nPols * self.n;
        let mut buff: Vec<u64> = vec![0u64; std::cmp::min(totalSize, maxBufferSize)];

        let mut p = 0usize;
        for i in 0..self.n {
            for j in 0..self.nPols {
                buff[p] = self.array[j][i].as_int() % 0xFFFFFFFF00000001; //u128
                p += 1;
                if p == buff.capacity() {
                    // copy to [u8]
                    let buff8: &[u8] = unsafe {
                        std::slice::from_raw_parts(
                            buff.as_ptr() as *const u8,
                            buff.len() * std::mem::size_of::<u64>(),
                        )
                    };
                    writer.write(&buff8)?;
                    p = 0;
                }
            }
        }
        if p > 0 {
            let pb: &[u64] = &buff[..p];
            let buff8: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    buff.as_ptr() as *const u8,
                    buff.len() * std::mem::size_of::<u64>(),
                )
            };
            writer.write(&buff8)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::polsarray::{PolKind, PolsArray};
    use crate::types::{self, PIL};
    use std::collections::HashMap;
    #[test]
    fn test_load_polsarray() {
        let pil = types::load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut cp = PolsArray::new(&pil, PolKind::Constant, 32);
        cp.load("data/fib.const").unwrap();
        cp.save("data/fib.const.cp").unwrap();

        let mut cmp = PolsArray::new(&pil, PolKind::Commit, 32);
        cmp.load("data/fib.exec").unwrap();
        cmp.save("data/fib.exec.cp").unwrap();
    }
}
