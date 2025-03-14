// copy from https://github.com/TaceoLabs/co-snarks/tree/main/co-circom/circom-types/src/groth16 partially
use crate::traits::{ArkworksPairingBridge, ArkworksPrimeFieldBridge, CheckElement};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use serde::{
    de::{self},
    ser::SerializeSeq,
    Deserialize, Serialize, Serializer,
};
use std::{io::Read, marker::PhantomData, str::FromStr};
/// Represents a public input for a Groth16 proof. Implements [`serde::Deserialize`] and [`serde::Serialize`] for loading/storing public inputs from/to JSON formats defined by arkworks.
#[derive(Debug, PartialEq, Eq)]
pub struct JsonPublicInput<F: PrimeField + FromStr> {
    /// The values of the public input.
    pub values: Vec<F>,
}

struct FrSeqVisitor<F: PrimeField + FromStr> {
    phantom_data: PhantomData<F>,
}

impl<F: PrimeField + FromStr> FrSeqVisitor<F> {
    fn new() -> Self {
        Self { phantom_data: PhantomData }
    }
}

impl<'de, F: PrimeField + FromStr> de::Visitor<'de> for FrSeqVisitor<F> {
    type Value = Vec<F>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of elements on a PrimeField as string with radix 10")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = vec![];
        while let Some(s) = seq.next_element::<String>()? {
            values.push(F::from_str(&s).map_err(|_| de::Error::custom("invalid field element"))?);
        }
        Ok(values)
    }
}

impl<'de, F: PrimeField + FromStr> de::Deserialize<'de> for JsonPublicInput<F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Self { values: deserializer.deserialize_seq(FrSeqVisitor::<F>::new())? })
    }
}

impl<F: PrimeField + FromStr> Serialize for JsonPublicInput<F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.values.len()))?;
        for fr in self.values.iter() {
            seq.serialize_element(&fr.to_string())?;
        }
        seq.end()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonVerificationKey<P: Pairing + ArkworksPairingBridge>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    /// The protocol used to generate the proof (always `"groth16"`)
    pub protocol: String,
    /// The number of public inputs
    #[serde(rename = "nPublic")]
    pub n_public: usize,
    /// The element α of the verification key ∈ G1
    #[serde(rename = "vk_alpha_1")]
    #[serde(serialize_with = "P::serialize_g1::<_>")]
    #[serde(deserialize_with = "P::deserialize_g1_element::<_>")]
    pub alpha_1: P::G1Affine,
    /// The element β of the verification key ∈ G2
    #[serde(rename = "vk_beta_2")]
    #[serde(serialize_with = "P::serialize_g2::<_>")]
    #[serde(deserialize_with = "P::deserialize_g2_element::<_>")]
    pub beta_2: P::G2Affine,
    /// The γ of the verification key ∈ G2
    #[serde(rename = "vk_gamma_2")]
    #[serde(serialize_with = "P::serialize_g2::<_>")]
    #[serde(deserialize_with = "P::deserialize_g2_element::<_>")]
    pub gamma_2: P::G2Affine,
    /// The element δ of the verification key ∈ G2
    #[serde(rename = "vk_delta_2")]
    #[serde(serialize_with = "P::serialize_g2::<_>")]
    #[serde(deserialize_with = "P::deserialize_g2_element::<_>")]
    pub delta_2: P::G2Affine,
    /// The pairing of α and β of the verification key ∈ Gt
    #[serde(rename = "vk_alphabeta_12")]
    #[serde(serialize_with = "P::serialize_gt::<_>")]
    #[serde(deserialize_with = "P::deserialize_gt_element::<_>")]
    pub alpha_beta_gt: P::TargetField,
    /// Used to bind the public inputs to the proof
    #[serde(rename = "IC")]
    #[serde(serialize_with = "serialize_g1_sequence::<_,P>")]
    #[serde(deserialize_with = "deserialize_g1_sequence::<_,P>")]
    pub ic: Vec<P::G1Affine>,
}

impl<P: Pairing + ArkworksPairingBridge> JsonVerificationKey<P>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    /// Deserializes a [`JsonVerificationKey`] from a reader.
    pub fn from_reader<R: Read>(rdr: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(rdr)
    }
}

fn serialize_g1_sequence<S: Serializer, P: Pairing + ArkworksPairingBridge>(
    p: &[P::G1Affine],
    ser: S,
) -> Result<S::Ok, S::Error>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    let mut seq = ser.serialize_seq(Some(p.len())).unwrap();
    let maybe_error = p
        .iter()
        .map(|p| P::g1_to_strings_projective(p))
        .map(|strings| seq.serialize_element(&strings))
        .find(|r| r.is_err());
    if let Some(Err(err)) = maybe_error {
        Err(err)
    } else {
        seq.end()
    }
}
fn deserialize_g1_sequence<'de, D, P: Pairing + ArkworksPairingBridge>(
    deserializer: D,
) -> Result<Vec<P::G1Affine>, D::Error>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_seq(G1SeqVisitor::<P>::new(CheckElement::Yes))
}
struct G1SeqVisitor<P: Pairing + ArkworksPairingBridge>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    check: CheckElement,
    phantom_data: PhantomData<P>,
}

impl<P: Pairing + ArkworksPairingBridge> G1SeqVisitor<P>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    fn new(check: CheckElement) -> Self {
        Self { check, phantom_data: PhantomData }
    }
}
impl<'de, P: Pairing + ArkworksPairingBridge> de::Visitor<'de> for G1SeqVisitor<P>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    type Value = Vec<P::G1Affine>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a sequence of elements representing 
        projective points on G1, which in turn are seqeunces of three
         elements on the BaseField of the Curve.",
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = vec![];
        while let Some(point) = seq.next_element::<Vec<String>>()? {
            //check if there are no more elements
            if point.len() != 3 {
                return Err(de::Error::invalid_length(point.len(), &self));
            } else {
                values.push(
                    P::g1_from_strings_projective(&point[0], &point[1], &point[2], self.check)
                        .map_err(|_| {
                            de::Error::custom("Invalid projective point on G1.".to_owned())
                        })?,
                );
            }
        }
        Ok(values)
    }
}

/// Represents a Groth16 proof in JSON format that was created by Gnark. Supports de/serialization using [`serde`].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Groth16Proof<P: Pairing + ArkworksPairingBridge>
where
    P::BaseField: ArkworksPrimeFieldBridge,
    P::ScalarField: ArkworksPrimeFieldBridge,
{
    /// Proof element A (or 1) in G1
    #[serde(serialize_with = "P::serialize_g1::<_>")]
    #[serde(deserialize_with = "P::deserialize_g1_element::<_>")]
    pub pi_a: P::G1Affine,
    /// Proof element B (or 2) in G2
    #[serde(serialize_with = "P::serialize_g2::<_>")]
    #[serde(deserialize_with = "P::deserialize_g2_element::<_>")]
    pub pi_b: P::G2Affine,
    /// Proof element C (or 3) in G1
    #[serde(serialize_with = "P::serialize_g1::<_>")]
    #[serde(deserialize_with = "P::deserialize_g1_element::<_>")]
    pub pi_c: P::G1Affine,
    /// The protocol used to generate the proof (always `"groth16"`)
    pub protocol: String,
    /// The curve used to generate the proof
    pub curve: String,
}
