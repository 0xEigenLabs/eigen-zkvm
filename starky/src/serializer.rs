// input json of plonk
#![allow(non_snake_case)]

use crate::f3g::F3G;
use crate::f5g::F5G;
use crate::stark_gen::StarkProof;
use crate::traits::FieldExtension;
use crate::traits::{MTNodeType, MerkleTree};
use ff::PrimeField;
use fields::field_gl::Fr as FGL;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

impl Serialize for F3G {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let elems = self.as_elements();
        if self.dim == 1 {
            serializer.serialize_str(&elems[0].as_int().to_string())
        } else if self.dim == 3 {
            let mut seq = serializer.serialize_seq(Some(elems.len()))?;
            for v in elems.iter() {
                seq.serialize_element(&v.as_int().to_string())?;
            }
            seq.end()
        } else {
            panic!("Invalid dim {}", self);
        }
    }
}

impl<'de> Deserialize<'de> for F3G {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = F3G;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct F3G")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element::<String>()? {
                    let entry: u64 = entry.parse().unwrap();
                    entries.push(FGL::from(entry));
                }
                Ok(F3G::from_vec(entries))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let ien: u64 = s.parse().unwrap();
                Ok(F3G::from(ien))
            }
        }
        deserializer.deserialize_any(EntriesVisitor)
    }
}

impl Serialize for F5G {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let elems = self.as_elements();
        if self.dim == 1 {
            serializer.serialize_str(&elems[0].as_int().to_string())
        } else if self.dim == 5 {
            let mut seq = serializer.serialize_seq(Some(elems.len()))?;
            for v in elems.iter() {
                seq.serialize_element(&v.as_int().to_string())?;
            }
            seq.end()
        } else {
            panic!("Invalid dim {}", self);
        }
    }
}

impl<'de> Deserialize<'de> for F5G {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = F5G;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct F5G")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element::<String>()? {
                    let entry: u64 = entry.parse().unwrap();
                    entries.push(FGL::from(entry));
                }
                Ok(F5G::from_vec(entries))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let ien: u64 = s.parse().unwrap();
                Ok(F5G::from(ien))
            }
        }
        deserializer.deserialize_any(EntriesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::digest::ElementDigest;
    use crate::f3g::F3G;
    use crate::f5g::F5G;
    use crate::field_bls12381::Fr as Fr_BLS12381;
    use crate::field_bn128::Fr;
    use crate::merklehash::MerkleTreeGL;
    use crate::merklehash_bls12381::MerkleTreeBLS12381;
    use crate::merklehash_bn128::MerkleTreeBN128;
    use crate::polsarray::PolKind;
    use crate::polsarray::PolsArray;
    use crate::serializer::StarkProof;
    use crate::stark_setup::StarkSetup;
    use crate::traits::FieldExtension;
    use crate::traits::MTNodeType;
    use crate::transcript::TranscriptGL;
    use crate::transcript_bls12381::TranscriptBLS128;
    use crate::transcript_bn128::TranscriptBN128;
    use crate::types::load_json;
    use crate::types::StarkStruct;
    use crate::types::PIL;
    use fields::field_gl::Fr as FGL;
    use rand::Rand;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_serialize_f3g() {
        let input = F3G::from(123);
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);

        let mut rng = rand::thread_rng();
        let input = F3G::from_vec(
            [
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
            ]
            .to_vec(),
        );
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);
    }

    #[test]
    fn test_serialize_f5g() {
        let input = F5G::from(123);
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);

        let mut rng = rand::thread_rng();
        let input = F5G::from_vec(
            [
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
            ]
            .to_vec(),
        );
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);
    }

    #[test]
    fn test_serialize_node_wrapper() {
        env_logger::try_init().unwrap_or_default();
        let mut rng = rand::thread_rng();
        let four_fgl = ElementDigest::<4, FGL>::new(&[
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
        ]);

        let four_fgl_ser = serde_json::to_string(&four_fgl).unwrap();
        let actual_four_fgl: ElementDigest<4, FGL> = serde_json::from_str(&four_fgl_ser).unwrap();
        assert_eq!(four_fgl.0, actual_four_fgl.0);

        let one_fgl: ElementDigest<4, FGL> = ElementDigest::from_scalar(&FGL::rand(&mut rng));
        let one_fgl_ser = serde_json::to_string(&one_fgl).unwrap();
        let actual_one_fgl: ElementDigest<4, FGL> = serde_json::from_str(&one_fgl_ser).unwrap();
        assert_eq!(one_fgl.0, actual_one_fgl.0);

        let one_fr: ElementDigest<4, Fr> = ElementDigest::from_scalar(&Fr::rand(&mut rng));
        let one_fr_ser = serde_json::to_string(&one_fr).unwrap();
        let actual_one_fr: ElementDigest<4, Fr> = serde_json::from_str(&one_fr_ser).unwrap();
        assert_eq!(one_fr.0, actual_one_fr.0);

        let one_fr: ElementDigest<4, Fr_BLS12381> =
            ElementDigest::from_scalar(&Fr_BLS12381::rand(&mut rng));
        let one_fr_ser = serde_json::to_string(&one_fr).unwrap();
        let actual_one_fr: ElementDigest<4, Fr_BLS12381> =
            serde_json::from_str(&one_fr_ser).unwrap();
        assert_eq!(one_fr.0, actual_one_fr.0);
    }

    #[test]
    fn test_serialize_stark_proof_bn128_ser_der() {
        env_logger::try_init().unwrap_or_default();
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();

        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        //let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();

        // serde to json
        let serialized = serde_json::to_string(&starkproof).unwrap();
        let mut file = File::create("/tmp/test_stark_proof_serialize.json").unwrap();
        write!(file, "{}", serialized).unwrap();
        // deserialized
        let actual: StarkProof<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();

        let mut file = File::create("/tmp/test_stark_proof_serialize.actual.json").unwrap();
        let serialized2 = serde_json::to_string(&actual).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(serialized, serialized2);
        assert_eq!(actual.root1, starkproof.root1);
        assert_eq!(actual.root2, starkproof.root2);
        assert_eq!(actual.root3, starkproof.root3);
        assert_eq!(actual.root4, starkproof.root4);
        assert_eq!(actual.rootC, starkproof.rootC);
        assert_eq!(actual.publics, starkproof.publics);
        assert_eq!(actual.evals, starkproof.evals);
        assert_eq!(actual.fri_proof, starkproof.fri_proof);
        assert_eq!(actual, starkproof);
    }

    #[test]
    fn test_serialize_stark_proof_gl_ser_der() {
        let mut pil = load_json::<PIL>("data/fib.pil.json.gl").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const.gl").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm.gl").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.gl").unwrap();

        let setup =
            StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        //let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

        let starkproof = StarkProof::<MerkleTreeGL>::stark_gen::<TranscriptGL>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "",
        )
        .unwrap();

        // serde to json
        let serialized = serde_json::to_string(&starkproof).unwrap();
        let mut file = File::create("/tmp/test_stark_proof_serialize.gl.json").unwrap();
        write!(file, "{}", serialized).unwrap();
        // deserialized
        let actual: StarkProof<MerkleTreeGL> = serde_json::from_str(&serialized).unwrap();

        let mut file = File::create("/tmp/test_stark_proof_serialize.actual.gl.json").unwrap();
        let serialized2 = serde_json::to_string(&actual).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(serialized, serialized2);
        assert_eq!(actual.root1, starkproof.root1);
        assert_eq!(actual.root2, starkproof.root2);
        assert_eq!(actual.root3, starkproof.root3);
        assert_eq!(actual.root4, starkproof.root4);
        assert_eq!(actual.rootC, starkproof.rootC);
        assert_eq!(actual.publics, starkproof.publics);
        assert_eq!(actual.evals, starkproof.evals);
        assert_eq!(actual.fri_proof, starkproof.fri_proof);
        assert_eq!(actual, starkproof);
    }

    #[test]
    fn test_serialize_stark_proof_bls12381_ser_der() {
        env_logger::try_init().unwrap_or_default();
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.bls12381").unwrap();

        let setup =
            StarkSetup::<MerkleTreeBLS12381>::new(&const_pol, &mut pil, &stark_struct, None)
                .unwrap();
        //let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

        let starkproof = StarkProof::<MerkleTreeBLS12381>::stark_gen::<TranscriptBLS128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();

        // serde to json
        let serialized = serde_json::to_string(&starkproof).unwrap();
        let mut file = File::create("/tmp/test_stark_proof_serialize.bls12381.json").unwrap();
        write!(file, "{}", serialized).unwrap();
        // deserialized
        let actual: StarkProof<MerkleTreeBLS12381> = serde_json::from_str(&serialized).unwrap();

        let mut file =
            File::create("/tmp/test_stark_proof_serialize.bls12381.actual.json").unwrap();
        let serialized2 = serde_json::to_string(&actual).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(serialized, serialized2);
        assert_eq!(actual.root1, starkproof.root1);
        assert_eq!(actual.root2, starkproof.root2);
        assert_eq!(actual.root3, starkproof.root3);
        assert_eq!(actual.root4, starkproof.root4);
        assert_eq!(actual.rootC, starkproof.rootC);
        assert_eq!(actual.publics, starkproof.publics);
        assert_eq!(actual.evals, starkproof.evals);
        assert_eq!(actual.fri_proof, starkproof.fri_proof);
        assert_eq!(actual, starkproof);
    }
}
