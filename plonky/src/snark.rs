use crate::errors::Result;
use franklin_crypto::bellman::pairing::ff::PrimeField;
use rand::Rng;
/// The basic functionality for a SNARK.
pub trait SNARK<F: PrimeField> {
    type Circuit;
    type AssignedCircuit;
    type ProvingKey: Clone;
    type VerificationKey: Clone;
    type PreparedVerificationKey;
    type Proof: Clone;

    /// Takes in a description of a computation (specified in R1CS constraints),
    /// and samples proving and verification keys for that circuit.
    fn circuit_specific_setup<R: Rng>(
        circuit: Self::Circuit,
        rng: &mut R,
    ) -> Result<(Self::ProvingKey, Self::PreparedVerificationKey)>;

    /// Generates a proof of satisfaction of the arithmetic circuit C (specified
    /// as R1CS constraints).
    fn prove<R: Rng>(
        circuit_pk: &Self::ProvingKey,
        circuit: Self::AssignedCircuit,
        rng: &mut R,
    ) -> Result<Self::Proof>;

    /// Checks that `proof` is a valid proof of the satisfaction of circuit
    /// encoded in `circuit_pvk`, with respect to the public input `public_input`,
    /// specified as R1CS constraints.
    fn verify_with_processed_vk(
        circuit_pvk: &Self::PreparedVerificationKey,
        public_input: &[F],
        proof: &Self::Proof,
    ) -> Result<bool>;
}
