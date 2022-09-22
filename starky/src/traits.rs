/// Defines basic arithmetic in an extension of a StarkField of a given degree.
///
/// This trait defines how to perform multiplication and compute a Frobenius automorphisms of an
/// element in an extension of degree N for a given [StarkField]. It as assumed that an element in
/// degree N extension field can be represented by N field elements in the base field.
///
/// Implementation of this trait implicitly defines the irreducible polynomial over which the
/// extension field is defined.
pub trait ExtensibleField<const N: usize> {
    /// Returns a product of `a` and `b` in the field defined by this extension.
    fn mul(a: [Self; N], b: [Self; N]) -> [Self; N]
    where
        Self: Sized;

    /// Returns a product of `a` and `b` in the field defined by this extension. `b` represents
    /// an element in the base field.
    fn mul_base(a: [Self; N], b: Self) -> [Self; N]
    where
        Self: Sized;

    /// Returns Frobenius automorphisms for `x` in the field defined by this extension.
    fn frobenius(x: [Self; N]) -> [Self; N]
    where
        Self: Sized;

    /// Returns true if this extension is supported for the underlying base field.
    fn is_supported() -> bool {
        true
    }
}
