#![allow(non_snake_case)]
use super::{
    compressor12_pil, compressor12_pil::CompressorNameSpace::*,
    compressor12_pil::CompressorPolName::*, compressor12_setup::Options, constants::CPOSEIDON,
};
use crate::pilcom::compile_pil_from_str;
use crate::r1cs2plonk::{r1cs2plonk, PlonkAdd, PlonkGate};
use algebraic::circom_circuit::R1CS;
use array_tool::vec::Shift;
use fields::field_gl::Fr as FGL;
use fields::field_gl::GL;
use starky::helper;
use starky::polsarray::{PolKind, PolsArray};
use starky::types::PIL;
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct PlonkSetup {
    pub(crate) pil_str: String,
    pub(crate) const_pols: PolsArray,
    pub(crate) s_map: Vec<Vec<u64>>,
    pub(crate) plonk_additions: Vec<PlonkAdd>,
}

impl PlonkSetup {
    pub fn new(r1cs: &R1CS<GL>, opts: &Options) -> Self {
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

        Self { pil_str, const_pols, s_map, plonk_additions: plonk_setup_info.pa }
    }
}

#[derive(Debug)]
pub(crate) struct NormalPlonkInfo {
    pub N: usize,
    // never used fileds
    // pub n_constaints: usize,
    // pub n_plonk_gates: usize,
    // pub n_plonk_adds: usize,
}

impl NormalPlonkInfo {
    pub(crate) fn new(plonk_constrains: &[PlonkGate]) -> Self {
        let mut uses: BTreeMap<String, usize> = BTreeMap::new();
        let plonk_constrains_len = plonk_constrains.len();
        for (i, c) in plonk_constrains.iter().enumerate() {
            if (i % 10000) == 0 {
                log::trace!("Plonk info constraint processing... {i}/{plonk_constrains_len}");
            }
            let k = c.str_key();

            uses.entry(k.clone()).and_modify(|e| *e += 1).or_insert_with(|| 1);
        }
        let mut result = uses.values().collect::<Vec<_>>();
        result.sort(); // sort by asc

        let mut N = result.iter().fold(0, |acc, x| acc + (**x - 1) / 2 + 1);
        N = (N - 1) / 2 + 1;

        Self {
            N,
            // n_constaints: r1cs.constraints.len(),
            // n_plonk_gates: plonk_constrains_len,
            // n_plonk_adds: plonk_additions.len(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CustomGateInfo {
    pub(crate) poseidon_id: u64,
    pub(crate) c_mul_add_id: u64,
    pub(crate) fft_params: BTreeMap<usize, Vec<FGL>>,
    pub(crate) ev_pol_id: u64,

    pub(crate) n_poseidon: u64,
    pub(crate) n_c_mul_add: u64,
    pub(crate) n_fft: u64,
    pub(crate) n_ev_pol: u64,
}

impl CustomGateInfo {
    // equal to `typeof customGatesInfo.FFT4Parameters[cgu.id] !== "undefined"` in js
    // Defined: properer index and has value.
    #[inline(always)]
    pub fn check_fft_param_defined(fft_params: &BTreeMap<usize, Vec<FGL>>, index: u64) -> bool {
        fft_params.get(&(index as usize)).is_some()
    }

    fn from_r1cs(r1cs: &R1CS<GL>) -> Self {
        let mut c_mul_add_id = 0;
        let mut poseidon_id = 0;
        let mut ev_pol_id = 0;
        // let mut fft_params = vec![vec![]; r1cs.custom_gates.len()];
        let mut fft_params: BTreeMap<usize, Vec<FGL>> = BTreeMap::new();

        for (i, c) in r1cs.custom_gates.iter().enumerate() {
            match c.template_name.as_str() {
                "CMulAdd" => {
                    c_mul_add_id = i as u64;
                    assert!(c.parameters.is_empty());
                }
                "Poseidon12" => {
                    poseidon_id = i as u64;
                    assert!(c.parameters.is_empty());
                }
                "EvPol4" => {
                    ev_pol_id = i as u64;
                    assert!(c.parameters.is_empty());
                }
                "FFT4" => {
                    fft_params.insert(i, c.parameters.clone());
                }
                _ => panic!("Invalid custom gate {}", c.template_name),
            }
        }

        let mut n_c_mul_add = 0;
        let mut n_poseidon = 0;
        let mut n_fft = 0;
        let mut n_ev_pol = 0;
        for c in r1cs.custom_gates_uses.iter() {
            if c.id == c_mul_add_id {
                n_c_mul_add += 1;
            } else if c.id == poseidon_id {
                n_poseidon += 1;
            } else if Self::check_fft_param_defined(&fft_params, c.id) {
                n_fft += 1;
            } else if c.id == ev_pol_id {
                n_ev_pol += 1;
            } else {
                panic!("Custom gate not defined {}", c.id);
            }
        }

        Self {
            poseidon_id,
            c_mul_add_id,
            fft_params,
            ev_pol_id,
            n_poseidon,
            n_c_mul_add,
            n_fft,
            n_ev_pol,
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
    // pub(crate) plonk_info: NormalPlonkInfo, // Never used.
}

impl PlonkSetupRenderInfo {
    pub fn plonk_setup_render(r1cs: &R1CS<GL>, opts: &Options) -> Self {
        // 1. r1cs to plonk
        let (plonk_constrains, plonk_additions) = r1cs2plonk(r1cs);

        // 2. get normal plonk info
        let plonk_info = NormalPlonkInfo::new(&plonk_constrains);
        // 3. get custom gate info

        let custom_gates_info = CustomGateInfo::from_r1cs(r1cs);

        // 4. calculate columns,rows,constraints info.
        let n_publics = r1cs.num_inputs + r1cs.num_outputs - 1;
        let n_public_rows = (n_publics - 1) / 12 + 1;

        log::debug!("{n_publics} {n_public_rows} {} {:?}", plonk_info.N, custom_gates_info);
        let n_used = n_public_rows
            + plonk_info.N
            + (custom_gates_info.n_c_mul_add
                + custom_gates_info.n_poseidon * 31
                + custom_gates_info.n_fft * 2
                + custom_gates_info.n_ev_pol * 2) as usize;

        let mut n_bits = helper::log2_any(n_used - 1) + 1;
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
        }
    }
}

pub fn plonk_setup_compressor(
    r1cs: &R1CS<GL>,
    pil: &PIL,
    plonk_setup_info: &PlonkSetupRenderInfo,
) -> (PolsArray, Vec<Vec<u64>>) {
    // 1. construct init ConstantPolsArray
    log::debug!("pil: new constant");
    let mut const_pols = PolsArray::new(pil, PolKind::Constant);

    let n_used = plonk_setup_info.n_used;
    let n_publics = plonk_setup_info.n_publics;
    let n_public_rows = (n_publics - 1) / 12 + 1;

    log::debug!("n_used {n_used}, n_publics {n_publics}, rows: {n_public_rows}");
    // 2. init sMap and construct it.
    let mut s_map: Vec<Vec<u64>> = vec![vec![0u64; n_used]; 12];

    let mut r = 0;

    // Paste public inputs.
    for i in 0..n_public_rows {
        let index = r + i;
        for pol_name in [EVPOL4, CMULADD, GATE, POSEIDON12, PARTIAL, FFT4] {
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &pol_name.to_string(),
                0,
                index,
                FGL::ZERO,
            );
        }
        for k in 0..12 {
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &C.to_string(),
                k,
                index,
                FGL::ZERO,
            );
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
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    struct ParRow {
        row: usize,
        n_used: usize,
    }
    // Paste plonk constraints.
    let mut partial_rows: BTreeMap<String, ParRow> = BTreeMap::new();
    let mut half_rows: Vec<ParRow> = vec![];
    let plonk_constraints = &plonk_setup_info.pg;
    for (i, c) in plonk_constraints.iter().enumerate() {
        if (i % 10000) == 0 {
            log::trace!("Processing constraint... {}/{}", i, plonk_constraints.len())
        }
        let k = c.str_key();
        let pr = partial_rows.get_mut(&k);
        if let Some(pr) = pr {
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
        } else if !half_rows.is_empty() {
            let mut pr = half_rows.shift().unwrap();
            let index = pr.row;
            for (i, value) in
                [9_usize, 6, 7, 8, 10, 11].iter().zip([c.3, c.4, c.5, c.6, c.7, FGL::ZERO].iter())
            {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    *i,
                    index,
                    *value,
                );
            }

            s_map[pr.n_used * 3][pr.row] = c.0 as u64;
            s_map[pr.n_used * 3 + 1][pr.row] = c.1 as u64;
            s_map[pr.n_used * 3 + 2][pr.row] = c.2 as u64;
            pr.n_used += 1;
            partial_rows.insert(k, pr);
        } else {
            let index = r;
            for (i, value) in
                [3_usize, 0, 1, 2, 4, 5].iter().zip([c.3, c.4, c.5, c.6, c.7, FGL::ZERO].iter())
            {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    *i,
                    index,
                    *value,
                );
            }
            for (pol_name, value) in [GATE, POSEIDON12, PARTIAL, CMULADD, EVPOL4, FFT4]
                .iter()
                .zip(vec![FGL::ONE, FGL::ZERO, FGL::ZERO, FGL::ZERO, FGL::ZERO, FGL::ZERO])
            {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &pol_name.to_string(),
                    0,
                    index,
                    value,
                );
            }

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
            half_rows.push(*pr);
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
        for i in [9, 6, 7, 8, 10, 11] {
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &C.to_string(),
                i,
                hr.row,
                FGL::ZERO,
            );
        }
    }

    // 4. Generate Custom Gates
    let custom_gates_info = &plonk_setup_info.custom_gates_info;
    for (i, cgu) in r1cs.custom_gates_uses.iter().enumerate() {
        if (i % 10000) == 0 {
            log::trace!("Processing custom gates... {}/{}", i, r1cs.custom_gates_uses.len());
        }
        if cgu.id == custom_gates_info.poseidon_id {
            assert_eq!(cgu.signals.len(), 31 * 12);
            for j in 0..31 {
                let index = r + j;
                for k in 0..12 {
                    s_map[k][r + j] = cgu.signals[j * 12 + k];
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &C.to_string(),
                        k,
                        index,
                        FGL::from(CPOSEIDON[j * 12 + k]),
                    );
                }
                for pol_name in [GATE, CMULADD, EVPOL4, FFT4] {
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &pol_name.to_string(),
                        0,
                        index,
                        FGL::ZERO,
                    );
                }
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &POSEIDON12.to_string(),
                    0,
                    index,
                    if j < 30 { FGL::ONE } else { FGL::ZERO },
                );
                let tt = if !(4..26).contains(&j) { FGL::ZERO } else { FGL::ONE };
                let tt = if j < 30 { tt } else { FGL::ZERO };
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &PARTIAL.to_string(),
                    0,
                    index,
                    tt,
                );
            }
            r += 31;
        } else if cgu.id == custom_gates_info.c_mul_add_id {
            if r < n_used {
                for (j, map) in s_map.iter_mut().enumerate().take(12) {
                    map[r] = cgu.signals[j];
                }
            }
            let index = r;
            for pol_name in [GATE, POSEIDON12, PARTIAL, EVPOL4, FFT4] {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &pol_name.to_string(),
                    0,
                    index,
                    FGL::ZERO,
                );
            }
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &CMULADD.to_string(),
                0,
                index,
                FGL::ONE,
            );
            for i in 0..12 {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    i,
                    index,
                    if i == 9 || i == 10 { FGL::ONE } else { FGL::ZERO },
                );
            }

            r += 1;
        } else if CustomGateInfo::check_fft_param_defined(&custom_gates_info.fft_params, cgu.id) {
            for (j, map) in s_map.iter_mut().enumerate().take(12) {
                map[r] = cgu.signals[j];
                map[r + 1] = cgu.signals[12 + j];
            }
            let index = r;
            for pol_name in [GATE, POSEIDON12, CMULADD, PARTIAL, EVPOL4] {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &pol_name.to_string(),
                    0,
                    index,
                    FGL::ZERO,
                );
            }
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &FFT4.to_string(),
                0,
                index,
                FGL::ONE,
            );
            let index = r + 1;
            for pol_name in [GATE, POSEIDON12, CMULADD, PARTIAL, EVPOL4, FFT4] {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &pol_name.to_string(),
                    0,
                    index,
                    FGL::ZERO,
                );
            }

            let t = custom_gates_info.fft_params[&(cgu.id as usize)][3];
            let scale = custom_gates_info.fft_params[&(cgu.id as usize)][2];
            let incW = custom_gates_info.fft_params[&(cgu.id as usize)][1];
            let firstW = custom_gates_info.fft_params[&(cgu.id as usize)][0];
            let firstW2 = firstW * firstW;

            let index = r;
            if t.as_int() == 4 {
                for i in [6, 7, 8] {
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &C.to_string(),
                        i,
                        index,
                        FGL::ZERO,
                    );
                }
                for (i, value) in [0, 1, 2, 3, 4, 5].iter().zip(
                    [
                        scale,
                        scale * firstW2,
                        scale * firstW,
                        scale * firstW * firstW2,
                        scale * firstW * incW,
                        scale * firstW * firstW2 * incW,
                    ]
                    .iter(),
                ) {
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &C.to_string(),
                        *i,
                        index,
                        *value,
                    );
                }
            } else if t.as_int() == 2 {
                for i in [0, 1, 2, 3, 4, 5] {
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &C.to_string(),
                        i,
                        index,
                        FGL::ZERO,
                    );
                }
                for (i, value) in
                    [6, 7, 8].iter().zip([scale, scale * firstW, scale * firstW * incW].iter())
                {
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &C.to_string(),
                        *i,
                        index,
                        *value,
                    );
                }
            } else {
                panic!("invalit FFT4 type: {}", t);
            }

            let index = r;
            for i in [9, 10, 11] {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    i,
                    index,
                    FGL::ZERO,
                );
            }
            for k in 0..12 {
                let index = r + 1;
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    k,
                    index,
                    FGL::ZERO,
                );
            }
            r += 2;
        } else if cgu.id == custom_gates_info.ev_pol_id {
            for (j, map) in s_map.iter_mut().enumerate().take(12) {
                map[r] = cgu.signals[j];
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    j,
                    r,
                    FGL::ZERO,
                );
            }
            for (j, map) in s_map.iter_mut().enumerate().take(9) {
                map[r + 1] = cgu.signals[12 + j];
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    j,
                    r + 1,
                    FGL::ZERO,
                );
            }
            for (j, map) in s_map.iter_mut().enumerate().take(12).skip(9) {
                map[r + 1] = 0;

                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &C.to_string(),
                    j,
                    r + 1,
                    FGL::ZERO,
                );
            }
            let index = r;
            for pol_name in [GATE, POSEIDON12, CMULADD, PARTIAL, FFT4] {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &pol_name.to_string(),
                    0,
                    index,
                    FGL::ZERO,
                );
            }
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &EVPOL4.to_string(),
                0,
                index,
                FGL::ONE,
            );

            let index = r + 1;
            for pol_name in [GATE, POSEIDON12, CMULADD, PARTIAL, EVPOL4, FFT4] {
                const_pols.set_matrix(
                    pil,
                    &Compressor.to_string(),
                    &pol_name.to_string(),
                    0,
                    index,
                    FGL::ZERO,
                );
            }
            r += 2
        } else {
            panic!("Custom gate not defined: {}", cgu.id);
        }
    }

    // 5. Calculate S Polynomials
    let N = 1 << plonk_setup_info.n_bits;
    let ks = helper::get_ks(11);
    let mut w = FGL::ONE;
    for i in 0..N {
        if (i % 10000) == 0 {
            log::trace!("Preparing S... {}/{}", i, N);
        }
        const_pols.set_matrix(pil, &Compressor.to_string(), &S.to_string(), 0, i, w);
        for j in 1..12 {
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &S.to_string(),
                j,
                i,
                w * ks[j - 1],
            );
        }
        w = w * (starky::constant::MG.0[plonk_setup_info.n_bits]);
    }

    #[derive(Debug)]
    struct Grid {
        row: usize,
        col: usize,
    }
    let mut last_signal: BTreeMap<u64, Grid> = BTreeMap::new();
    for i in 0..r {
        if (i % 10000) == 0 {
            log::trace!("Connection S... {}/{}", i, r);
        }
        for (j, map) in s_map.iter_mut().enumerate().take(12) {
            if i < n_used {
                let key = map[i];
                if key == 0 {
                    continue;
                }
                let ls = last_signal.get(&key);
                if let Some(ls) = ls {
                    // connect and swap the value.
                    let left = const_pols.get(
                        pil,
                        &Compressor.to_string(),
                        &S.to_string(),
                        ls.col,
                        ls.row,
                    );
                    let right = const_pols.get(pil, &Compressor.to_string(), &S.to_string(), j, i);
                    const_pols.set_matrix(pil, &Compressor.to_string(), &S.to_string(), j, i, left);
                    const_pols.set_matrix(
                        pil,
                        &Compressor.to_string(),
                        &S.to_string(),
                        ls.col,
                        ls.row,
                        right,
                    );
                } else {
                    last_signal.insert(key, Grid { col: j, row: i });
                }
            }
        }
    }

    // 6. Fill unused rows.
    while r < N {
        if (r % 100000) == 0 {
            log::trace!("Empty gates... {}/{}", r, N);
        }
        let index = r;
        for pol_name in [EVPOL4, CMULADD, GATE, POSEIDON12, PARTIAL, FFT4] {
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &pol_name.to_string(),
                0,
                index,
                FGL::ZERO,
            );
        }
        for k in 0..12 {
            const_pols.set_matrix(
                pil,
                &Compressor.to_string(),
                &C.to_string(),
                k,
                index,
                FGL::ZERO,
            );
        }
        r += 1;
    }
    // construct Lagrange Basis Polynomial: Li(x)
    for i in 0..n_public_rows {
        let np = format!("L{}", i + 1);
        for j in 0..N {
            const_pols.set_matrix(pil, &Global.to_string(), &np, 0, j, FGL::ZERO);
        }
        const_pols.set_matrix(pil, &Global.to_string(), &np, 0, i, FGL::ONE);
    }

    (const_pols, s_map)
}
