use crate::compressor12::compressor12_pil;
use crate::compressor12_setup::Options;
use crate::pilcom::compile_pil_from_str;
use crate::polsarray::PolsArray;
use crate::r1cs2plonk::{r1cs2plonk, PlonkAdd, PlonkGate};
use crate::types::PIL;
use crate::{pilcom, polsarray};
use ff::PrimeField;
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
        let pil_json = compile_pil_from_str(&pil_str);

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

pub(crate) struct NormalPlonkInfo {
    pub N: usize,
    pub n_constaints: usize,
    pub n_plonk_gates: usize,
    pub n_plonk_adds: usize,
}

impl NormalPlonkInfo {
    pub(crate) fn new(
        r1cs: &R1CS<GL>,
        plonk_constrains: &Vec<PlonkGate>,
        plonk_additions: &Vec<PlonkAdd>,
    ) -> Self {
        let mut uses: HashMap<String, usize> = HashMap::new();
        let plonk_constrains_len = plonk_constrains.len();
        for (i, c) in plonk_constrains.iter().enumerate() {
            if (i % 10000) == 0 {
                log::info!("Plonk info constraint processing... {i}/{plonk_constrains_len}");
            }
            let k = c.str_key();

            uses.entry(k.clone())
                .and_modify(|e| *e += 1)
                .or_insert_with(1);
        }
        let mut result = uses.values().collect::<Vec<usize>>();
        result.sort(); // sort by asc

        let mut N = result.iter().fold(0, |acc, &x| acc + (x - 1) / 4 + 1);
        N = (N - 1) / 2 + 1;

        Self {
            N,
            n_constaints: r1cs.constraints.len(),
            n_plonk_gates: plonk_constrains_len,
            n_plonk_adds: plonk_additions.len(),
        }
    }
}

pub(crate) struct CustomGateInfo<F: PrimeField> {
    pub(crate) poseidon_id: u64,
    pub(crate) c_mul_add_id: u64,
    pub(crate) fft_params: Vec<F>,
    pub(crate) ev_pol_id: u64,

    pub(crate) n_poseidon: u64,
    pub(crate) n_c_mul_add: u64,
    pub(crate) n_fft: u64,
    pub(crate) n_ev_pol: u64,
}

impl<F: PrimeField> CustomGateInfo<F> {
    fn from_r1cs(r1cs: &R1CS<GL>) -> Self {
        let mut c_mul_add_id = None;
        let mut poseidon_id = None;
        let mut ev_pol_id = None;
        let mut fft_params = None;

        let mut n_c_mul_add = 0;
        let mut n_poseidon = 0;
        let mut n_fft = 0;
        let mut n_ev_pol = 0;

        for (i, c) in r1cs.custom_gates.iter().enumerate() {
            assert!(c.parameters.len() == 0);
            match c.template_name.as_str() {
                "CMulAdd" => {
                    if None == c_mul_add_id {
                        c_mul_add_id = Some(i as u64);
                    }
                    n_c_mul_add += 1;
                }
                "Poseidon12" => {
                    if None == poseidon_id {
                        poseidon_id = Some(i as u64);
                    }
                    n_poseidon += 1;
                }
                "EvPol4" => {
                    if None == ev_pol_id {
                        ev_pol_id = Some(i as u64);
                    }
                    n_ev_pol += 1;
                }
                "FFT4" => {
                    if None == fft_params {
                        fft_params = Some(c.parameters.clone());
                    }
                    n_fft += 1;
                }
                _ => panic!("Invalid custom gate {}", c.template_name),
            }
        }

        Self {
            poseidon_id: poseidon_id.unwrap_or(0),
            c_mul_add_id: c_mul_add_id.unwrap_or(0),
            fft_params: fft_params.unwrap_or(vec![]),
            ev_pol_id: ev_pol_id.unwrap_or(0),
            n_poseidon,
            n_c_mul_add,
            n_fft,
            n_ev_pol,
        }
    }
}

// todo field type F3G?
pub struct PlonkSetupRenderInfo<F: PrimeField> {
    n_used: usize,
    n_bits: usize,
    n_publics: usize,
    pub(crate) pg: Vec<PlonkGate>,
    pub(crate) pa: Vec<PlonkAdd>,
    custom_gates_info: CustomGateInfo<F>,
    pub(crate) plonk_info: NormalPlonkInfo,
}

impl<F: PrimeField> PlonkSetupRenderInfo<F> {
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
            + plonk_info.N
            + custom_gates_info.n_c_mul_add
            + custom_gates_info.n_poseidon * 31
            + custom_gates_info.n_fft * 2
            + custom_gates_info.n_ev_pol * 2;

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

pub fn plonk_setup_compressor<F: PrimeField>(
    r1cs: &R1CS<GL>,
    pil: &PIL,
    plonk_setup_info: &PlonkSetupRenderInfo<F>,
) -> (PolsArray, Vec<Vec<u64>>) {
    // 1. construct init ConstantPolsArray
    let mut const_pols = PolsArray::new(pil, polsarray::PolKind::Constant);

    let n_used = plonk_setup_info.n_used;
    let n_publics = plonk_setup_info.n_publics;
    let n_public_rows = (n_publics - 1) / 12 + 1;

    // 2. init sMap and construct it.
    let mut s_map: Vec<Vec<u64>> = vec![vec![0u64; n_used]; 12];
    // for i in 0..12 {
    //     s_map[i] = vec![0u64; n_used];
    // }

    let mut r = 0;

    // Paste public inputs.
    for i in 0..n_public_rows {
        // constPols.Compressor.EVPOL4[r+i] = 0n;
        // constPols.Compressor.CMULADD[r+i] = 0n;
        // constPols.Compressor.GATE[r+i] = 0n;
        // constPols.Compressor.POSEIDON12[r+i] = 0n;
        // constPols.Compressor.PARTIAL[r+i] = 0n;
        // constPols.Compressor.FFT4[r+i] = 0n;
        for j in 0..12 {
            // compressor.QMDS[r] = FGL::ZERO;
        }
    }
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
    let mut half_rows: Vec<ParRow> = vec![];
    let plonk_constraints = &plonk_setup_info.pg;
    for (i, c) in plonk_constraints.iter().enumerate() {
        if (i % 10000) == 0 {
            log::info!("Processing constraint... {}/{}", i, plonk_constraints.len())
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
                half_rows.push(*pr);
                partial_rows.remove(&k);
            } else if pr.n_used == 4 {
                partial_rows.remove(&k);
            }
        } else if half_rows.len() > 0 {
            // todo
            let mut pr = ParRow { row: 0, n_used: 0 };
            // const pr = halfRows.shift();
            // constPols.Compressor.C[9][pr.row] = c[3];
            // constPols.Compressor.C[6][pr.row] = c[4];
            // constPols.Compressor.C[7][pr.row] = c[5];
            // constPols.Compressor.C[8][pr.row] = c[6];
            // constPols.Compressor.C[10][pr.row] = c[7];
            // constPols.Compressor.C[11][pr.row] = 0n;
            //
            s_map[pr.n_used * 3][pr.row] = c.0 as u64;
            s_map[pr.n_used * 3 + 1][pr.row] = c.1 as u64;
            s_map[pr.n_used * 3 + 2][pr.row] = c.2 as u64;
            pr.n_used += 1;
            partial_rows.insert(k, pr.clone());
        } else {
            // compressor.Qm[r] = c.3.clone();
            // compressor.Ql[r] = c.4.clone();
            // compressor.Qr[r] = c.5.clone();
            // compressor.Qo[r] = c.6.clone();
            // compressor.Qk[r] = c.7.clone();
            // compressor.QCMul[r] = FGL::ZERO;
            // compressor.QMDS[r] = FGL::ZERO;

            s_map[0][r] = c.0 as u64;
            s_map[1][r] = c.1 as u64;
            s_map[2][r] = c.2 as u64;
            partial_rows.insert(k, ParRow { row: r, n_used: 1 });
            r += 1;
        }
    }

    // Terminate the empty rows (Copyn the same constraint)
    for (_, pr) in partial_rows.iter_mut() {
        if pr.n_used == 1 {
            s_map[3][pr.row] = s_map[0][pr.row];
            s_map[4][pr.row] = s_map[1][pr.row];
            s_map[5][pr.row] = s_map[2][pr.row];
            pr.n_used += 1;
            half_rows.push(pr.clone());
        } else if pr.n_used == 3 {
            s_map[9][pr.row] = s_map[6][pr.row];
            s_map[10][pr.row] = s_map[7][pr.row];
            s_map[11][pr.row] = s_map[8][pr.row];
        } else {
            panic!(" meet error when terminate the empty rows")
        }
    }

    for hr in half_rows.iter() {
        s_map[6][hr.row] = 0;
        s_map[7][hr.row] = 0;
        s_map[8][hr.row] = 0;
        s_map[9][hr.row] = 0;
        s_map[10][hr.row] = 0;
        s_map[11][hr.row] = 0;
        // constPols.Compressor.C[9][pr.row] = 0n;
        // constPols.Compressor.C[6][pr.row] = 0n;
        // constPols.Compressor.C[7][pr.row] = 0n;
        // constPols.Compressor.C[8][pr.row] = 0n;
        // constPols.Compressor.C[10][pr.row] = 0n;
        // constPols.Compressor.C[11][pr.row] = 0n;
    }

    // 4. Generate Custom Gates
    let custom_gates_info = &plonk_setup_info.custom_gates_info;
    for (i, cgu) in r1cs.custom_gates_uses.iter().enumerate() {
        if (i % 10000) == 0 {
            log::info!(
                "Processing custom gates... {}/{}",
                i,
                r1cs.custom_gates_uses.len()
            );
        }
        if cgu.id == custom_gates_info.poseidon_id {
            assert_eq!(cgu.signals.len(), 31 * 12);
            for j in 0..31 {
                for k in 0..12 {
                    s_map[k][r + j] = cgu.signals[i * 12 + k];
                    // constPols.Compressor.C[j][r+i] = CPOSEIDON[i*12+j];
                }
                // compressor.Qm[r] = FGL::ZERO;
                // compressor.Ql[r] = FGL::ZERO;
                // compressor.Qr[r] = FGL::ZERO;
                // compressor.Qo[r] = FGL::ZERO;
                // compressor.Qk[r] = FGL::ZERO;
                // compressor.QCMul[r] = FGL::ZERO;
                // compressor.QMDS[r] = FGL::ONE;
                // compressor.Qm[r + 1] = FGL::ZERO;
                // compressor.Ql[r + 1] = FGL::ZERO;
                // compressor.Qr[r + 1] = FGL::ZERO;
                // compressor.Qo[r + 1] = FGL::ZERO;
                // compressor.Qk[r + 1] = FGL::ZERO;
                // compressor.QCMul[r + 1] = FGL::ZERO;
                // compressor.QMDS[r + 1] = FGL::ZERO;
            }
            r += 31;
        } else if cgu.id == custom_gates_info.c_mul_add_id {
            for j in 0..12 {
                s_map[j][r] = cgu.signals[j];
            }
            // compressor.Qm[r] = FGL::ZERO;
            // compressor.Ql[r] = FGL::ZERO;
            // compressor.Qr[r] = FGL::ZERO;
            // compressor.Qo[r] = FGL::ZERO;
            // compressor.Qk[r] = FGL::ZERO;
            // compressor.QCMul[r] = FGL::ONE;
            // compressor.QMDS[r] = FGL::ZERO;

            r += 1;
        } else if (custom_gates_info.fft_params.len() as u64) < cgu.id {
            for j in 0..12 {
                s_map[j][r] = cgu.signals[j];
            }
            for j in 0..12 {
                s_map[j][r + 1] = cgu.signals[12 + j];
            }
            // constPols.Compressor.CMULADD[r] = 0n;
            // constPols.Compressor.GATE[r] = 0n;
            // constPols.Compressor.POSEIDON12[r] = 0n;
            // constPols.Compressor.PARTIAL[r] = 0n;
            // constPols.Compressor.EVPOL4[r] = 0n;
            // constPols.Compressor.FFT4[r] = 1n;
            // constPols.Compressor.CMULADD[r+1] = 0n;
            // constPols.Compressor.GATE[r+1] = 0n;
            // constPols.Compressor.POSEIDON12[r+1] = 0n;
            // constPols.Compressor.PARTIAL[r+1] = 0n;
            // constPols.Compressor.EVPOL4[r+1] = 0n;
            // constPols.Compressor.FFT4[r+1] = 0n;

            // todo check.
            r += 2;
        } else if cgu.id == custom_gates_info.ev_pol_id {
            for j in 0..12 {
                s_map[j][r] = cgu.signals[j];
                // constPols.Compressor.C[j][r+i] = CPOSEIDON[i*12+j];
            }
            for j in 0..9 {
                s_map[j][r + 1] = cgu.signals[12 + j];
                // constPols.Compressor.C[j][r+i] = CPOSEIDON[i*12+j];
            }
            for j in 9..12 {
                s_map[j][r + 1] = 0;
                // constPols.Compressor.C[j][r+i] = CPOSEIDON[i*12+j];
            }

            // constPols.Compressor.GATE[r+1] = 0n;
            // constPols.Compressor.POSEIDON12[r+1] = 0n;
            // constPols.Compressor.PARTIAL[r+1] = 0n;
            // constPols.Compressor.EVPOL4[r+1] = 0n;
            // constPols.Compressor.FFT4[r+1] = 0n;
            r += 2
        } else {
            panic!("Custom gate not defined: {}", cgu.id);
        }
    }

    // 5. Calculate S Polynomials
    let N = 1 << plonk_setup_info.n_bits;
    let ks = crate::helper::get_ks(11);
    let mut w = FGL::ONE;
    for i in 0..N {
        if (i % 10000) == 0 {
            log::info!("Preparing S... {}/{}", i, plonk_setup_info.plonk_info.N);
        }
        // compressor.S[0][i] = w;
        for j in 1..12 {
            // compressor.S[j][i] = w * ks[j - 1];
        }
        w = w * (crate::constant::MG.0[plonk_setup_info.n_bits].to_be());
    }

    struct Grid {
        row: usize,
        col: usize,
    };
    let mut last_signal: HashMap<u64, Grid> = HashMap::new();
    for i in 0..r {
        if (i % 10000) == 0 {
            log::info!("Connection S... {}/{}", i, r);
        }
        for j in 0..12 {
            let key = s_map[j][i];
            if key > 0 {
                let ls = last_signal.get(&key);
                if ls.is_some() {
                    let ls = ls.unwrap();
                    //connect(&mut compressor.S[ls.col], ls.row, &mut compressor.S[j], i);
                } else {
                    last_signal.insert(key, Grid { col: j, row: i });
                }
            }
        }
    }

    // 6. Fill unused rows.
    while r < N {
        if (r % 100000) == 0 {
            log::info!("Empty gates... {}/{}", r, N);
        }
        // compressor.Qm[r] = FGL::ZERO;
        // compressor.Ql[r] = FGL::ZERO;
        // compressor.Qr[r] = FGL::ZERO;
        // compressor.Qo[r] = FGL::ZERO;
        // compressor.Qk[r] = FGL::ZERO;
        // compressor.QCMul[r] = FGL::ZERO;
        // compressor.QMDS[r] = FGL::ZERO;
        for j in 0..12 {
            // compressor.QMDS[r] = FGL::ZERO;
        }
        r += 1;
    }
    // construct Lagrange Basis Polynomial: Li(x)
    for i in 0..n_public_rows {
        let L = const_pols.get_mut(&"Global".to_string(), &format!("L{}", i + 1));
        for i in 0..N {
            L[i] = FGL::ZERO;
        }
        L[i] = FGL::ONE;
    }

    (const_pols, s_map)
}
