use crate::f3g::F3G;
use crate::f5g::F5G;
use crate::field_bls12381::Fr as Fr_bls12381;
use crate::field_bn128::Fr;
use crate::helper;
use crate::stark_gen::StarkProof;
use crate::traits::FieldExtension;
use crate::traits::{MTNodeType, MerkleTree};
use fields::field_gl::Fr as FGL;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};

// A Visitor is a type that holds methods that a Deserializer can drive
// depending on what is contained in the input data.
//
// In the case of a map we need generic type parameters K and V to be
// able to set the output type correctly, but don't require any state.
// This is an example of a "zero sized type" in Rust. The PhantomData
// keeps the compiler from complaining about unused generic type
// parameters.
struct StarkProofVisitor<M: MerkleTree> {
    marker: PhantomData<fn() -> StarkProof<M>>,
}

impl<M: MerkleTree> StarkProofVisitor<M> {
    fn new() -> Self {
        StarkProofVisitor {
            marker: PhantomData,
        }
    }
}

// This is the trait that Deserializers are going to be driving. There
// is one method for each type of data that our type knows how to
// deserialize from. There are many other methods that are not
// implemented here, for example deserializing from integers or strings.
// By default those methods will return an error, which makes sense
// because we cannot deserialize a StarkProof from an integer or string.
impl<'de, M: MerkleTree> Visitor<'de> for StarkProofVisitor<M>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    // The type that our Visitor is going to produce.
    type Value = StarkProof<M>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // Deserialize StarkProof from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    // fn visit_seq<V>(self, mut seq: V) -> Result<Duration, V::Error>
    //     where
    //         V: SeqAccess<'de>,
    // {
    //     let secs = seq.next_element()?
    //         .ok_or_else(|| de::Error::invalid_length(0, &self))?;
    //     let nanos = seq.next_element()?
    //         .ok_or_else(|| de::Error::invalid_length(1, &self))?;
    //     Ok(Duration::new(secs, nanos))
    // }
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = StarkProof::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

// This is the trait that informs Serde how to deserialize StarkProof.
impl<'de, M> Deserialize<'de> for StarkProof<M>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of StarkProof.
        deserializer.deserialize_map(StarkProofVisitor::new())
    }
}
