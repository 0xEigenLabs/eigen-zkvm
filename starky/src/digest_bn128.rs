use crate::poseidon_bn128::Fr;
use core::slice;
use ff::Field;
use winter_crypto::Digest;
use winter_math::StarkField;
use winter_math::{fields::f64::BaseElement, FieldElement};
use winter_utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

const DIGEST_SIZE: usize = 4;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ElementDigest([BaseElement; DIGEST_SIZE]);

impl ElementDigest {
    pub fn new(value: [BaseElement; DIGEST_SIZE]) -> Self {
        Self(value)
    }

    pub fn as_elements(&self) -> &[BaseElement] {
        &self.0
    }

    pub fn digests_as_elements(digests: &[Self]) -> &[BaseElement] {
        let p = digests.as_ptr();
        let len = digests.len() * DIGEST_SIZE;
        unsafe { slice::from_raw_parts(p as *const BaseElement, len) }
    }
}

/// Fr always consists of [u64; limbs], here for bn128, the limbs is 4.
impl From<&Fr> for ElementDigest {
    fn from(e: &Fr) -> Self {
        let mut result = [BaseElement::ZERO; DIGEST_SIZE];
        result[0] = BaseElement::from(e.0 .0[0]);
        result[1] = BaseElement::from(e.0 .0[1]);
        result[2] = BaseElement::from(e.0 .0[2]);
        result[3] = BaseElement::from(e.0 .0[3]);
        ElementDigest::new(result)
    }
}

impl Into<Fr> for ElementDigest {
    fn into(self) -> Fr {
        let mut result = Fr::zero();
        result.0 .0[0] = self.0[0].as_int().into();
        result.0 .0[1] = self.0[1].as_int().into();
        result.0 .0[2] = self.0[2].as_int().into();
        result.0 .0[3] = self.0[3].as_int().into();
        result
    }
}

impl Digest for ElementDigest {
    fn as_bytes(&self) -> [u8; 32] {
        let mut result = [0; 32];
        result[..8].copy_from_slice(&self.0[0].as_int().to_le_bytes());
        result[8..16].copy_from_slice(&self.0[1].as_int().to_le_bytes());
        result[16..24].copy_from_slice(&self.0[2].as_int().to_le_bytes());
        result[24..].copy_from_slice(&self.0[3].as_int().to_le_bytes());

        result
    }
}

impl Default for ElementDigest {
    fn default() -> Self {
        ElementDigest([BaseElement::default(); DIGEST_SIZE])
    }
}

impl Serializable for ElementDigest {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&self.as_bytes());
    }
}

impl Deserializable for ElementDigest {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let e1 = BaseElement::new(source.read_u64()?);
        let e2 = BaseElement::new(source.read_u64()?);
        let e3 = BaseElement::new(source.read_u64()?);
        let e4 = BaseElement::new(source.read_u64()?);
        // TODO: check if the field elements are valid?

        Ok(Self([e1, e2, e3, e4]))
    }
}

impl From<[BaseElement; DIGEST_SIZE]> for ElementDigest {
    fn from(value: [BaseElement; DIGEST_SIZE]) -> Self {
        Self(value)
    }
}

impl From<ElementDigest> for [BaseElement; DIGEST_SIZE] {
    fn from(value: ElementDigest) -> Self {
        value.0
    }
}

impl From<ElementDigest> for [u8; 32] {
    fn from(value: ElementDigest) -> Self {
        value.as_bytes()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::digest_bn128::ElementDigest;
    use crate::poseidon_bn128::Fr;
    use ff::PrimeField;
    use rand_utils::rand_vector;
    use winter_math::fields::f64::BaseElement;

    #[test]
    fn test_fr_to_element_digest_and_versus() {
        let b4 = rand_vector::<BaseElement>(4);
        let b4 = ElementDigest::new(b4.try_into().unwrap());
        let f1: Fr = b4.into();

        let b4_: ElementDigest = ElementDigest::from(&f1);
        assert_eq!(b4, b4_);

        let f: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495616", // Fr::MODULE - 1
        )
        .unwrap();

        let e = ElementDigest::from(&f);
        let f2: Fr = e.into();
        assert_eq!(f, f2);
    }
}
