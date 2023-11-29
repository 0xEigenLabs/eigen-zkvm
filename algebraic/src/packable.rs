use crate::ff::PrimeFieldRepr;
use crate::packed::PackedField;

/// Points us to the default packing for a particular field. There may me multiple choices of
/// PackedField for a particular Field (e.g. every Field is also a PackedField), but this is the
/// recommended one. The recommended packing varies by target_arch and target_feature.
pub trait Packable: PrimeFieldRepr {
    type Packing: PackedField<Scalar = Self>;
}

impl<F> Packable for F
where
    F: PrimeFieldRepr + PackedField<Scalar = F>,
{
    type Packing = Self;
}

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))
))]
impl Packable for crate::field_gl::FrRepr {
    type Packing = crate::arch::x86_64::avx2_field_gl::Avx2GoldilocksField;
}

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx512bw",
    target_feature = "avx512cd",
    target_feature = "avx512dq",
    target_feature = "avx512f",
    target_feature = "avx512vl"
))]
impl Packable for crate::field_gl::FrRepr {
    type Packing = crate::arch::x86_64::avx512_field_gl::Avx512GoldilocksField;
}
