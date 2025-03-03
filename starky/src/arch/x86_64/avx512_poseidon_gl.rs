#![allow(non_snake_case)]
use crate::constant::POSEIDON_CONSTANTS_OPT_AVX512;
use crate::poseidon_constants_avx512 as constants;
use anyhow::bail;
use anyhow::Result;
use core::arch::x86_64::*;
use fields::arch::x86_64::avx512_field_gl::Avx512GoldilocksField;
use fields::field_gl::{Fr as FGL, FrRepr};
use fields::packed::PackedField;
use fields::PrimeField;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ConstantsAvx512 {
    pub c: Vec<FrRepr>,
    pub m: Vec<FrRepr>,
    pub p: Vec<FrRepr>,
    pub s: Vec<FrRepr>,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,
}

pub fn load_constants_avx512() -> ConstantsAvx512 {
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

    ConstantsAvx512 { c, m, p, s, n_rounds_f: 8, n_rounds_p: 22 }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Poseidon;

impl Default for Poseidon {
    fn default() -> Self {
        Self::new()
    }
}

#[inline(always)]
unsafe fn spmv_avx512_4x12(
    r: &mut Avx512GoldilocksField,
    st0: Avx512GoldilocksField,
    st1: Avx512GoldilocksField,
    st2: Avx512GoldilocksField,
    m: &[FrRepr],
) {
    let m = Avx512GoldilocksField::pack_slice(m);
    *r = (st0 * m[0]) + (st1 * m[1]) + (st2 * m[2]);
}

impl Poseidon {
    pub fn new() -> Poseidon {
        Self {}
    }

    // #[inline(always)]
    // unsafe fn _extract_u64s_from_m512i(value: __m512i) -> [u64; 8] {
    //     mem::transmute(value)
    // }

    #[inline(always)]
    fn pow7(x: &mut Avx512GoldilocksField) {
        let aux = *x;
        *x = x.square();
        *x *= aux;
        *x = x.square();
        *x *= aux;
    }

    #[inline(always)]
    fn pow7_triple(
        st0: &mut Avx512GoldilocksField,
        st1: &mut Avx512GoldilocksField,
        st2: &mut Avx512GoldilocksField,
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
    fn add_avx512(
        st0: &mut Avx512GoldilocksField,
        st1: &mut Avx512GoldilocksField,
        st2: &mut Avx512GoldilocksField,
        c: &[FrRepr],
    ) {
        let c = Avx512GoldilocksField::pack_slice(c);
        *st0 = *st0 + c[0];
        *st1 = *st1 + c[1];
        *st2 = *st2 + c[2];
    }

    #[inline(always)]
    fn mult_add_avx512(
        st0: &mut Avx512GoldilocksField,
        st1: &mut Avx512GoldilocksField,
        st2: &mut Avx512GoldilocksField,
        s0: Avx512GoldilocksField,
        s: &[FrRepr],
    ) {
        let s = Avx512GoldilocksField::pack_slice(s);
        *st0 = *st0 + s[0] * s0;
        *st1 = *st1 + s[1] * s0;
        *st2 = *st2 + s[2] * s0;
    }

    #[inline(always)]
    unsafe fn mmult_avx512(
        st0: &mut Avx512GoldilocksField,
        st1: &mut Avx512GoldilocksField,
        st2: &mut Avx512GoldilocksField,
        p: &[FrRepr],
    ) {
        let mut tmp0 = Avx512GoldilocksField::ZEROS;
        let mut tmp1 = Avx512GoldilocksField::ZEROS;
        let mut tmp2 = Avx512GoldilocksField::ZEROS;
        Self::mmult_avx512_4x12(&mut tmp0, *st0, *st1, *st2, &p[0..96]);
        Self::mmult_avx512_4x12(&mut tmp1, *st0, *st1, *st2, &p[96..192]);
        Self::mmult_avx512_4x12(&mut tmp2, *st0, *st1, *st2, &p[192..288]);
        *st0 = tmp0;
        *st1 = tmp1;
        *st2 = tmp2;
    }

    // Dense matrix-vector product
    #[inline(always)]
    unsafe fn mmult_avx512_4x12(
        tmp: &mut Avx512GoldilocksField,
        st0: Avx512GoldilocksField,
        st1: Avx512GoldilocksField,
        st2: Avx512GoldilocksField,
        m: &[FrRepr],
    ) {
        let mut r0 = Avx512GoldilocksField::ZEROS;
        let mut r1 = Avx512GoldilocksField::ZEROS;
        let mut r2 = Avx512GoldilocksField::ZEROS;
        let mut r3 = Avx512GoldilocksField::ZEROS;
        spmv_avx512_4x12(&mut r0, st0, st1, st2, &m[0..24]);
        spmv_avx512_4x12(&mut r1, st0, st1, st2, &m[24..48]);
        spmv_avx512_4x12(&mut r2, st0, st1, st2, &m[48..72]);
        spmv_avx512_4x12(&mut r3, st0, st1, st2, &m[72..96]);
        // Transpose: transform de 4x4 matrix stored in rows r0...r3 to the columns c0...c3
        let (t0, t2) = Avx512GoldilocksField::interleave2(r0.get(), r2.get());
        let (t1, t3) = Avx512GoldilocksField::interleave2(r1.get(), r3.get());
        let c0 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpacklo_pd(
            _mm512_castsi512_pd(t0),
            _mm512_castsi512_pd(t1),
        )));
        let c1 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpackhi_pd(
            _mm512_castsi512_pd(t0),
            _mm512_castsi512_pd(t1),
        )));
        let c2 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpacklo_pd(
            _mm512_castsi512_pd(t2),
            _mm512_castsi512_pd(t3),
        )));
        let c3 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpackhi_pd(
            _mm512_castsi512_pd(t2),
            _mm512_castsi512_pd(t3),
        )));
        // Add columns to obtain result
        *tmp = c0 + c1 + c2 + c3;
    }

    #[inline(always)]
    unsafe fn mmult_avx512_8(
        st0: &mut Avx512GoldilocksField,
        st1: &mut Avx512GoldilocksField,
        st2: &mut Avx512GoldilocksField,
        m: &[FrRepr],
    ) {
        let mut tmp0 = Avx512GoldilocksField::ZEROS;
        let mut tmp1 = Avx512GoldilocksField::ZEROS;
        let mut tmp2 = Avx512GoldilocksField::ZEROS;
        Self::mmult_avx512_4x12_8(&mut tmp0, *st0, *st1, *st2, &m[0..96]);
        Self::mmult_avx512_4x12_8(&mut tmp1, *st0, *st1, *st2, &m[96..192]);
        Self::mmult_avx512_4x12_8(&mut tmp2, *st0, *st1, *st2, &m[192..288]);
        *st0 = tmp0;
        *st1 = tmp1;
        *st2 = tmp2;
    }

    // Dense matrix-vector product
    #[inline(always)]
    unsafe fn mmult_avx512_4x12_8(
        tmp: &mut Avx512GoldilocksField,
        st0: Avx512GoldilocksField,
        st1: Avx512GoldilocksField,
        st2: Avx512GoldilocksField,
        m: &[FrRepr],
    ) {
        let mut r0 = Avx512GoldilocksField::ZEROS;
        let mut r1 = Avx512GoldilocksField::ZEROS;
        let mut r2 = Avx512GoldilocksField::ZEROS;
        let mut r3 = Avx512GoldilocksField::ZEROS;
        Self::spmv_avx512_4x12_8(&mut r0, st0, st1, st2, &m[0..24]);
        Self::spmv_avx512_4x12_8(&mut r1, st0, st1, st2, &m[24..48]);
        Self::spmv_avx512_4x12_8(&mut r2, st0, st1, st2, &m[48..72]);
        Self::spmv_avx512_4x12_8(&mut r3, st0, st1, st2, &m[72..96]);
        // Transpose: transform de 4x4 matrix stored in rows r0...r3 to the columns c0...c3
        let (t0, t2) = Avx512GoldilocksField::interleave2(r0.get(), r2.get());
        let (t1, t3) = Avx512GoldilocksField::interleave2(r1.get(), r3.get());
        let c0 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpacklo_pd(
            _mm512_castsi512_pd(t0),
            _mm512_castsi512_pd(t1),
        )));
        let c1 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpackhi_pd(
            _mm512_castsi512_pd(t0),
            _mm512_castsi512_pd(t1),
        )));
        let c2 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpacklo_pd(
            _mm512_castsi512_pd(t2),
            _mm512_castsi512_pd(t3),
        )));
        let c3 = Avx512GoldilocksField::new(_mm512_castpd_si512(_mm512_unpackhi_pd(
            _mm512_castsi512_pd(t2),
            _mm512_castsi512_pd(t3),
        )));
        // Add columns to obtain result
        *tmp = c0 + c1 + c2 + c3;
    }

    #[inline(always)]
    unsafe fn spmv_avx512_4x12_8(
        r: &mut Avx512GoldilocksField,
        st0: Avx512GoldilocksField,
        st1: Avx512GoldilocksField,
        st2: Avx512GoldilocksField,
        m: &[FrRepr],
    ) {
        let m = Avx512GoldilocksField::pack_slice(&m);
        let mut c0_h = Avx512GoldilocksField::ZEROS;
        let mut c0_l = Avx512GoldilocksField::ZEROS;
        let mut c1_h = Avx512GoldilocksField::ZEROS;
        let mut c1_l = Avx512GoldilocksField::ZEROS;
        let mut c2_h = Avx512GoldilocksField::ZEROS;
        let mut c2_l = Avx512GoldilocksField::ZEROS;
        Self::mult_avx512_72(&mut c0_h, &mut c0_l, st0, m[0]);
        Self::mult_avx512_72(&mut c1_h, &mut c1_l, st1, m[1]);
        Self::mult_avx512_72(&mut c2_h, &mut c2_l, st2, m[2]);
        let c_h = c0_h + c1_h + c2_h;
        let c_l = c0_l + c1_l + c2_l;
        *r = Avx512GoldilocksField::reduce(c_h.get(), c_l.get())
    }

    #[inline(always)]
    unsafe fn mult_avx512_72(
        c_h: &mut Avx512GoldilocksField,
        c_l: &mut Avx512GoldilocksField,
        a: Avx512GoldilocksField,
        b: Avx512GoldilocksField,
    ) {
        // Obtain a_h in the lower 32 bits
        let a_h = _mm512_castps_si512(_mm512_movehdup_ps(_mm512_castsi512_ps(a.get())));

        // c = (a_h+a_l)*(b_l)=a_h*b_l+a_l*b_l=c_hl+c_ll
        // note: _mm512_mul_epu32 uses only the lower 32bits of each chunk so a=a_l and b=b_l
        let c_hl = _mm512_mul_epu32(a_h, b.get());
        let c_ll = _mm512_mul_epu32(a.get(), b.get());

        // Bignum addition
        // Ranges: c_hl[95:32], c_ll[63:0]
        // parts that intersect must be added

        // LOW PART:
        // 1: r0 = c_hl + c_ll_h
        //    does not overflow: c_hl <= (2^32-1)*(2^8-1)< 2^40
        //                       c_ll_h <= 2^32-1
        //                       c_hl + c_ll_h <= 2^41
        let c_ll_h = _mm512_srli_epi64(c_ll, 32);
        let r0 = _mm512_add_epi64(c_hl, c_ll_h);

        // 2: c_l = r0_l | c_ll_l
        const LO_32_BITS_MASK: __mmask16 = 0xAAAA;
        let r0_l = _mm512_castps_si512(_mm512_moveldup_ps(_mm512_castsi512_ps(r0)));
        *c_l = Avx512GoldilocksField::new(_mm512_mask_blend_epi32(LO_32_BITS_MASK, c_ll, r0_l));
        // HIGH PART: c_h =  r0_h
        *c_h = Avx512GoldilocksField::new(_mm512_srli_epi64(r0, 32));
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
        if inp.len() != 16 {
            bail!(format!("Wrong inputs length {} != 16", inp.len(),));
        }
        if init_state.len() != 8 {
            bail!(format!("Capacity inputs length {} != 8", init_state.len(),));
        }
        let t = 24;
        let n_rounds_f = POSEIDON_CONSTANTS_OPT_AVX512.n_rounds_f;
        let n_rounds_p = POSEIDON_CONSTANTS_OPT_AVX512.n_rounds_p;
        let C = &POSEIDON_CONSTANTS_OPT_AVX512.c;
        let S = &POSEIDON_CONSTANTS_OPT_AVX512.s;
        let M = &POSEIDON_CONSTANTS_OPT_AVX512.m;
        let P = &POSEIDON_CONSTANTS_OPT_AVX512.p;

        let mut _state = vec![FGL::ZERO; t];
        _state[0..16].clone_from_slice(inp);
        _state[16..].clone_from_slice(init_state);

        let state: Vec<_> = _state.iter().map(|x| x.into_repr()).collect();
        let mut state_vec = state.to_vec();
        let st = Avx512GoldilocksField::pack_slice_mut(&mut state_vec);
        let mut st0 = st[0];
        let mut st1 = st[1];
        let mut st2 = st[2];
        Self::add_avx512(&mut st0, &mut st1, &mut st2, &C[0..t]);

        for r in 0..(n_rounds_f / 2 - 1) {
            Self::pow7_triple(&mut st0, &mut st1, &mut st2);
            Self::add_avx512(&mut st0, &mut st1, &mut st2, &C[(r + 1) * t..((r + 1) * t + t)]);
            Self::mmult_avx512_8(&mut st0, &mut st1, &mut st2, &M[0..288]);
        }

        Self::pow7_triple(&mut st0, &mut st1, &mut st2);
        Self::add_avx512(&mut st0, &mut st1, &mut st2, &C[96..120]);
        Self::mmult_avx512(&mut st0, &mut st1, &mut st2, &P[0..288]);

        for r in 0..n_rounds_p {
            let st0_slice = st0.as_slice_mut();
            let mut s_arr = {
                [
                    st0_slice[0],
                    FrRepr([0]),
                    FrRepr([0]),
                    FrRepr([0]),
                    st0_slice[4],
                    FrRepr([0]),
                    FrRepr([0]),
                    FrRepr([0]),
                ]
            };
            let mut _st0 = Avx512GoldilocksField::from_slice_mut(&mut s_arr);

            Self::pow7(&mut _st0);
            let c_arr = {
                [
                    C[(4 + 1) * t + r],
                    FrRepr([0]),
                    FrRepr([0]),
                    FrRepr([0]),
                    C[(4 + 1) * t + r],
                    FrRepr([0]),
                    FrRepr([0]),
                    FrRepr([0]),
                ]
            };
            let c = Avx512GoldilocksField::from_slice(&c_arr);
            *_st0 = *_st0 + *c;
            let st0_slice = st0.as_slice_mut();
            st0_slice[0] = _st0.as_slice_mut()[0];
            st0_slice[4] = _st0.as_slice_mut()[4];

            let mut tmp = Avx512GoldilocksField::ZEROS;
            spmv_avx512_4x12(&mut tmp, st0, st1, st2, &S[t * 2 * r..(t * 2 * r + t)]);
            let tmp_slice = tmp.as_slice_mut();
            let sum0 = FGL::from_repr(tmp_slice[0]).unwrap()
                + FGL::from_repr(tmp_slice[1]).unwrap()
                + FGL::from_repr(tmp_slice[2]).unwrap()
                + FGL::from_repr(tmp_slice[3]).unwrap();
            let sum1 = FGL::from_repr(tmp_slice[4]).unwrap()
                + FGL::from_repr(tmp_slice[5]).unwrap()
                + FGL::from_repr(tmp_slice[6]).unwrap()
                + FGL::from_repr(tmp_slice[7]).unwrap();

            let tmp_arr = {
                [
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[0],
                    _st0.as_slice_mut()[4],
                    _st0.as_slice_mut()[4],
                    _st0.as_slice_mut()[4],
                    _st0.as_slice_mut()[4],
                ]
            };
            let s0 = Avx512GoldilocksField::from_slice(&tmp_arr);
            Self::mult_add_avx512(
                &mut st0,
                &mut st1,
                &mut st2,
                *s0,
                &S[(t * (2 * r + 1))..(t * (2 * r + 2))],
            );

            let st0_slice = st0.as_slice_mut();
            st0_slice[0] = sum0.into_repr();
            st0_slice[4] = sum1.into_repr();
        }

        for r in 0..(n_rounds_f / 2 - 1) {
            Self::pow7_triple(&mut st0, &mut st1, &mut st2);
            Self::add_avx512(
                &mut st0,
                &mut st1,
                &mut st2,
                &C[((n_rounds_f / 2 + 1) * t + n_rounds_p + r * t)
                    ..((n_rounds_f / 2 + 1) * t + n_rounds_p + r * t + t)],
            );
            Self::mmult_avx512_8(&mut st0, &mut st1, &mut st2, &M[0..288]);
        }

        Self::pow7_triple(&mut st0, &mut st1, &mut st2);
        Self::mmult_avx512(&mut st0, &mut st1, &mut st2, &M[0..288]);

        let st0_slice = st0.as_slice();

        let mut result_vec: Vec<FGL> = Vec::new();
        result_vec.extend(st0_slice.iter().map(|&repr| FGL::from_repr(repr).unwrap()));
        Ok(result_vec[..out].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::arch::x86_64::avx512_poseidon_gl::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_poseidon_opt_hash_all_0_avx() {
        let poseidon = Poseidon::new();
        let input = vec![FGL::ZERO; 16];
        let state = vec![FGL::ZERO; 8];

        let start = Instant::now();
        let res = poseidon.hash(&input, &state, 8).unwrap();
        let hash_avx512_duration = start.elapsed();
        log::debug!("hash_avx512_duration_0: {:?}", hash_avx512_duration);

        let expected = vec![
            FGL::from(0x3c18a9786cb0b359u64),
            FGL::from(0xc4055e3364a246c3u64),
            FGL::from(0x7953db0ab48808f4u64),
            FGL::from(0xc71603f33a1144cau64),
            FGL::from(0x3c18a9786cb0b359u64),
            FGL::from(0xc4055e3364a246c3u64),
            FGL::from(0x7953db0ab48808f4u64),
            FGL::from(0xc71603f33a1144cau64),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_poseidon_opt_hash_avx512() {
        let poseidon = Poseidon::new();
        let input = vec![
            FGL::from(0u64),
            FGL::from(1u64),
            FGL::from(2u64),
            FGL::from(3u64),
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::from(4u64),
            FGL::from(5u64),
            FGL::from(6u64),
            FGL::from(7u64),
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
        ];
        let state = vec![
            FGL::from(8u64),
            FGL::from(9u64),
            FGL::from(10u64),
            FGL::from(11u64),
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
        ];

        let start = Instant::now();
        let res = poseidon.hash(&input, &state, 8).unwrap();
        let hash_avx512_duration = start.elapsed();
        log::debug!("hash_avx512_duration_0: {:?}", hash_avx512_duration);

        let expected = vec![
            FGL::from(0xd64e1e3efc5b8e9eu64),
            FGL::from(0x53666633020aaa47u64),
            FGL::from(0xd40285597c6a8825u64),
            FGL::from(0x613a4f81e81231d2u64),
            FGL::from(0x3c18a9786cb0b359u64),
            FGL::from(0xc4055e3364a246c3u64),
            FGL::from(0x7953db0ab48808f4u64),
            FGL::from(0xc71603f33a1144cau64),
        ];
        assert_eq!(res, expected);
    }
    #[test]
    fn test_poseidon_opt_hash_1_11_avx512_average() {
        let poseidon = Poseidon::new();
        let input = vec![
            FGL::from(0u64),
            FGL::from(1u64),
            FGL::from(2u64),
            FGL::from(3u64),
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::from(4u64),
            FGL::from(5u64),
            FGL::from(6u64),
            FGL::from(7u64),
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
        ];
        let state = vec![
            FGL::from(8u64),
            FGL::from(9u64),
            FGL::from(10u64),
            FGL::from(11u64),
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
            FGL::ZERO,
        ];
        let mut total_duration = Duration::new(0, 0);
        let iterations = 100;

        for _ in 0..iterations {
            let start = Instant::now();
            let _res = poseidon.hash(&input, &state, 4).unwrap();
            total_duration += start.elapsed();
        }

        let average_duration = total_duration / iterations;
        log::debug!("Average hash_avx512_duration_1: {:?}", average_duration);
    }
}
