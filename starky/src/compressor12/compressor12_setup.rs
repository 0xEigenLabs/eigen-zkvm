#![allow(non_snake_case)]
#![allow(dead_code)]
use crate::compressor12::compressor12_pil;
use crate::r1cs2plonk::{r1cs2plonk, PlonkAdd, PlonkGate};
use crate::{pilcom, polsarray};
use plonky::circom_circuit::R1CS;
use plonky::field_gl::Fr as FGL;
use plonky::field_gl::GL;
use plonky::reader::load_r1cs;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub struct Options {
    pub force_bits: usize,
}

#[derive(Default, Debug)]
pub struct Compressor {
    pub Qm: Vec<FGL>,
    pub Ql: Vec<FGL>,
    pub Qr: Vec<FGL>,
    pub Qo: Vec<FGL>,
    pub Qk: Vec<FGL>,
    pub QCMul: Vec<FGL>,
    pub QMDS: Vec<FGL>,
    pub S: Vec<Vec<FGL>>,
}

impl Compressor {
    pub fn new(sz: usize) -> Self {
        Compressor {
            Qm: vec![FGL::ZERO; sz],
            Ql: vec![FGL::ZERO; sz],
            Qr: vec![FGL::ZERO; sz],
            Qo: vec![FGL::ZERO; sz],
            Qk: vec![FGL::ZERO; sz],
            QCMul: vec![FGL::ZERO; sz],
            QMDS: vec![FGL::ZERO; sz],
            S: vec![Vec::new(); sz],
        }
    }
}

struct NormalPlonkInfo {
    pub N: usize,
    pub nConstaints: usize,
    pub nPlonkGates: usize,
    pub nPlonkAdds: usize,
}

fn get_normal_plonkinfo(
    r1cs: &R1CS<GL>,
    pa: &Vec<PlonkGate>,
    pc: &Vec<PlonkAdd>,
) -> NormalPlonkInfo {
    let mut uses: HashMap<String, usize> = HashMap::new();
    for (i, c) in pa.iter().enumerate() {
        if (i % 10000) == 0 {
            println!("Plonk info constraint processing... {}/{}", i, pa.len());
        }
        let k = c.str_key();
        if uses.get(&k).is_none() {
            uses.insert(k.clone(), 0);
        }
        *uses.get_mut(&k).unwrap() += 1;
    }
    let mut result = uses
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<Vec<(String, usize)>>();
    result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut N = 0;
    result.iter().for_each(|e| {
        N += (e.1 - 1) / 4 + 1;
    });

    NormalPlonkInfo {
        N,
        nConstaints: r1cs.constraints.len(),
        nPlonkGates: pa.len(),
        nPlonkAdds: pc.len(),
    }
}

struct CustomGateInfo {
    pub cmul_id: u64,
    pub cmds_id: u64,
    pub n_cmul: u64,
    pub n_mds: u64,
}

fn get_custom_gate_info(
    r1cs: &R1CS<GL>,
    _pa: &Vec<PlonkGate>,
    _pc: &Vec<PlonkAdd>,
) -> CustomGateInfo {
    let mut cmul_id = 0;
    let mut cmds_id = 0;
    let mut bcmul = false;
    let mut bcmds = false;
    assert_eq!(r1cs.custom_gates.len(), 2);
    for (i, c) in r1cs.custom_gates.iter().enumerate() {
        match c.template_name.as_str() {
            "CMul" => {
                cmul_id = i as u64;
                bcmul = true;
                assert!(c.parameters.len() == 0);
            }
            "MDS" => {
                cmds_id = i as u64;
                bcmds = true;
                assert!(c.parameters.len() == 0);
            }
            _ => panic!("Invalid custom gate {}", c.template_name),
        }
    }
    if !bcmul {
        panic!("CMul custom gate not defined");
    }
    if !bcmds {
        panic!("cmds_id custom gate not defined");
    }

    let mut n_cmul = 0;
    let mut n_mds = 0;
    for (_i, c) in r1cs.custom_gates_uses.iter().enumerate() {
        if c.id == cmul_id {
            n_cmul += 1;
        } else if c.id == cmds_id {
            n_mds += 1;
        } else {
            panic!("Custom gate not defined {}", c.id);
        }
    }
    CustomGateInfo {
        cmul_id,
        cmds_id,
        n_cmul,
        n_mds,
    }
}

pub struct PlonkSetupInfo {
    n_used: usize,
    n_bits: usize,
    n_publics: usize,
    pg: Vec<PlonkGate>,
    pa: Vec<PlonkAdd>,
    custom_gates_info: CustomGateInfo,
    plonkinfo: NormalPlonkInfo,
}

pub fn plonk_setup_render(r1cs: &R1CS<GL>, opts: &Options, out_pil: &str) -> PlonkSetupInfo {
    let pc = r1cs2plonk(r1cs);
    let plonkinfo = get_normal_plonkinfo(r1cs, &pc.0, &pc.1);
    let custom_gates_info = get_custom_gate_info(r1cs, &pc.0, &pc.1);

    let n_publics = r1cs.num_inputs + r1cs.num_outputs;
    let n_public_rows = (n_publics - 1) / 12 + 1;

    let n_used = n_public_rows
        + plonkinfo.N as usize
        + custom_gates_info.n_cmul as usize
        + custom_gates_info.n_mds as usize * 2;
    let mut n_bits = crate::helper::log2_any(n_used - 1) + 1;
    if opts.force_bits > 0 {
        n_bits = opts.force_bits;
    }
    let com_pil = compressor12_pil::render(n_bits, n_publics);
    let mut file = File::create(out_pil).unwrap();
    write!(file, "{}", com_pil).unwrap();

    // let pil = crate::pilcom::compile(com_pil);
    // let const_pols = PolsArray::new();

    PlonkSetupInfo {
        n_used,
        n_bits,
        n_publics,
        pg: pc.0,
        pa: pc.1,
        custom_gates_info,
        plonkinfo,
    }
}

// pub fn setup(circuit_file: &String, opts: &Options, out_pil: &str) -> Result<()> {
//     // // a.generate plonk circuit pil file.
//     // // b.compile(pil) to construct .cm file.
//     // const res = await plonkSetup(r1cs, options);
//     //
//
//     let r1cs = load_r1cs::<GL>(circuit_file);
//     let (pc, pa) = r1cs2plonk(&r1cs);
//     println!("pc {}, pa {}", pc.len(), pa.len());
//     let opts = Options { force_bits: 0 };
//     let plonksetupinfo = plonk_setup_render(&r1cs, &opts, out_pil);
//
//     // await fs.promises.writeFile(pilFile, res.pilStr, "utf8");
//     //
//     // await res.constPols.saveToFile(constFile);
//     //
//     // await writeExecFile(execFile,res.plonkAdditions,  res.sMap);
//
//     let ser_proof_str = serde_json::to_string_pretty(&serialized_proof)?;
//     let ser_inputs_str = serde_json::to_string_pretty(&inputs)?;
//
//     std::fs::write(proof_json, ser_proof_str.as_bytes())?;
//     std::fs::write(public_json, ser_inputs_str.as_bytes())?;
//
//     Result::Ok(())
// }

/*
pub fn plonk_setup_fix_compressor(
    r1cs: &R1CS<GL>,
    opts: &Options,
    pil: &PIL,
    aux: &PlonkSetupInfo,
) {
    let mut const_pols = polsarray::PolsArray::new(pil, polsarray::PolKind::Constant);

    let n_used = aux.n_used;
    let n_publics = aux.n_publics;
    let n_public_rows = (n_publics - 1) / 12 + 1;

    let mut r = 0;
    let mut compressor: Compressor = Compressor::new(n_public_rows + r);

    let mut s_map: Vec<Vec<u64>> = vec![Vec::new(); 12];
    for i in 0..12 {
        s_map[i] = vec![0u64; n_used];
    }

    for i in 0..n_publics {
        s_map[i % 12][r + i / 12] = 1 + i as u64;
    }

    for i in n_publics..(n_public_rows * 12) {
        s_map[i % 12][r + i / 12] = 0;
    }
    r += n_public_rows;
    // Paste plonk constraints.

    struct ParRow {
        row: usize,
        n_used: usize,
    };
    let mut partial_rows: HashMap<String, ParRow> = HashMap::new();
    let plonk_constraints = &aux.pg;
    for (i, c) in plonk_constraints.iter().enumerate() {
        if (i % 10000) == 0 {
            println!("Processing constraint... {}/{}", i, plonk_constraints.len())
        };
        let k = c.str_key();
        let pr = partial_rows.get_mut(&k);
        if pr.is_some() {
            let pr = pr.unwrap();
            s_map[pr.n_used * 3][pr.row] = c.0 as u64;
            s_map[pr.n_used * 3 + 1][pr.row] = c.1 as u64;
            s_map[pr.n_used * 3 + 2][pr.row] = c.2 as u64;
            pr.n_used += 1;
            if pr.n_used == 4 {
                partial_rows.remove(&k);
            }
        } else {
            compressor.Qm[r] = c.3.clone();
            compressor.Ql[r] = c.4.clone();
            compressor.Qr[r] = c.5.clone();
            compressor.Qo[r] = c.6.clone();
            compressor.Qk[r] = c.7.clone();
            compressor.QCMul[r] = FGL::ZERO;
            compressor.QMDS[r] = FGL::ZERO;
            s_map[0][r] = c.0 as u64;
            s_map[1][r] = c.1 as u64;
            s_map[2][r] = c.2 as u64;
            partial_rows.insert(k, ParRow { row: r, n_used: 1 });
            r += 1;
        }
    }

    // Terminate the empty rows (Copyn the same constraint)
    for (k, pr) in partial_rows.iter() {
        for j in pr.n_used..4 {
            s_map[j * 3][pr.row] = s_map[0][pr.row];
            s_map[j * 3 + 1][pr.row] = s_map[1][pr.row];
            s_map[j * 3 + 2][pr.row] = s_map[2][pr.row];
        }
    }

    // Generate Custom Gates
    for (i, cgu) in r1cs.custom_gates_uses.iter().enumerate() {
        if (i % 10000) == 0 {
            println!(
                "Processing custom gates... {}/{}",
                i,
                r1cs.custom_gates_uses.len()
            );
        }
        if cgu.id == aux.custom_gates_info.cmds_id {
            assert_eq!(cgu.signals.len(), 24);
            for i in 0..12 {
                s_map[i][r] = cgu.signals[i];
                s_map[i][r + 1] = cgu.signals[i + 12];
            }
            compressor.Qm[r] = FGL::ZERO;
            compressor.Ql[r] = FGL::ZERO;
            compressor.Qr[r] = FGL::ZERO;
            compressor.Qo[r] = FGL::ZERO;
            compressor.Qk[r] = FGL::ZERO;
            compressor.QCMul[r] = FGL::ZERO;
            compressor.QMDS[r] = FGL::ONE;
            compressor.Qm[r + 1] = FGL::ZERO;
            compressor.Ql[r + 1] = FGL::ZERO;
            compressor.Qr[r + 1] = FGL::ZERO;
            compressor.Qo[r + 1] = FGL::ZERO;
            compressor.Qk[r + 1] = FGL::ZERO;
            compressor.QCMul[r + 1] = FGL::ZERO;
            compressor.QMDS[r + 1] = FGL::ZERO;

            r += 2;
        } else if cgu.id == aux.custom_gates_info.cmul_id {
            for i in 0..9 {
                s_map[i][r] = cgu.signals[i];
            }
            for i in 9..12 {
                s_map[i][r] = 0;
            }
            compressor.Qm[r] = FGL::ZERO;
            compressor.Ql[r] = FGL::ZERO;
            compressor.Qr[r] = FGL::ZERO;
            compressor.Qo[r] = FGL::ZERO;
            compressor.Qk[r] = FGL::ZERO;
            compressor.QCMul[r] = FGL::ONE;
            compressor.QMDS[r] = FGL::ZERO;

            r += 1;
        }
    }

    // Calculate S Polynomials
    let ks = crate::helper::get_ks(11);
    let mut w = FGL::ONE;
    compressor.S = vec![Vec::new(); 12];
    for i in 0..12 {
        compressor.S[i] = vec![FGL::ZERO; aux.plonkinfo.N];
    }
    for i in 0..aux.plonkinfo.N {
        if (i % 10000) == 0 {
            println!("Preparing S... {}/{}", i, aux.plonkinfo.N);
        }
        compressor.S[0][i] = w;
        for j in 1..12 {
            compressor.S[j][i] = w * ks[j - 1];
        }
        w = w * (crate::constant::MG.0[aux.n_bits].to_be());
    }

    struct Grid {
        row: usize,
        col: usize,
    };
    let mut last_signal: HashMap<u64, Grid> = HashMap::new();

    for i in 0..r {
        if (i % 10000) == 0 {
            println!("Connection S... {}/{}", i, r);
        }
        for j in 0..12 {
            if s_map[j][i] > 0 {
                let ls = last_signal.get(&s_map[j][i]);
                if ls.is_some() {
                    let ls = ls.unwrap();
                    //connect(&mut compressor.S[ls.col], ls.row, &mut compressor.S[j], i);
                    let tmp = compressor.S[j][i];
                    let tmp2 = compressor.S[ls.col][ls.row];
                    compressor.S[ls.col][ls.row] = tmp;
                    compressor.S[j][i] = tmp2;
                } else {
                    last_signal.insert(s_map[j][i], Grid { col: j, row: i });
                }
            }
        }
    }

    // Fill unused rows
    while r < aux.plonkinfo.N {
        if (r % 100000) == 0 {
            println!("Empty gates... {}/{}", r, aux.plonkinfo.N);
        }
        compressor.Qm[r] = FGL::ZERO;
        compressor.Ql[r] = FGL::ZERO;
        compressor.Qr[r] = FGL::ZERO;
        compressor.Qo[r] = FGL::ZERO;
        compressor.Qk[r] = FGL::ZERO;
        compressor.QCMul[r] = FGL::ZERO;
        compressor.QMDS[r] = FGL::ZERO;
        r += 1;
    }

    /*
    for i in 0..n_public_rows {
        let L = const_pols.get_mut(&"Global".to_string(), &format!("L{}", i + 1));
        for i in 0..aux.plonkinfo.N {
            L[i] = FGL::ZERO;
        }
        L[i] = FGL::ONE;
    }

    (
        pilStr: pilStr,
        constPols: constPols,
        s_map: s_map,
        plonkAdditions: plonkAdditions
    )
    */
}
*/
