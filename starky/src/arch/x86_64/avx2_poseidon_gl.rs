#![allow(non_snake_case)]
use crate::constant::POSEIDON_CONSTANTS_OPT_AVX2;
use crate::poseidon_constants_avx2 as constants;
use anyhow::{bail, Result};
use core::arch::x86_64::*;
use fields::arch::x86_64::avx2_field_gl::Avx2GoldilocksField;
use fields::field_gl::{Fr as FGL, FrRepr};
use fields::packed::PackedField;
use fields::PrimeField;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ConstantsAvx2 {
    pub c: Vec<FrRepr>,
    pub m: Vec<FrRepr>,
    pub p: Vec<FrRepr>,
    pub s: Vec<FrRepr>,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,
}

pub fn load_constants_avx2() -> ConstantsAvx2 {
    let (c_str, m_str, p_str, s_str) = constants::constants();
    let mut c: Vec<FrRepr> = Vec::new();
    for v1 in c_str {
        c.push(FrRepr([v1]));
    }
    let mut m: Vec<FrRepr> = Vec::new();
    for v1 in m_str {
        m.push(FrRepr([v1]));
    }

    let mut p: Vec<FrRepr> = Vec::new();
    for v1 in p_str {
        p.push(FrRepr([v1]));
    }

    let mut s: Vec<FrRepr> = Vec::new();
    for v1 in s_str {
        s.push(FrRepr([v1]));
    }

    ConstantsAvx2 { c, m, p, s, n_rounds_f: 8, n_rounds_p: 22 }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Poseidon;

impl Default for Poseidon {
    fn default() -> Self {
        Self::new()
    }
}

#[inline(always)]
unsafe fn spmv_avx_4x12(
    r: &mut Avx2GoldilocksField,
    st0: Avx2GoldilocksField,
    st1: Avx2GoldilocksField,
    st2: Avx2GoldilocksField,
    m: &[FrRepr],
) {
    let m = Avx2GoldilocksField::pack_slice(m);
    *r = (st0 * m[0]) + (st1 * m[1]) + (st2 * m[2]);
}

impl Poseidon {
    pub fn new() -> Poseidon {
        Self {}
    }

    // #[inline(always)]
    // unsafe fn _extract_u64s_from_m256i(value: __m256i) -> [u64; 4] {
    //     mem::transmute(value)
    // }

    #[inline(always)]
    fn pow7(x: &mut Avx2GoldilocksField) {
        let aux = *x;
        *x = x.square();
        *x *= aux;
        *x = x.square();
        *x *= aux;
    }

    #[inline(always)]
    fn pow7_triple(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
    ) {
        let aux0 = *st0;
        let aux1 = *st1;
        let aux2 = *st2;
        *st0 = st0.square();
        *st1 = st1.square();
        *st2 = st2.square();
        *st0 *= aux0;
        *st1 *= aux1;
        *st2 *= aux2;
        *st0 = st0.square();
        *st1 = st1.square();
        *st2 = st2.square();
        *st0 *= aux0;
        *st1 *= aux1;
        *st2 *= aux2;
    }

    #[inline(always)]
    fn add_avx(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
        c: &[FrRepr],
    ) {
        let c = Avx2GoldilocksField::pack_slice(c);
        *st0 = *st0 + c[0];
        *st1 = *st1 + c[1];
        *st2 = *st2 + c[2];
    }

    #[inline(always)]
    fn mult_add_avx(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
        s0: Avx2GoldilocksField,
        s: &[FrRepr],
    ) {
        let s = Avx2GoldilocksField::pack_slice(s);
        *st0 = *st0 + s[0] * s0;
        *st1 = *st1 + s[1] * s0;
        *st2 = *st2 + s[2] * s0;
    }

    #[inline(always)]
    unsafe fn mmult_avx(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
        p: &[FrRepr],
    ) {
        let mut tmp0 = Avx2GoldilocksField::ZEROS;
        let mut tmp1 = Avx2GoldilocksField::ZEROS;
        let mut tmp2 = Avx2GoldilocksField::ZEROS;
        Self::mmult_avx_4x12(&mut tmp0, *st0, *st1, *st2, &p[0..48]);
        Self::mmult_avx_4x12(&mut tmp1, *st0, *st1, *st2, &p[48..96]);
        Self::mmult_avx_4x12(&mut tmp2, *st0, *st1, *st2, &p[96..144]);
        *st0 = tmp0;
        *st1 = tmp1;
        *st2 = tmp2;
    }

    // Dense matrix-vector product
    #[inline(always)]
    unsafe fn mmult_avx_4x12(
        tmp: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: &[FrRepr],
    ) {
        let mut r0 = Avx2GoldilocksField::ZEROS;
        let mut r1 = Avx2GoldilocksField::ZEROS;
        let mut r2 = Avx2GoldilocksField::ZEROS;
        let mut r3 = Avx2GoldilocksField::ZEROS;
        spmv_avx_4x12(&mut r0, st0, st1, st2, &m[0..12]);
        spmv_avx_4x12(&mut r1, st0, st1, st2, &m[12..24]);
        spmv_avx_4x12(&mut r2, st0, st1, st2, &m[24..36]);
        spmv_avx_4x12(&mut r3, st0, st1, st2, &m[36..48]);
        // Transpose: transform de 4x4 matrix stored in rows r0...r3 to the columns c0...c3
        let t0 = _mm256_permute2f128_si256(r0.get(), r2.get(), 0b00100000);
        let t1 = _mm256_permute2f128_si256(r1.get(), r3.get(), 0b00100000);
        let t2 = _mm256_permute2f128_si256(r0.get(), r2.get(), 0b00110001);
        let t3 = _mm256_permute2f128_si256(r1.get(), r3.get(), 0b00110001);
        let c0 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpacklo_pd(
            _mm256_castsi256_pd(t0),
            _mm256_castsi256_pd(t1),
        )));
        let c1 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpackhi_pd(
            _mm256_castsi256_pd(t0),
            _mm256_castsi256_pd(t1),
        )));
        let c2 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpacklo_pd(
            _mm256_castsi256_pd(t2),
            _mm256_castsi256_pd(t3),
        )));
        let c3 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpackhi_pd(
            _mm256_castsi256_pd(t2),
            _mm256_castsi256_pd(t3),
        )));
        // Add columns to obtain result
        *tmp = c0 + c1 + c2 + c3;
    }

    #[inline(always)]
    unsafe fn mmult_avx_8(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
        m: &[FrRepr],
    ) {
        let mut tmp0 = Avx2GoldilocksField::ZEROS;
        let mut tmp1 = Avx2GoldilocksField::ZEROS;
        let mut tmp2 = Avx2GoldilocksField::ZEROS;
        Self::mmult_avx_4x12_8(&mut tmp0, *st0, *st1, *st2, &m[0..48]);
        Self::mmult_avx_4x12_8(&mut tmp1, *st0, *st1, *st2, &m[48..96]);
        Self::mmult_avx_4x12_8(&mut tmp2, *st0, *st1, *st2, &m[96..144]);
        *st0 = tmp0;
        *st1 = tmp1;
        *st2 = tmp2;
    }

    // Dense matrix-vector product
    #[inline(always)]
    unsafe fn mmult_avx_4x12_8(
        tmp: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: &[FrRepr],
    ) {
        let mut r0 = Avx2GoldilocksField::ZEROS;
        let mut r1 = Avx2GoldilocksField::ZEROS;
        let mut r2 = Avx2GoldilocksField::ZEROS;
        let mut r3 = Avx2GoldilocksField::ZEROS;
        Self::spmv_avx_4x12_8(&mut r0, st0, st1, st2, &m[0..12]);
        Self::spmv_avx_4x12_8(&mut r1, st0, st1, st2, &m[12..24]);
        Self::spmv_avx_4x12_8(&mut r2, st0, st1, st2, &m[24..36]);
        Self::spmv_avx_4x12_8(&mut r3, st0, st1, st2, &m[36..48]);
        // Transpose: transform de 4x4 matrix stored in rows r0...r3 to the columns c0...c3
        let t0 = _mm256_permute2f128_si256(r0.get(), r2.get(), 0b00100000);
        let t1 = _mm256_permute2f128_si256(r1.get(), r3.get(), 0b00100000);
        let t2 = _mm256_permute2f128_si256(r0.get(), r2.get(), 0b00110001);
        let t3 = _mm256_permute2f128_si256(r1.get(), r3.get(), 0b00110001);
        let c0 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpacklo_pd(
            _mm256_castsi256_pd(t0),
            _mm256_castsi256_pd(t1),
        )));
        let c1 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpackhi_pd(
            _mm256_castsi256_pd(t0),
            _mm256_castsi256_pd(t1),
        )));
        let c2 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpacklo_pd(
            _mm256_castsi256_pd(t2),
            _mm256_castsi256_pd(t3),
        )));
        let c3 = Avx2GoldilocksField::new(_mm256_castpd_si256(_mm256_unpackhi_pd(
            _mm256_castsi256_pd(t2),
            _mm256_castsi256_pd(t3),
        )));
        // Add columns to obtain result
        *tmp = c0 + c1 + c2 + c3;
    }

    #[inline(always)]
    unsafe fn spmv_avx_4x12_8(
        r: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: &[FrRepr],
    ) {
        let m = Avx2GoldilocksField::pack_slice(&m);
        let mut c0_h = Avx2GoldilocksField::ZEROS;
        let mut c0_l = Avx2GoldilocksField::ZEROS;
        let mut c1_h = Avx2GoldilocksField::ZEROS;
        let mut c1_l = Avx2GoldilocksField::ZEROS;
        let mut c2_h = Avx2GoldilocksField::ZEROS;
        let mut c2_l = Avx2GoldilocksField::ZEROS;
        Self::mult_avx_72(&mut c0_h, &mut c0_l, st0, m[0]);
        Self::mult_avx_72(&mut c1_h, &mut c1_l, st1, m[1]);
        Self::mult_avx_72(&mut c2_h, &mut c2_l, st2, m[2]);
        let c_h = c0_h + c1_h + c2_h;
        let c_l = c0_l + c1_l + c2_l;
        *r = Avx2GoldilocksField::reduce(c_h.get(), c_l.get())
    }

    #[inline(always)]
    unsafe fn mult_avx_72(
        c_h: &mut Avx2GoldilocksField,
        c_l: &mut Avx2GoldilocksField,
        a: Avx2GoldilocksField,
        b: Avx2GoldilocksField,
    ) {
        // Obtain a_h in the lower 32 bits
        let a_h = _mm256_srli_epi64(a.get(), 32);
        //__m256i a_h = _mm256_castps_si256(_mm256_movehdup_ps(_mm256_castsi256_ps(a)));

        // c = (a_h+a_l)*(b_l)=a_h*b_l+a_l*b_l=c_hl+c_ll
        // note: _mm256_mul_epu32 uses only the lower 32bits of each chunk so a=a_l and b=b_l
        let c_hl = _mm256_mul_epu32(a_h, b.get());
        let c_ll = _mm256_mul_epu32(a.get(), b.get());

        // Bignum addition
        // Ranges: c_hl[95:32], c_ll[63:0]
        // parts that intersect must be added

        // LOW PART:
        // 1: r0 = c_hl + c_ll_h
        //    does not overflow: c_hl <= (2^32-1)*(2^8-1)< 2^40
        //                       c_ll_h <= 2^32-1
        //                       c_hl + c_ll_h <= 2^41
        let c_ll_h = _mm256_srli_epi64(c_ll, 32);
        let r0 = _mm256_add_epi64(c_hl, c_ll_h);

        // 2: c_l = r0_l | c_ll_l
        let r0_l = _mm256_slli_epi64(r0, 32);
        //__m256i r0_l = _mm256_castps_si256(_mm256_moveldup_ps(_mm256_castsi256_ps(r0)));
        *c_l = Avx2GoldilocksField::new(_mm256_blend_epi32(c_ll, r0_l, 0xaa));
        // HIGH PART: c_h =  r0_h
        *c_h = Avx2GoldilocksField::new(_mm256_srli_epi64(r0, 32));
    }

    pub fn hash(&self, inp: &Vec<FGL>, init_state: &[FGL], out: usize) -> Result<Vec<FGL>> {
        unsafe { self.hash_inner(inp, init_state, out) }
    }

    unsafe fn hash_inner(
        &self,
        inp: &Vec<FGL>,
        init_state: &[FGL],
        out: usize,
    ) -> Result<Vec<FGL>> {
        if inp.len() != 8 {
            bail!(format!("Wrong inputs length {} != 8", inp.len(),));
        }
        if init_state.len() != 4 {
            bail!(format!("Capacity inputs length {} != 4", init_state.len(),));
        }
        let t = 12;
        let n_rounds_f = POSEIDON_CONSTANTS_OPT_AVX2.n_rounds_f;
        let n_rounds_p = POSEIDON_CONSTANTS_OPT_AVX2.n_rounds_p;
        let C = &POSEIDON_CONSTANTS_OPT_AVX2.c;
        let S = &POSEIDON_CONSTANTS_OPT_AVX2.s;
        let M = &POSEIDON_CONSTANTS_OPT_AVX2.m;
        let P = &POSEIDON_CONSTANTS_OPT_AVX2.p;

        let mut _state = vec![FGL::ZERO; t];
        _state[0..8].clone_from_slice(inp);
        _state[8..].clone_from_slice(init_state);

        let state: Vec<_> = _state.iter().map(|x| x.into_repr()).collect();
        let mut state_vec = state.to_vec();
        let st = Avx2GoldilocksField::pack_slice_mut(&mut state_vec);
        let mut st0 = st[0];
        let mut st1 = st[1];
        let mut st2 = st[2];

        Self::add_avx(&mut st0, &mut st1, &mut st2, &C[0..12]);
        for r in 0..(n_rounds_f / 2 - 1) {
            Self::pow7_triple(&mut st0, &mut st1, &mut st2);
            Self::add_avx(&mut st0, &mut st1, &mut st2, &C[(r + 1) * 12..((r + 1) * 12 + 12)]);
            Self::mmult_avx_8(&mut st0, &mut st1, &mut st2, &M[0..144]);
        }
        Self::pow7_triple(&mut st0, &mut st1, &mut st2);
        Self::add_avx(&mut st0, &mut st1, &mut st2, &C[48..60]);
        Self::mmult_avx(&mut st0, &mut st1, &mut st2, &P[0..144]);

        for r in 0..n_rounds_p {
            let st0_slice = st0.as_slice_mut();
            let mut s_arr = { [st0_slice[0], FrRepr([0]), FrRepr([0]), FrRepr([0])] };
            let mut _st0 = Avx2GoldilocksField::from_slice_mut(&mut s_arr);

            Self::pow7(&mut _st0);
            let c_arr = { [C[(4 + 1) * 12 + r], FrRepr([0]), FrRepr([0]), FrRepr([0])] };
            let c = Avx2GoldilocksField::from_slice(&c_arr);
            *_st0 = *_st0 + *c;
            let st0_slice = st0.as_slice_mut();
            st0_slice[0] = _st0.as_slice_mut()[0];

            let mut tmp = Avx2GoldilocksField::ZEROS;
            spmv_avx_4x12(&mut tmp, st0, st1, st2, &S[12 * 2 * r..(12 * 2 * r + 12)]);
            let tmp_slice = tmp.as_slice_mut();
            let sum = FGL::from_repr(tmp_slice[0]).unwrap()
                + FGL::from_repr(tmp_slice[1]).unwrap()
                + FGL::from_repr(tmp_slice[2]).unwrap()
                + FGL::from_repr(tmp_slice[3]).unwrap();

            let tmp_arr = {
                [
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[0],
                ]
            };
            let s0 = Avx2GoldilocksField::from_slice(&tmp_arr);
            Self::mult_add_avx(
                &mut st0,
                &mut st1,
                &mut st2,
                *s0,
                &S[(12 * (2 * r + 1))..(12 * (2 * r + 2))],
            );

            let st0_slice = st0.as_slice_mut();
            st0_slice[0] = sum.into_repr();
        }

        for r in 0..(n_rounds_f / 2 - 1) {
            Self::pow7_triple(&mut st0, &mut st1, &mut st2);
            Self::add_avx(
                &mut st0,
                &mut st1,
                &mut st2,
                &C[((n_rounds_f / 2 + 1) * t + n_rounds_p + r * t)
                    ..((n_rounds_f / 2 + 1) * t + n_rounds_p + r * t + 12)],
            );
            Self::mmult_avx_8(&mut st0, &mut st1, &mut st2, &M[0..144]);
        }
        Self::pow7_triple(&mut st0, &mut st1, &mut st2);
        Self::mmult_avx(&mut st0, &mut st1, &mut st2, &M[0..144]);

        let st0_slice = st0.as_slice();

        let mut result_vec: Vec<FGL> = Vec::new();
        result_vec.extend(st0_slice.iter().map(|&repr| FGL::from_repr(repr).unwrap()));

        Ok(result_vec[..out].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::arch::x86_64::avx2_poseidon_gl::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_poseidon_opt_hash_all_0_avx() {
        let poseidon = Poseidon::new();
        let input = vec![FGL::ZERO; 8];
        let state = vec![FGL::ZERO; 4];

        let start = Instant::now();
        let res = poseidon.hash(&input, &state, 4).unwrap();
        let hash_avx2_duration = start.elapsed();
        log::debug!("hash_avx2_duration_0: {:?}", hash_avx2_duration);

        let expected = vec![
            FGL::from(0x3c18a9786cb0b359u64),
            FGL::from(0xc4055e3364a246c3u64),
            FGL::from(0x7953db0ab48808f4u64),
            FGL::from(0xc71603f33a1144cau64),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_poseidon_opt_hash_1_11_avx() {
        let poseidon = Poseidon::new();
        let input = (0u64..8).map(FGL::from).collect::<Vec<FGL>>();
        let state = (8u64..12).map(FGL::from).collect::<Vec<FGL>>();
        let start = Instant::now();
        let res = poseidon.hash(&input, &state, 4).unwrap();
        let hash_avx2_duration = start.elapsed();
        log::debug!("hash_avx2_duration_1: {:?}", hash_avx2_duration);

        let expected = vec![
            FGL::from(0xd64e1e3efc5b8e9eu64),
            FGL::from(0x53666633020aaa47u64),
            FGL::from(0xd40285597c6a8825u64),
            FGL::from(0x613a4f81e81231d2u64),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_poseidon_opt_hash_all_neg_1_avx() {
        let poseidon = Poseidon::new();
        let init = FGL::ZERO - FGL::ONE;
        let input = vec![init; 8];
        let state = vec![init; 4];
        let start = Instant::now();
        let res = poseidon.hash(&input, &state, 4).unwrap();
        let hash_avx2_duration = start.elapsed();
        log::debug!("hash_avx2_duration_2: {:?}", hash_avx2_duration);

        let expected = vec![
            FGL::from(0xbe0085cfc57a8357u64),
            FGL::from(0xd95af71847d05c09u64),
            FGL::from(0xcf55a13d33c1c953u64),
            FGL::from(0x95803a74f4530e82u64),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_poseidon_opt_hash_1_11_avx_average() {
        let poseidon = Poseidon::new();
        let input = (0u64..8).map(FGL::from).collect::<Vec<FGL>>();
        let state = (8u64..12).map(FGL::from).collect::<Vec<FGL>>();

        let mut total_duration = Duration::new(0, 0);
        let iterations = 100;

        for _ in 0..iterations {
            let start = Instant::now();
            let _res = poseidon.hash(&input, &state, 4).unwrap();
            total_duration += start.elapsed();
        }

        let average_duration = total_duration / iterations;
        log::debug!("Average hash_avx2_duration_1: {:?}", average_duration);
    }

    #[test]
    fn test_poseidon_opt_hash_all_neg_1_avx_average() {
        let poseidon = Poseidon::new();
        let init = FGL::ZERO - FGL::ONE;
        let input = vec![init; 8];
        let state = vec![init; 4];

        let mut total_duration = Duration::new(0, 0);
        let iterations = 100;

        for _ in 0..iterations {
            let start = Instant::now();
            let _res = poseidon.hash(&input, &state, 4).unwrap();
            total_duration += start.elapsed();
        }

        let average_duration = total_duration / iterations;
        log::debug!("Average hash_avx2_duration_2: {:?}", average_duration);
    }

    #[test]
    fn test_spmv_avx_4x12() {
        let mut out = Avx2GoldilocksField::ZEROS;
        let in0 = Avx2GoldilocksField::from_slice(&[
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
        ]);
        let in1 = Avx2GoldilocksField::from_slice(&[
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
        ]);
        let in2 = Avx2GoldilocksField::from_slice(&[
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
            FrRepr([18446744069414584320]),
        ]);

        let in12 = [FrRepr([18446744069414584320]); 12];
        unsafe {
            spmv_avx_4x12(&mut out, *in0, *in1, *in2, &in12);
        };
        let tmp_slice = out.as_slice_mut();
        let _sum = FGL::from_repr(tmp_slice[0]).unwrap()
            + FGL::from_repr(tmp_slice[1]).unwrap()
            + FGL::from_repr(tmp_slice[2]).unwrap()
            + FGL::from_repr(tmp_slice[3]).unwrap();
    }
}
