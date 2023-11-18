#![allow(non_snake_case)]
use crate::constant::POSEIDON_CONSTANTS_OPT_AVX2;
use crate::poseidon_constants_avx as constants;
use algebraic::arch::x86_64::avx2_field_gl::Avx2GoldilocksField;
use algebraic::packed::PackedField;
use core::arch::x86_64::*;
use core::mem;
use plonky::field_gl::Fr as FGL;
use plonky::field_gl::FrRepr;
use plonky::Field;
use plonky::PrimeField;

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

    ConstantsAvx2 {
        c,
        m,
        p,
        s,
        n_rounds_f: 8,
        n_rounds_p: 22,
    }
}

pub struct Poseidon;

impl Default for Poseidon {
    fn default() -> Self {
        Self::new()
    }
}

impl Poseidon {
    pub fn new() -> Poseidon {
        Self {}
    }

    // #[inline(always)]
    // unsafe fn extract_u64s_from_m256i(value: __m256i) -> [u64; 4] {
    //     mem::transmute(value)
    // }

    #[inline(always)]
    fn pow7_avx2(
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
        c: Vec<FrRepr>,
    ) {
        let c0 = Avx2GoldilocksField::pack_slice(&c[0..4])[0];
        let c1 = Avx2GoldilocksField::pack_slice(&c[4..8])[0];
        let c2 = Avx2GoldilocksField::pack_slice(&c[8..12])[0];
        *st0 = *st0 + c0;
        *st1 = *st1 + c1;
        *st2 = *st2 + c2;
    }

    #[inline(always)]
    unsafe fn mmult_avx(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
        p: Vec<FrRepr>,
    ) {
        let mut tmp0 = Avx2GoldilocksField::ZEROS;
        let mut tmp1 = Avx2GoldilocksField::ZEROS;
        let mut tmp2 = Avx2GoldilocksField::ZEROS;
        Self::mmult_avx_4x12(&mut tmp0, *st0, *st1, *st2, p[0..48].to_vec());
        Self::mmult_avx_4x12(&mut tmp1, *st0, *st1, *st2, p[48..96].to_vec());
        Self::mmult_avx_4x12(&mut tmp2, *st0, *st1, *st2, p[96..144].to_vec());
        *st0 = tmp0;
        *st1 = tmp1;
        *st2 = tmp2;
    }

    // Dense matrix-vector product
    #[inline]
    unsafe fn mmult_avx_4x12(
        tmp: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: Vec<FrRepr>,
    ) {
        let mut r0 = Avx2GoldilocksField::ZEROS;
        let mut r1 = Avx2GoldilocksField::ZEROS;
        let mut r2 = Avx2GoldilocksField::ZEROS;
        let mut r3 = Avx2GoldilocksField::ZEROS;
        Self::spmv_avx_4x12(&mut r0, st0, st1, st2, m[0..12].to_vec());
        Self::spmv_avx_4x12(&mut r1, st0, st1, st2, m[12..24].to_vec());
        Self::spmv_avx_4x12(&mut r2, st0, st1, st2, m[24..36].to_vec());
        Self::spmv_avx_4x12(&mut r3, st0, st1, st2, m[36..48].to_vec());
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

    #[inline]
    unsafe fn spmv_avx_4x12(
        r: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: Vec<FrRepr>,
    ) {
        let m0 = Avx2GoldilocksField::pack_slice(&m[0..4])[0];
        let m1 = Avx2GoldilocksField::pack_slice(&m[4..8])[0];
        let m2 = Avx2GoldilocksField::pack_slice(&m[8..12])[0];
        *r = (st0 * m0) + (st1 * m1) + (st2 * m2)
    }

    #[inline(always)]
    unsafe fn mmult_avx_8(
        st0: &mut Avx2GoldilocksField,
        st1: &mut Avx2GoldilocksField,
        st2: &mut Avx2GoldilocksField,
        m: Vec<FrRepr>,
    ) {
        let mut tmp0 = Avx2GoldilocksField::ZEROS;
        let mut tmp1 = Avx2GoldilocksField::ZEROS;
        let mut tmp2 = Avx2GoldilocksField::ZEROS;
        Self::mmult_avx_4x12_8(&mut tmp0, *st0, *st1, *st2, m[0..48].to_vec());
        Self::mmult_avx_4x12_8(&mut tmp1, *st0, *st1, *st2, m[48..96].to_vec());
        Self::mmult_avx_4x12_8(&mut tmp2, *st0, *st1, *st2, m[96..144].to_vec());
        *st0 = tmp0;
        *st1 = tmp1;
        *st2 = tmp2;
    }

    // Dense matrix-vector product
    #[inline]
    unsafe fn mmult_avx_4x12_8(
        tmp: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: Vec<FrRepr>,
    ) {
        let mut r0 = Avx2GoldilocksField::ZEROS;
        let mut r1 = Avx2GoldilocksField::ZEROS;
        let mut r2 = Avx2GoldilocksField::ZEROS;
        let mut r3 = Avx2GoldilocksField::ZEROS;
        Self::spmv_avx_4x12_8(&mut r0, st0, st1, st2, m[0..12].to_vec());
        Self::spmv_avx_4x12_8(&mut r1, st0, st1, st2, m[12..24].to_vec());
        Self::spmv_avx_4x12_8(&mut r2, st0, st1, st2, m[24..36].to_vec());
        Self::spmv_avx_4x12_8(&mut r3, st0, st1, st2, m[36..48].to_vec());
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

    #[inline]
    unsafe fn spmv_avx_4x12_8(
        r: &mut Avx2GoldilocksField,
        st0: Avx2GoldilocksField,
        st1: Avx2GoldilocksField,
        st2: Avx2GoldilocksField,
        m: Vec<FrRepr>,
    ) {
        let m0 = Avx2GoldilocksField::pack_slice(&m[0..4])[0];
        let m1 = Avx2GoldilocksField::pack_slice(&m[4..8])[0];
        let m2 = Avx2GoldilocksField::pack_slice(&m[8..12])[0];
        let mut c0_h = Avx2GoldilocksField::ZEROS;
        let mut c0_l = Avx2GoldilocksField::ZEROS;
        let mut c1_h = Avx2GoldilocksField::ZEROS;
        let mut c1_l = Avx2GoldilocksField::ZEROS;
        let mut c2_h = Avx2GoldilocksField::ZEROS;
        let mut c2_l = Avx2GoldilocksField::ZEROS;
        Self::mult_avx_72(&mut c0_h, &mut c0_l, st0, m0);
        Self::mult_avx_72(&mut c1_h, &mut c1_l, st1, m1);
        Self::mult_avx_72(&mut c2_h, &mut c2_l, st2, m2);
        let c_h = c0_h + c1_h + c2_h;
        let c_l = c0_l + c1_l + c2_l;
        *r = Avx2GoldilocksField::reduce(c_h.get(), c_l.get())
    }

    #[inline]
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

    pub unsafe fn hash(
        &self,
        inp: &Vec<FGL>,
        init_state: &[FGL],
        out: usize,
    ) -> Result<Vec<FGL>, String> {
        self.hash_inner(inp, init_state, out)
    }

    unsafe fn hash_inner(
        &self,
        inp: &Vec<FGL>,
        init_state: &[FGL],
        out: usize,
    ) -> Result<Vec<FGL>, String> {
        if inp.len() != 8 {
            return Err(format!("Wrong inputs length {} != 8", inp.len(),));
        }
        if init_state.len() != 4 {
            return Err(format!("Capacity inputs length {} != 4", init_state.len(),));
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

        let mut part0 = state[0..4].to_vec();
        let mut part1 = state[4..8].to_vec();
        let mut part2 = state[8..12].to_vec();

        let mut st0 = Avx2GoldilocksField::pack_slice_mut(&mut part0)[0];
        let mut st1 = Avx2GoldilocksField::pack_slice_mut(&mut part1)[0];
        let mut st2 = Avx2GoldilocksField::pack_slice_mut(&mut part2)[0];

        Self::add_avx(&mut st0, &mut st1, &mut st2, (&C[0..12]).to_vec());
        for r in 0..(n_rounds_f / 2 - 1) {
            Self::pow7_avx2(&mut st0, &mut st1, &mut st2);
            Self::add_avx(
                &mut st0,
                &mut st1,
                &mut st2,
                (&C[(r + 1) * 12..((r + 1) * 12 + 12)]).to_vec(),
            );
            Self::mmult_avx_8(&mut st0, &mut st1, &mut st2, (&M[0..144]).to_vec());
        }
        Self::pow7_avx2(&mut st0, &mut st1, &mut st2);
        Self::add_avx(&mut st0, &mut st1, &mut st2, (&C[48..60]).to_vec());
        Self::mmult_avx(&mut st0, &mut st1, &mut st2, (&P[0..144]).to_vec());

        let state_u64s = (unsafe { Self::extract_u64s_from_m256i(st0.get()) });
        println!("ok! pow7_u64s- {:?}", state_u64s);

        Ok(_state[0..out].to_vec())

        //         let mut tmp_state = vec![FGL::ZERO; t];
        //         for r in 0..(n_rounds_f / 2 - 1) {
        // state.iter_mut().for_each(Self::pow7);
        //
        //             println!("pow7[{}]: {:?}", r, state);
        //             state.iter_mut().enumerate().for_each(|(i, a)| {
        //                 a.add_assign(&C[(r + 1) * t + i]);
        //             });

        //             let sz = state.len();
        //             tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
        //                 let mut acc = FGL::ZERO;
        //                 for j in 0..sz {
        //                     let mut tmp = M[j][i];
        //                     tmp.mul_assign(&state[j]);
        //                     acc.add_assign(&tmp);
        //                 }
        //                 *out = acc;
        //             });
        //             state
        //                 .iter_mut()
        //                 .zip(tmp_state.iter())
        //                 .for_each(|(out, inp)| {
        //                     *out = *inp;
        //                 });
        //         }
        //         // println!("0- {:?}", state);
        //         // state.iter_mut().for_each(Self::pow7);
        //         state.chunks_exact_mut(4).for_each(|chunk| {
        //             let mut field_chunk = Avx2GoldilocksField([
        //                 chunk[0].into_repr(),
        //                 chunk[1].into_repr(),
        //                 chunk[2].into_repr(),
        //                 chunk[3].into_repr(),
        //             ]);

        //             Self::pow7_avx2(&mut field_chunk);

        //             for (i, field) in field_chunk.0.iter().enumerate() {
        //                 chunk[i] = FGL::from_repr(*field).unwrap();
        //             }
        //         });
        //         println!("00- {:?}", state);
        //         state.iter_mut().enumerate().for_each(|(i, a)| {
        //             a.add_assign(&C[(n_rounds_f / 2 - 1 + 1) * t + i]);
        //         }); //opt
        //         // println!("000- {:?}", state);
        //         let sz = state.len();
        //         tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
        //             let mut acc = FGL::ZERO;
        //             for j in 0..sz {
        //                 let mut tmp = P[j][i];
        //                 tmp.mul_assign(&state[j]);
        //                 acc.add_assign(&tmp);
        //             }
        //             *out = acc;
        //         });
        //         // println!("0000- {:?}", state);
        //         state
        //             .iter_mut()
        //             .zip(tmp_state.iter())
        //             .for_each(|(out, inp)| {
        //                 *out = *inp;
        //             });
        // // println!("1- {:?}", state);

        //         for r in 0..n_rounds_p {
        //             Self::pow7(&mut state[0]);
        //             state[0].add_assign(&C[(n_rounds_f / 2 + 1) * t + r]);

        //             let sz = state.len();
        //             let mut s0 = FGL::ZERO;
        //             for j in 0..sz {
        //                 let mut tmp = S[(t * 2 - 1) * r + j];
        //                 tmp.mul_assign(&state[j]);
        //                 s0.add_assign(&tmp);
        //             }

        //             for k in 1..t {
        //                 let mut tmp = S[(t * 2 - 1) * r + t + k - 1];
        //                 tmp.mul_assign(&state[0]);
        //                 state[k].add_assign(&tmp);
        //             }

        //             state[0] = s0;
        //         }
        // // println!("2- {:?}", state);
        //         for r in 0..(n_rounds_f / 2 - 1) {
        //             // state.iter_mut().for_each(Self::pow7);
        //             state.chunks_exact_mut(4).for_each(|chunk| {
        //                 let mut field_chunk = Avx2GoldilocksField([
        //                     chunk[0].into_repr(),
        //                     chunk[1].into_repr(),
        //                     chunk[2].into_repr(),
        //                     chunk[3].into_repr(),
        //                 ]);

        //                 Self::pow7_avx2(&mut field_chunk);

        //                 for (i, field) in field_chunk.0.iter().enumerate() {
        //                     chunk[i] = FGL::from_repr(*field).unwrap();
        //                 }
        //             });
        //             state.iter_mut().enumerate().for_each(|(i, a)| {
        //                 a.add_assign(&C[(n_rounds_f / 2 + 1) * t + n_rounds_p + r * t + i]);
        //             });

        //             let sz = state.len();
        //             tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
        //                 let mut acc = FGL::ZERO;
        //                 for j in 0..sz {
        //                     let mut tmp = M[j][i];
        //                     tmp.mul_assign(&state[j]);
        //                     acc.add_assign(&tmp);
        //                 }
        //                 *out = acc;
        //             });
        //             state
        //                 .iter_mut()
        //                 .zip(tmp_state.iter())
        //                 .for_each(|(out, inp)| {
        //                     *out = *inp;
        //                 });
        //         }

        //         // state.iter_mut().for_each(Self::pow7);
        //         state.chunks_exact_mut(4).for_each(|chunk| {
        //             let mut field_chunk = Avx2GoldilocksField([
        //                 chunk[0].into_repr(),
        //                 chunk[1].into_repr(),
        //                 chunk[2].into_repr(),
        //                 chunk[3].into_repr(),
        //             ]);

        //             Self::pow7_avx2(&mut field_chunk);

        //             for (i, field) in field_chunk.0.iter().enumerate() {
        //                 chunk[i] = FGL::from_repr(*field).unwrap();
        //             }
        //         });
        //         let sz = state.len();
        //         tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
        //             let mut acc = FGL::ZERO;
        //             for j in 0..sz {
        //                 let mut tmp = M[j][i];
        //                 tmp.mul_assign(&state[j]);
        //                 acc.add_assign(&tmp);
        //             }
        //             *out = acc;
        //         });
        //         state = tmp_state;

        //         Ok(state[0..out].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::arch::x86_64::avx2_poseidon_gl::*;
    use algebraic::arch::x86_64::avx2_field_gl::Avx2GoldilocksField;
    use algebraic::packed::PackedField;
    use plonky::field_gl::Fr as FGL;
    use plonky::PrimeField;
    use rand::Rand;

    // #[test]
    // fn test_pow7_avx2() {
    //     let mut rng = rand::thread_rng();
    //     let x = FGL::rand(&mut rng);
    //     let x7 = x * x * x * x * x * x * x;
    //     let a_arr = [x.into_repr(), x.into_repr(), x.into_repr(), x.into_repr()];
    //     let packed_a = Avx2GoldilocksField::from_slice(&a_arr);
    //     let mut x = *packed_a;
    //     Poseidon::pow7_avx2(&mut x);
    //     let arr_res = x.as_slice();
    //     assert_eq!(x7.into_repr(), arr_res[0]);
    // }

    #[test]
    fn test_poseidon_opt_hash_all_0_avx() {
        let poseidon = Poseidon::new();
        let input = vec![FGL::ZERO; 8];
        let state = vec![FGL::ZERO; 4];
        let res = unsafe { poseidon.hash(&input, &state, 4).unwrap() };
        let expected = vec![
            FGL::from(0x3c18a9786cb0b359u64),
            FGL::from(0xc4055e3364a246c3u64),
            FGL::from(0x7953db0ab48808f4u64),
            FGL::from(0xc71603f33a1144cau64),
        ];
        // assert_eq!(res, expected);
    }

    // #[test]
    // fn test_poseidon_opt_hash_1_11() {
    //     let poseidon = Poseidon::new();
    //     let input = (0u64..8).map(FGL::from).collect::<Vec<FGL>>();
    //     let state = (8u64..12).map(FGL::from).collect::<Vec<FGL>>();
    //     let res = poseidon.hash(&input, &state, 4).unwrap();
    //     let expected = vec![
    //         FGL::from(0xd64e1e3efc5b8e9eu64),
    //         FGL::from(0x53666633020aaa47u64),
    //         FGL::from(0xd40285597c6a8825u64),
    //         FGL::from(0x613a4f81e81231d2u64),
    //     ];
    //     assert_eq!(res, expected);
    // }

    // #[test]
    // fn test_poseidon_opt_hash_all_neg_1() {
    //     let poseidon = Poseidon::new();
    //     let init = FGL::ZERO - FGL::ONE;
    //     let input = vec![init; 8];
    //     let state = vec![init; 4];
    //     let res = poseidon.hash(&input, &state, 4).unwrap();
    //     let expected = vec![
    //         FGL::from(0xbe0085cfc57a8357u64),
    //         FGL::from(0xd95af71847d05c09u64),
    //         FGL::from(0xcf55a13d33c1c953u64),
    //         FGL::from(0x95803a74f4530e82u64),
    //     ];
    //     assert_eq!(res, expected);
    // }
}
