#![allow(non_snake_case)]
use crate::errors::Result;
use crate::f3g::F3G;
use crate::types::PIL;
use plonky::field_gl::Fr as FGL;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

fn fgl_pretty_print<S>(value: &Vec<Vec<FGL>>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seqs = serializer.serialize_seq(Some(value.len()))?;
    for v in value {
        let mut va = vec![0u64; v.len()];
        for (i, vv) in v.iter().enumerate() {
            va[i] = vv.as_int();
        }
        seqs.serialize_element(&va);
    }
    seqs.end()
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PolsArray {
    pub nPols: usize,
    // nameSpace, namePol, defArray's index,
    pub def: HashMap<String, HashMap<String, Vec<usize>>>,
    pub defArray: Vec<Pol>,
    #[serde(serialize_with = "fgl_pretty_print")]
    pub array: Vec<Vec<FGL>>,
    pub n: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pol {
    pub name: String,
    pub id: usize,
    pub idx: Option<usize>,
    pub polDeg: usize,
    pub elementType: Option<String>, // "field, s8, s16, s32, s64, u16, u8"
}

#[derive(Eq, PartialEq)]
pub enum PolKind {
    Commit,
    Constant,
}

impl PolsArray {
    pub fn new(pil: &PIL, kind: PolKind) -> Self {
        let nPols = match kind {
            PolKind::Commit => pil.nCommitments,
            PolKind::Constant => pil.nConstants,
        };

        let mut def: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();
        let mut defArray: Vec<Pol> = vec![Pol::default(); nPols];
        let mut array: Vec<Vec<FGL>> = vec![Vec::new(); nPols];
        for i in 0..array.len() {
            array[i] = vec![FGL::default(); nPols];
        }
        for (refName, ref_) in pil.references.iter() {
            if (ref_.type_ == "cmP" && kind == PolKind::Commit)
                || (ref_.type_ == "constP" && kind == PolKind::Constant)
            {
                let name_vec: Vec<&str> = refName.split('.').collect();
                let nameSpace = name_vec[0].to_string();
                let namePols = name_vec[1].to_string();

                if ref_.isArray {
                    let mut ns: HashMap<String, Vec<usize>> = HashMap::new();
                    let mut arrayPols: Vec<usize> = vec![0usize; ref_.len.unwrap()];
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
                            idx: Some(i),
                            elementType: match &ref_.elementType {
                                Some(x) => Some(x.clone()),
                                None => None,
                            },
                            polDeg: ref_.polDeg,
                        };
                        arrayPols[i] = ref_.id + i;
                        array[ref_.id + i] = vec![FGL::default(); ref_.polDeg];
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
                    let arrayPols: Vec<usize> = vec![ref_.id];
                    let mut ns: HashMap<String, Vec<usize>> = HashMap::new();
                    ns.insert(namePols, arrayPols);
                    def.insert(nameSpace, ns);
                    array[ref_.id] = vec![FGL::default(); ref_.polDeg];
                }
            }
        }

        for i in 0..nPols {
            if defArray[i].name.len() == 0 {
                panic!("Invalid pils sequence");
            }
        }

        PolsArray {
            nPols: defArray.len(),
            n: defArray[0].polDeg,
            defArray,
            array,
            def,
        }
    }

    #[inline(always)]
    pub fn get(&self, pil: &PIL, ns: &String, np: &String, i: usize, j: usize) -> FGL {
        let ref_id = self.get_pol_id(pil, ns, np, i);
        self.array[ref_id][j].clone()
    }

    /// Set the ns.np[i][j] = value, where ns is the namespace, np is the state variable, i is
    /// the i-th sub-variable of state np, and j is the i-row of np.
    ///
    /// e.g. For JS statement, constPols.Compressor.C[7][pr.row] = c[5], i is 7 and j is pr.row.
    ///
    /// Before calling this function, you must ensure that this polsarray has been initialized
    #[inline(always)]
    pub fn set_matrix(
        &mut self,
        pil: &PIL,
        ns: &String,
        np: &String,
        i: usize,
        j: usize,
        value: FGL,
    ) {
        let ref_id = self.get_pol_id(pil, ns, np, i);
        self.array[ref_id][j] = value;
    }
    pub fn get_pol_id(&self, pil: &PIL, ns: &String, np: &String, k: usize) -> usize {
        let pol = &pil.references[&format!("{}.{}", ns, np)];
        pol.id + k
    }

    #[inline(always)]
    pub(crate) fn get_np_index_of_array(&mut self, ns: &String, np: &String, i: usize) -> usize {
        let namespace = self.def.get(ns).unwrap();
        let namepols = namespace.get(np).unwrap();
        let np_id = namepols[i];
        np_id
    }

    #[inline(always)]
    pub fn set_array(&mut self, ns: &String, np: &String, i: usize, value: FGL) {
        self.set_matrix(ns, np, i, 0, value);
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
            log::info!(
                "loading {:?}.. {:?} of {}",
                fileName,
                k / 1024 / 1024,
                totalSize / 1024 / 1024
            );
            let mut n = std::cmp::min(buff8.len() / 8, totalSize - k);
            let rs = f.read(&mut buff8[..(n * 8)])?;
            log::info!(
                "read size: read size = {}, n = {}, k = {}, totalSize = {}",
                rs,
                n,
                k,
                totalSize
            );
            let buff: &[u64] = unsafe {
                std::slice::from_raw_parts(
                    buff8.as_ptr() as *const u64,
                    buff8.len() / std::mem::size_of::<u64>(),
                )
            };
            n = rs / 8;
            for l in 0..n {
                self.array[i][j] = FGL::from(buff[l]);
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

    pub fn write_buff(&self) -> Vec<F3G> {
        let mut buff: Vec<F3G> = vec![];
        for i in 0..self.n {
            for j in 0..self.nPols {
                buff.push(F3G::from(self.array[j][i]));
            }
        }
        buff
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::compressor12_pil::render;
    use crate::pilcom::compile_pil_from_str;
    use crate::types::{self, PIL};
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_load_polsarray() {
        let pil = types::load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut cp = PolsArray::new(&pil, PolKind::Constant);
        cp.load("data/fib.const").unwrap();
        cp.save("data/fib.const.cp").unwrap();

        let mut cmp = PolsArray::new(&pil, PolKind::Commit);
        cmp.load("data/fib.exec").unwrap();
        cmp.save("data/fib.exec.cp").unwrap();
    }

    #[test]
    fn test_dump_pols_array() {
        let pil_string = render(5, 5);

        let pil_json = compile_pil_from_str(&pil_string);

        let pols_array = PolsArray::new(&pil_json, PolKind::Constant);

        let input = serde_json::to_string_pretty(&pols_array).unwrap();
        let mut file = File::create(Path::new("./test_pols_array.json")).unwrap();
        write!(file, "{}", input);
    }
}
