//! Porting from plonky2
//! https://github.com/0xPolygonZero/plonky2/blob/main/field/src/arch/x86_64/avx512_goldilocks_field.rs
//!
//! How to build/run/test:
//! RUSTFLAGS='-C target-feature=+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl' cargo build --features "avx512" --release
#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512))]
use crate::ff::*;
use crate::field_gl::{Fr, FrRepr as GoldilocksField};
use crate::packed::PackedField;
use core::arch::x86_64::*;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::transmute;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// AVX512 Goldilocks Field
///
/// Ideally `Avx512GoldilocksField` would wrap `__m512i`. Unfortunately, `__m512i` has an alignment
/// of 64B, which would preclude us from casting `[GoldilocksField; 8]` (alignment 8B) to
/// `Avx512GoldilocksField`. We need to ensure that `Avx512GoldilocksField` has the same alignment as
/// `GoldilocksField`. Thus we wrap `[GoldilocksField; 8]` and use the `new` and `get` methods to
/// convert to and from `__m512i`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Avx512GoldilocksField(pub [GoldilocksField; 8]);

const WIDTH: usize = 8;

impl Avx512GoldilocksField {
    #[inline]
    pub fn new(x: __m512i) -> Self {
        unsafe { transmute(x) }
    }
    #[inline]
    pub fn get(&self) -> __m512i {
        unsafe { transmute(*self) }
    }
    #[inline]
    pub fn interleave2(x: __m512i, y: __m512i) -> (__m512i, __m512i) {
        unsafe { interleave2(x, y) }
    }
    #[inline]
    pub fn reduce(x: __m512i, y: __m512i) -> Avx512GoldilocksField {
        Self::new(unsafe { reduce128((x, y)) })
    }
    #[inline]
    pub fn square(&self) -> Avx512GoldilocksField {
        Self::new(unsafe { square(self.get()) })
    }
}

unsafe impl PackedField for Avx512GoldilocksField {
    const WIDTH: usize = 8;

    type Scalar = GoldilocksField;

    const ZEROS: Self = Self([GoldilocksField([0]); 8]);
    const ONES: Self = Self([GoldilocksField([1]); 8]);
    #[inline]
    fn from_slice(slice: &[GoldilocksField]) -> &Self {
        assert_eq!(slice.len(), WIDTH);
        unsafe { &*slice.as_ptr().cast() }
    }
    #[inline]
    fn from_slice_mut(slice: &mut [GoldilocksField]) -> &mut Self {
        assert_eq!(slice.len(), WIDTH);
        unsafe { &mut *slice.as_mut_ptr().cast() }
    }
    #[inline]
    fn as_slice(&self) -> &[GoldilocksField] {
        &self.0[..]
    }
    #[inline]
    fn as_slice_mut(&mut self) -> &mut [GoldilocksField] {
        &mut self.0[..]
    }

    #[inline]
    fn interleave(&self, other: Self, block_len: usize) -> (Self, Self) {
        let (v0, v1) = (self.get(), other.get());
        let (res0, res1) = match block_len {
            1 => unsafe { interleave1(v0, v1) },
            2 => unsafe { interleave2(v0, v1) },
            4 => unsafe { interleave4(v0, v1) },
            8 => (v0, v1),
            _ => panic!("unsupported block_len"),
        };
        (Self::new(res0), Self::new(res1))
    }
}

impl Add<Self> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(unsafe { add(self.get(), rhs.get()) })
    }
}
impl Add<GoldilocksField> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn add(self, rhs: GoldilocksField) -> Self {
        self + Self::from(rhs)
    }
}
impl Add<Avx512GoldilocksField> for GoldilocksField {
    type Output = Avx512GoldilocksField;
    #[inline]
    fn add(self, rhs: Self::Output) -> Self::Output {
        Self::Output::from(self) + rhs
    }
}
impl AddAssign<Self> for Avx512GoldilocksField {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl AddAssign<GoldilocksField> for Avx512GoldilocksField {
    #[inline]
    fn add_assign(&mut self, rhs: GoldilocksField) {
        *self = *self + rhs;
    }
}

impl Debug for Avx512GoldilocksField {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", self.get())
    }
}

impl Default for Avx512GoldilocksField {
    #[inline]
    fn default() -> Self {
        Self::ZEROS
    }
}

impl Div<GoldilocksField> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn div(self, rhs: GoldilocksField) -> Self {
        let rhs_value = Fr::from_repr(rhs).unwrap();
        let rhs_inverse = rhs_value.inverse().unwrap().into_repr();
        self * rhs_inverse
    }
}
impl DivAssign<GoldilocksField> for Avx512GoldilocksField {
    #[inline]
    fn div_assign(&mut self, rhs: GoldilocksField) {
        let rhs_value = Fr::from_repr(rhs).unwrap();
        let rhs_inverse = rhs_value.inverse().unwrap().into_repr();
        *self *= rhs_inverse;
    }
}

impl From<GoldilocksField> for Avx512GoldilocksField {
    fn from(x: GoldilocksField) -> Self {
        Self([x; 8])
    }
}

impl Mul<Self> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(unsafe { mul(self.get(), rhs.get()) })
    }
}
impl Mul<GoldilocksField> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: GoldilocksField) -> Self {
        self * Self::from(rhs)
    }
}
impl Mul<Avx512GoldilocksField> for GoldilocksField {
    type Output = Avx512GoldilocksField;
    #[inline]
    fn mul(self, rhs: Avx512GoldilocksField) -> Self::Output {
        Self::Output::from(self) * rhs
    }
}
impl MulAssign<Self> for Avx512GoldilocksField {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl MulAssign<GoldilocksField> for Avx512GoldilocksField {
    #[inline]
    fn mul_assign(&mut self, rhs: GoldilocksField) {
        *self = *self * rhs;
    }
}

impl Neg for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(unsafe { neg(self.get()) })
    }
}

// impl Product for Avx512GoldilocksField {
//     #[inline]
//     fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.reduce(|x, y| x * y).unwrap_or(Self::ONES)
//     }
// }

impl Sub<Self> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(unsafe { sub(self.get(), rhs.get()) })
    }
}
impl Sub<GoldilocksField> for Avx512GoldilocksField {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: GoldilocksField) -> Self {
        self - Self::from(rhs)
    }
}
impl Sub<Avx512GoldilocksField> for GoldilocksField {
    type Output = Avx512GoldilocksField;
    #[inline]
    fn sub(self, rhs: Avx512GoldilocksField) -> Self::Output {
        Self::Output::from(self) - rhs
    }
}
impl SubAssign<Self> for Avx512GoldilocksField {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl SubAssign<GoldilocksField> for Avx512GoldilocksField {
    #[inline]
    fn sub_assign(&mut self, rhs: GoldilocksField) {
        *self = *self - rhs;
    }
}

// impl Sum for Avx512GoldilocksField {
//     #[inline]
//     fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.reduce(|x, y| x + y).unwrap_or(Self::ZEROS)
//     }
// }

const FIELD_ORDER: __m512i = unsafe { transmute([GOLDILOCKS_FIELD_ORDER; 8]) };
const EPSILON: __m512i = unsafe { transmute([GoldilocksField([4294967295u64]); 8]) };
// Goldilocks Order
const GOLDILOCKS_FIELD_ORDER: u64 = 0xFFFFFFFF00000001;

#[inline]
unsafe fn canonicalize(x: __m512i) -> __m512i {
    let mask = _mm512_cmpge_epu64_mask(x, FIELD_ORDER);
    _mm512_mask_sub_epi64(x, mask, x, FIELD_ORDER)
}

#[inline]
unsafe fn add_no_double_overflow_64_64(x: __m512i, y: __m512i) -> __m512i {
    let res_wrapped = _mm512_add_epi64(x, y);
    let mask = _mm512_cmplt_epu64_mask(res_wrapped, y); // mask set if add overflowed
    let res = _mm512_mask_sub_epi64(res_wrapped, mask, res_wrapped, FIELD_ORDER);
    res
}

#[inline]
unsafe fn sub_no_double_overflow_64_64(x: __m512i, y: __m512i) -> __m512i {
    let mask = _mm512_cmplt_epu64_mask(x, y); // mask set if sub will underflow (x < y)
    let res_wrapped = _mm512_sub_epi64(x, y);
    let res = _mm512_mask_add_epi64(res_wrapped, mask, res_wrapped, FIELD_ORDER);
    res
}

#[inline]
unsafe fn add(x: __m512i, y: __m512i) -> __m512i {
    let res_s = add_no_double_overflow_64_64(x, canonicalize(y));
    canonicalize(res_s)
}

#[inline]
unsafe fn sub(x: __m512i, y: __m512i) -> __m512i {
    sub_no_double_overflow_64_64(x, canonicalize(y))
}

#[inline]
unsafe fn neg(y: __m512i) -> __m512i {
    _mm512_sub_epi64(FIELD_ORDER, canonicalize(y))
}

const LO_32_BITS_MASK: __mmask16 = unsafe { transmute(0b0101010101010101u16) };

#[inline]
unsafe fn mul64_64(x: __m512i, y: __m512i) -> (__m512i, __m512i) {
    // We want to move the high 32 bits to the low position. The multiplication instruction ignores
    // the high 32 bits, so it's ok to just duplicate it into the low position. This duplication can
    // be done on port 5; bitshifts run on port 0, competing with multiplication.
    //   This instruction is only provided for 32-bit floats, not integers. Idk why Intel makes the
    // distinction; the casts are free and it guarantees that the exact bit pattern is preserved.
    // Using a swizzle instruction of the wrong domain (float vs int) does not increase latency
    // since Haswell.
    let x_hi = _mm512_castps_si512(_mm512_movehdup_ps(_mm512_castsi512_ps(x)));
    let y_hi = _mm512_castps_si512(_mm512_movehdup_ps(_mm512_castsi512_ps(y)));

    // All four pairwise multiplications
    let mul_ll = _mm512_mul_epu32(x, y);
    let mul_lh = _mm512_mul_epu32(x, y_hi);
    let mul_hl = _mm512_mul_epu32(x_hi, y);
    let mul_hh = _mm512_mul_epu32(x_hi, y_hi);

    // Bignum addition
    // Extract high 32 bits of mul_ll and add to mul_hl. This cannot overflow.
    let mul_ll_hi = _mm512_srli_epi64::<32>(mul_ll);
    let t0 = _mm512_add_epi64(mul_hl, mul_ll_hi);
    // Extract low 32 bits of t0 and add to mul_lh. Again, this cannot overflow.
    // Also, extract high 32 bits of t0 and add to mul_hh.
    let t0_lo = _mm512_and_si512(t0, EPSILON);
    let t0_hi = _mm512_srli_epi64::<32>(t0);
    let t1 = _mm512_add_epi64(mul_lh, t0_lo);
    let t2 = _mm512_add_epi64(mul_hh, t0_hi);
    // Lastly, extract the high 32 bits of t1 and add to t2.
    let t1_hi = _mm512_srli_epi64::<32>(t1);
    let res_hi = _mm512_add_epi64(t2, t1_hi);

    // Form res_lo by combining the low half of mul_ll with the low half of t1 (shifted into high
    // position).
    let t1_lo = _mm512_castps_si512(_mm512_moveldup_ps(_mm512_castsi512_ps(t1)));
    let res_lo = _mm512_mask_blend_epi32(LO_32_BITS_MASK, t1_lo, mul_ll);

    (res_hi, res_lo)
}

#[inline]
unsafe fn square64(x: __m512i) -> (__m512i, __m512i) {
    // Get high 32 bits of x. See comment in mul64_64_s.
    let x_hi = _mm512_castps_si512(_mm512_movehdup_ps(_mm512_castsi512_ps(x)));

    // All pairwise multiplications.
    let mul_ll = _mm512_mul_epu32(x, x);
    let mul_lh = _mm512_mul_epu32(x, x_hi);
    let mul_hh = _mm512_mul_epu32(x_hi, x_hi);

    // Bignum addition, but mul_lh is shifted by 33 bits (not 32).
    let mul_ll_hi = _mm512_srli_epi64::<33>(mul_ll);
    let t0 = _mm512_add_epi64(mul_lh, mul_ll_hi);
    let t0_hi = _mm512_srli_epi64::<31>(t0);
    let res_hi = _mm512_add_epi64(mul_hh, t0_hi);

    // Form low result by adding the mul_ll and the low 31 bits of mul_lh (shifted to the high
    // position).
    let mul_lh_lo = _mm512_slli_epi64::<33>(mul_lh);
    let res_lo = _mm512_add_epi64(mul_ll, mul_lh_lo);

    (res_hi, res_lo)
}

#[inline]
unsafe fn reduce128(x: (__m512i, __m512i)) -> __m512i {
    let (hi0, lo0) = x;
    let hi_hi0 = _mm512_srli_epi64::<32>(hi0);
    let lo1 = sub_no_double_overflow_64_64(lo0, hi_hi0);
    let t1 = _mm512_mul_epu32(hi0, EPSILON);
    let _lo2 = add_no_double_overflow_64_64(lo1, t1);
    let lo2 = canonicalize(_lo2);
    lo2
}

#[inline]
unsafe fn mul(x: __m512i, y: __m512i) -> __m512i {
    reduce128(mul64_64(x, y))
}

#[inline]
unsafe fn square(x: __m512i) -> __m512i {
    reduce128(square64(x))
}

#[inline]
unsafe fn interleave1(x: __m512i, y: __m512i) -> (__m512i, __m512i) {
    let a = _mm512_unpacklo_epi64(x, y);
    let b = _mm512_unpackhi_epi64(x, y);
    (a, b)
}

const INTERLEAVE2_IDX_A: __m512i =
    unsafe { transmute([0o00u64, 0o01u64, 0o10u64, 0o11u64, 0o04u64, 0o05u64, 0o14u64, 0o15u64]) };
const INTERLEAVE2_IDX_B: __m512i =
    unsafe { transmute([0o02u64, 0o03u64, 0o12u64, 0o13u64, 0o06u64, 0o07u64, 0o16u64, 0o17u64]) };

#[inline]
unsafe fn interleave2(x: __m512i, y: __m512i) -> (__m512i, __m512i) {
    let a = _mm512_permutex2var_epi64(x, INTERLEAVE2_IDX_A, y);
    let b = _mm512_permutex2var_epi64(x, INTERLEAVE2_IDX_B, y);
    (a, b)
}

#[inline]
unsafe fn interleave4(x: __m512i, y: __m512i) -> (__m512i, __m512i) {
    let a = _mm512_shuffle_i64x2::<0x44>(x, y);
    let b = _mm512_shuffle_i64x2::<0xee>(x, y);
    (a, b)
}

#[cfg(test)]
mod tests {
    use super::Avx512GoldilocksField;
    use crate::ff::*;
    use crate::field_gl::{Fr, FrRepr as GoldilocksField};
    use crate::packed::PackedField;
    use std::time::Instant;

    fn test_vals_a() -> [GoldilocksField; 8] {
        [
            GoldilocksField([18446744069414584320u64]),
            GoldilocksField([9087029921428221768u64]),
            GoldilocksField([2441288194761790662u64]),
            GoldilocksField([5646033492608483824u64]),
            GoldilocksField([2779181197214900072u64]),
            GoldilocksField([2989742820063487116u64]),
            GoldilocksField([727880025589250743u64]),
            GoldilocksField([3803926346107752679u64]),
        ]
    }
    fn test_vals_b() -> [GoldilocksField; 8] {
        [
            GoldilocksField([18446744069414584320u64]),
            GoldilocksField([11009798273260028228u64]),
            GoldilocksField([2028722748960791447u64]),
            GoldilocksField([7929433601095175579u64]),
            GoldilocksField([6632528436085461172u64]),
            GoldilocksField([2145438710786785567u64]),
            GoldilocksField([11821483668392863016u64]),
            GoldilocksField([15638272883309521929u64]),
        ]
    }

    #[test]
    fn test_add1() {
        let a_arr = test_vals_a();
        let b_arr = test_vals_b();
        let start = Instant::now();
        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_b = *Avx512GoldilocksField::from_slice(&b_arr);
        let packed_res = packed_a + packed_b;
        let arr_res = packed_res.as_slice();
        let avx512_duration = start.elapsed();
        let start = Instant::now();
        let expected = a_arr
            .iter()
            .zip(b_arr)
            .map(|(&a, b)| Fr::from_repr(a).unwrap() + Fr::from_repr(b).unwrap());
        let expected_values: Vec<Fr> = expected.collect();
        // println!("expected values: {:?}", expected_values);
        let non_accelerated_duration = start.elapsed();
        for (exp, &res) in expected_values.iter().zip(arr_res) {
            assert_eq!(res, exp.into_repr());
        }

        println!("test_add_AVX512_accelerated time: {:?}", avx512_duration);
        println!("test_add_Non_accelerated time: {:?}", non_accelerated_duration);
    }

    #[test]
    fn test_mul() {
        let a_arr = test_vals_a();
        let b_arr = test_vals_b();
        let start = Instant::now();
        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_b = *Avx512GoldilocksField::from_slice(&b_arr);
        let packed_res = packed_a * packed_b;
        let arr_res = packed_res.as_slice();
        let avx512_duration = start.elapsed();

        let start = Instant::now();
        let expected = a_arr
            .iter()
            .zip(b_arr)
            .map(|(&a, b)| Fr::from_repr(a).unwrap() * Fr::from_repr(b).unwrap());
        let expected_values: Vec<Fr> = expected.collect();
        let non_accelerated_duration = start.elapsed();
        // println!("expected values: {:?}", expected_values);

        for (exp, &res) in expected_values.iter().zip(arr_res) {
            assert_eq!(res, exp.into_repr());
        }

        println!("test_mul_AVX512_accelerated time: {:?}", avx512_duration);
        println!("test_mul_Non_accelerated time: {:?}", non_accelerated_duration);
    }

    #[test]
    fn test_div() {
        let a_arr = test_vals_a();
        let start = Instant::now();
        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_res = packed_a / GoldilocksField([7929433601095175579u64]);
        let arr_res = packed_res.as_slice();
        let avx512_duration = start.elapsed();
        // println!("arr_res: {:?}", arr_res);

        let start = Instant::now();
        let expected = a_arr.iter().map(|&a| {
            Fr::from_repr(a).unwrap()
                / Fr::from_repr(GoldilocksField([7929433601095175579u64])).unwrap()
        });
        let expected_values: Vec<Fr> = expected.collect();
        let non_accelerated_duration = start.elapsed();
        // println!("expected values: {:?}", expected_values);

        for (exp, &res) in expected_values.iter().zip(arr_res) {
            assert_eq!(res, exp.into_repr());
        }

        println!("test_div_AVX512_accelerated time: {:?}", avx512_duration);
        println!("test_div_Non_accelerated time: {:?}", non_accelerated_duration);
    }

    #[test]
    fn test_square() {
        let a_arr = test_vals_a();
        let start = Instant::now();
        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_res = packed_a.square();
        let arr_res = packed_res.as_slice();
        let avx512_duration = start.elapsed();
        // println!("arr_res: {:?}", arr_res);

        let start = Instant::now();
        let mut expected_values = Vec::new();
        for &a in &a_arr {
            match Fr::from_repr(a) {
                Ok(mut fr) => {
                    fr.square();
                    expected_values.push(fr);
                }
                Err(_) => {
                    continue;
                }
            }
        }
        let non_accelerated_duration = start.elapsed();
        // println!("expected values: {:?}", expected_values);
        for (exp, &res) in expected_values.iter().zip(arr_res) {
            assert_eq!(res, exp.into_repr());
        }
        println!("test_square_AVX512_accelerated time: {:?}", avx512_duration);
        println!("test_square_Non_accelerated time: {:?}", non_accelerated_duration);
    }

    #[test]
    fn test_neg() {
        let a_arr = test_vals_a();
        let start = Instant::now();
        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_res = -packed_a;
        let arr_res = packed_res.as_slice();
        let avx512_duration = start.elapsed();
        // println!("arr_res: {:?}", arr_res);

        let start = Instant::now();
        let expected = a_arr.iter().map(|&a| -Fr::from_repr(a).unwrap());
        let expected_values: Vec<Fr> = expected.collect();
        let non_accelerated_duration = start.elapsed();
        // println!("expected values: {:?}", expected_values);

        for (exp, &res) in expected_values.iter().zip(arr_res) {
            assert_eq!(res, exp.into_repr());
        }

        println!("test_neg_AVX512_accelerated time: {:?}", avx512_duration);
        println!("test_neg_Non_accelerated time: {:?}", non_accelerated_duration);
    }

    #[test]
    fn test_sub() {
        let a_arr = test_vals_a();
        let b_arr = test_vals_b();
        let start = Instant::now();
        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_b = *Avx512GoldilocksField::from_slice(&b_arr);
        let packed_res = packed_a - packed_b;
        let arr_res = packed_res.as_slice();
        let avx512_duration = start.elapsed();
        // println!("arr_res: {:?}", arr_res);

        let start = Instant::now();
        let expected = a_arr
            .iter()
            .zip(b_arr)
            .map(|(&a, b)| Fr::from_repr(a).unwrap() - Fr::from_repr(b).unwrap());
        let expected_values: Vec<Fr> = expected.collect();
        let non_accelerated_duration = start.elapsed();
        // println!("expected values: {:?}", expected_values);

        for (exp, &res) in expected_values.iter().zip(arr_res) {
            assert_eq!(res, exp.into_repr());
        }

        println!("test_sub_AVX512_accelerated time: {:?}", avx512_duration);
        println!("test_sub_Non_accelerated time: {:?}", non_accelerated_duration);
    }

    #[test]
    fn test_interleave_is_involution() {
        let a_arr = test_vals_a();
        let b_arr = test_vals_b();

        let packed_a = *Avx512GoldilocksField::from_slice(&a_arr);
        let packed_b = *Avx512GoldilocksField::from_slice(&b_arr);
        {
            // Interleave, then deinterleave.
            let (x, y) = packed_a.interleave(packed_b, 1);
            let (res_a, res_b) = x.interleave(y, 1);
            assert_eq!(res_a.as_slice(), a_arr);
            assert_eq!(res_b.as_slice(), b_arr);
        }
        {
            let (x, y) = packed_a.interleave(packed_b, 2);
            let (res_a, res_b) = x.interleave(y, 2);
            assert_eq!(res_a.as_slice(), a_arr);
            assert_eq!(res_b.as_slice(), b_arr);
        }
        {
            let (x, y) = packed_a.interleave(packed_b, 4);
            let (res_a, res_b) = x.interleave(y, 4);
            assert_eq!(res_a.as_slice(), a_arr);
            assert_eq!(res_b.as_slice(), b_arr);
        }
        {
            let (x, y) = packed_a.interleave(packed_b, 8);
            let (res_a, res_b) = x.interleave(y, 8);
            assert_eq!(res_a.as_slice(), a_arr);
            assert_eq!(res_b.as_slice(), b_arr);
        }
    }

    #[test]
    fn test_interleave() {
        let in_a: [GoldilocksField; 8] = [
            GoldilocksField([0u64]),
            GoldilocksField([1u64]),
            GoldilocksField([2u64]),
            GoldilocksField([3u64]),
            GoldilocksField([4u64]),
            GoldilocksField([5u64]),
            GoldilocksField([6u64]),
            GoldilocksField([7u64]),
        ];
        let in_b: [GoldilocksField; 8] = [
            GoldilocksField([10u64]),
            GoldilocksField([11u64]),
            GoldilocksField([12u64]),
            GoldilocksField([13u64]),
            GoldilocksField([14u64]),
            GoldilocksField([15u64]),
            GoldilocksField([16u64]),
            GoldilocksField([17u64]),
        ];
        let int1_a: [GoldilocksField; 8] = [
            GoldilocksField([0u64]),
            GoldilocksField([10u64]),
            GoldilocksField([2u64]),
            GoldilocksField([12u64]),
            GoldilocksField([4u64]),
            GoldilocksField([14u64]),
            GoldilocksField([6u64]),
            GoldilocksField([16u64]),
        ];
        let int1_b: [GoldilocksField; 8] = [
            GoldilocksField([1u64]),
            GoldilocksField([11u64]),
            GoldilocksField([3u64]),
            GoldilocksField([13u64]),
            GoldilocksField([5u64]),
            GoldilocksField([15u64]),
            GoldilocksField([7u64]),
            GoldilocksField([17u64]),
        ];
        let int2_a: [GoldilocksField; 8] = [
            GoldilocksField([0u64]),
            GoldilocksField([1u64]),
            GoldilocksField([10u64]),
            GoldilocksField([11u64]),
            GoldilocksField([4u64]),
            GoldilocksField([5u64]),
            GoldilocksField([14u64]),
            GoldilocksField([15u64]),
        ];
        let int2_b: [GoldilocksField; 8] = [
            GoldilocksField([2u64]),
            GoldilocksField([3u64]),
            GoldilocksField([12u64]),
            GoldilocksField([13u64]),
            GoldilocksField([6u64]),
            GoldilocksField([7u64]),
            GoldilocksField([16u64]),
            GoldilocksField([17u64]),
        ];
        let int4_a: [GoldilocksField; 8] = [
            GoldilocksField([0u64]),
            GoldilocksField([1u64]),
            GoldilocksField([2u64]),
            GoldilocksField([3u64]),
            GoldilocksField([10u64]),
            GoldilocksField([11u64]),
            GoldilocksField([12u64]),
            GoldilocksField([13u64]),
        ];
        let int4_b: [GoldilocksField; 8] = [
            GoldilocksField([4u64]),
            GoldilocksField([5u64]),
            GoldilocksField([6u64]),
            GoldilocksField([7u64]),
            GoldilocksField([14u64]),
            GoldilocksField([15u64]),
            GoldilocksField([16u64]),
            GoldilocksField([17u64]),
        ];

        let packed_a = *Avx512GoldilocksField::from_slice(&in_a);
        let packed_b = *Avx512GoldilocksField::from_slice(&in_b);
        {
            let (x1, y1) = packed_a.interleave(packed_b, 1);
            assert_eq!(x1.as_slice(), int1_a);
            assert_eq!(y1.as_slice(), int1_b);
        }
        {
            let (x2, y2) = packed_a.interleave(packed_b, 2);
            assert_eq!(x2.as_slice(), int2_a);
            assert_eq!(y2.as_slice(), int2_b);
        }
        {
            let (x4, y4) = packed_a.interleave(packed_b, 4);
            assert_eq!(x4.as_slice(), int4_a);
            assert_eq!(y4.as_slice(), int4_b);
        }
        {
            let (x8, y8) = packed_a.interleave(packed_b, 8);
            assert_eq!(x8.as_slice(), in_a);
            assert_eq!(y8.as_slice(), in_b);
        }
    }
}
