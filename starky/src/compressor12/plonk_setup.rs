use crate::compressor12::compressor12_pil;
use crate::compressor12_setup::Options;
use crate::pilcom::compile_pil;
use crate::polsarray::PolsArray;
use crate::r1cs2plonk::{r1cs2plonk, PlonkAdd, PlonkGate};
use crate::types::PIL;
use crate::{pilcom, polsarray};
use plonky::circom_circuit::R1CS;
use plonky::field_gl::Fr as FGL;
use plonky::field_gl::GL;
use plonky::reader::load_r1cs;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Default)]
pub struct PlonkSetup {
    pub(crate) pil_str: String,
    pub(crate) const_pols: PolsArray,
    pub(crate) s_map: Vec<Vec<u64>>,
    pub(crate) plonk_additions: Vec<PlonkAdd>,
}

impl PlonkSetup {
    pub fn plonk_setup(r1cs: &R1CS<GL>, opts: &Options) -> Self {
        // 1. plonk_setup_render phase
        let plonk_setup_info = PlonkSetupRenderInfo::plonk_setup_render(r1cs, opts);

        // 2. render .pil file by template.
        // //      And save as a file.
        let pil_str = compressor12_pil::render(plonk_setup_info.n_bits, plonk_setup_info.n_publics);
        // let mut file = File::create(out_pil.clone()).unwrap();
        // write!(file, "{}", pil_str).unwrap();

        // 3. compile pil to pil_json
        let pil_json = compile_pil(&pil_str);

        //4. plonk_setup_fix_compressor phase
        let (const_pols, s_map) = plonk_setup_compressor(r1cs, &pil_json, &plonk_setup_info);

        Self {
            pil_str,
            const_pols,
            s_map,
            plonk_additions: plonk_setup_info.pa,
        }
    }
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

pub(crate) struct NormalPlonkInfo {
    pub N: usize,
    pub nConstaints: usize,
    pub nPlonkGates: usize,
    pub nPlonkAdds: usize,
}

impl NormalPlonkInfo {
    // need check
    pub(crate) fn new(r1cs: &R1CS<GL>, pa: &Vec<PlonkGate>, pc: &Vec<PlonkAdd>) -> Self {
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

        Self {
            N,
            nConstaints: r1cs.constraints.len(),
            nPlonkGates: pa.len(),
            nPlonkAdds: pc.len(),
        }
    }
}

pub(crate) struct CustomGateInfo {
    pub cmul_id: u64,
    pub cmds_id: u64,
    pub n_cmul: u64,
    pub n_mds: u64,
}

impl CustomGateInfo {
    // need check
    fn from_r1cs(r1cs: &R1CS<GL>) -> Self {
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
        Self {
            cmul_id,
            cmds_id,
            n_cmul,
            n_mds,
        }
    }
}

pub struct PlonkSetupRenderInfo {
    n_used: usize,
    n_bits: usize,
    n_publics: usize,
    pub(crate) pg: Vec<PlonkGate>,
    pub(crate) pa: Vec<PlonkAdd>,
    custom_gates_info: CustomGateInfo,
    pub(crate) plonk_info: NormalPlonkInfo,
}

impl PlonkSetupRenderInfo {
    // need check
    pub fn plonk_setup_render(r1cs: &R1CS<GL>, opts: &Options) -> Self {
        // 1. r1cs to plonk
        let (plonk_constrains, plonk_additions) = r1cs2plonk(r1cs);

        // 2. get normal plonk info
        let plonk_info = NormalPlonkInfo::new(r1cs, &plonk_constrains, &plonk_additions);
        // 3. get custom gate info
        let custom_gates_info = CustomGateInfo::from_r1cs(r1cs);

        // 4. calculate columns,rows,constraints info.
        let n_publics = r1cs.num_inputs + r1cs.num_outputs;
        let n_public_rows = (n_publics - 1) / 12 + 1;
        let n_used = n_public_rows
            + plonk_info.N as usize
            + custom_gates_info.n_cmul as usize
            + custom_gates_info.n_mds as usize * 2;
        let mut n_bits = crate::helper::log2_any(n_used - 1) + 1;
        if opts.force_bits > 0 {
            n_bits = opts.force_bits;
        }

        Self {
            n_used,
            n_bits,
            n_publics,
            pg: plonk_constrains,
            pa: plonk_additions,
            custom_gates_info,
            plonk_info,
        }
    }
}

pub fn plonk_setup_compressor(
    r1cs: &R1CS<GL>,
    pil: &PIL,
    aux: &PlonkSetupRenderInfo,
) -> (PolsArray, Vec<Vec<u64>>) {
    // 1. construct init ConstantPolsArray
    let mut const_pols = polsarray::PolsArray::new(pil, polsarray::PolKind::Constant);

    let n_used = aux.n_used;
    let n_publics = aux.n_publics;
    let n_public_rows = (n_publics - 1) / 12 + 1;

    // 2. init sMap and construct it.
    let mut s_map: Vec<Vec<u64>> = vec![Vec::new(); 12];
    for i in 0..12 {
        s_map[i] = vec![0u64; n_used];
    }

    let mut r = 0;

    // Paste public inputs. todo check
    let mut compressor: Compressor = Compressor::new(n_public_rows + r);

    for i in 0..n_publics {
        s_map[i % 12][r + i / 12] = 1 + i as u64;
    }
    for i in n_publics..(n_public_rows * 12) {
        s_map[i % 12][r + i / 12] = 0;
    }
    r += n_public_rows;

    // 3. Paste plonk constraints.
    struct ParRow {
        row: usize,
        n_used: usize,
    };
    let mut partial_rows: HashMap<String, ParRow> = HashMap::new();
    let mut half_rows = vec![];
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
            if pr.n_used == 2 {
                half_rows.push(pr);
                partial_rows.remove(&k);
            } else if pr.n_used == 4 {
                partial_rows.remove(&k);
            }
        } else if half_rows.len() > 0 {
            // todo
            // const pr = halfRows.shift();
            // constPols.Compressor.C[9][pr.row] = c[3];
            // constPols.Compressor.C[6][pr.row] = c[4];
            // constPols.Compressor.C[7][pr.row] = c[5];
            // constPols.Compressor.C[8][pr.row] = c[6];
            // constPols.Compressor.C[10][pr.row] = c[7];
            // constPols.Compressor.C[11][pr.row] = 0n;
            //
            // sMap[pr.nUsed*3][pr.row] = c[0];
            // sMap[pr.nUsed*3+1][pr.row] = c[1];
            // sMap[pr.nUsed*3+2][pr.row] = c[2];
            // pr.nUsed ++;
            // partialRows[k] = pr;
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
    // toco check
    for (k, pr) in partial_rows.iter() {
        for j in pr.n_used..4 {
            s_map[j * 3][pr.row] = s_map[0][pr.row];
            s_map[j * 3 + 1][pr.row] = s_map[1][pr.row];
            s_map[j * 3 + 2][pr.row] = s_map[2][pr.row];
        }
    }

    // 4. Generate Custom Gates
    // todo need append.
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

    // 5. Calculate S Polynomials
    let ks = crate::helper::get_ks(11);
    let mut w = FGL::ONE;
    compressor.S = vec![Vec::new(); 12];
    // for i in 0..12 {
    //     compressor.S[i] = vec![FGL::ZERO; aux.plonkinfo.N];
    // }
    for i in 0..aux.plonk_info.N {
        if (i % 10000) == 0 {
            println!("Preparing S... {}/{}", i, aux.plonk_info.N);
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

    // 6. Fill unused rows.
    while r < aux.plonk_info.N {
        if (r % 100000) == 0 {
            println!("Empty gates... {}/{}", r, aux.plonk_info.N);
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

    for i in 0..n_public_rows {
        let L = const_pols.get_mut(&"Global".to_string(), &format!("L{}", i + 1));
        for i in 0..aux.plonk_info.N {
            L[i] = FGL::ZERO;
        }
        L[i] = FGL::ONE;
    }

    (const_pols, s_map)
}
