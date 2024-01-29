use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str;

use crate::bellman_ce::{
    pairing::Engine,
    plonk::{
        better_cs::cs::PlonkCsWidth4WithNextStepParams,
        better_cs::keys::{Proof, VerificationKey},
    },
};

#[cfg(not(feature = "wasm"))]
use crate::aggregation::{AggregatedProof, AggregationVerificationKey};

/// load proof by filename
pub fn load_proof<E: Engine>(filename: &str) -> Proof<E, PlonkCsWidth4WithNextStepParams> {
    Proof::<E, PlonkCsWidth4WithNextStepParams>::read(
        File::open(filename).unwrap_or_else(|_| panic!("read proof file err, {}", filename)),
    )
    .expect("read proof err")
}

/// load multiple proofs form a list
pub fn load_proofs_from_list<E: Engine>(
    list: &str,
) -> Vec<Proof<E, PlonkCsWidth4WithNextStepParams>> {
    let file = File::open(list).unwrap_or_else(|_| panic!("read proof list file err, {}", list));
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("could not parse line"))
        .collect();
    let proofs: Vec<Proof<E, PlonkCsWidth4WithNextStepParams>> = lines
        .iter()
        .map(|l| {
            log::trace!("reading {:?}", l);
            load_proof::<E>(l)
        })
        .collect();

    assert!(!proofs.is_empty(), "no proof file found!");

    let num_inputs = proofs[0].num_inputs;
    for p in &proofs {
        assert_eq!(p.num_inputs, num_inputs, "proofs num_inputs mismatch!");
    }

    proofs
}

/// load verification key file by filename
pub fn load_verification_key<E: Engine>(
    filename: &str,
) -> VerificationKey<E, PlonkCsWidth4WithNextStepParams> {
    let mut reader = BufReader::with_capacity(
        1 << 24,
        File::open(filename).unwrap_or_else(|_| panic!("read vk file err, {}", filename)),
    );
    VerificationKey::<E, PlonkCsWidth4WithNextStepParams>::read(&mut reader).expect("read vk err")
}

/// load aggregation proof file by filename
#[cfg(not(feature = "wasm"))]
pub fn load_aggregated_proof(filename: &str) -> AggregatedProof {
    AggregatedProof::read(File::open(filename).expect("read aggregated proof file err"))
        .expect("read aggregated proof err")
}

/// load aggregation verification key file by filename
#[cfg(not(feature = "wasm"))]
pub fn load_aggregation_verification_key(filename: &str) -> AggregationVerificationKey<'static> {
    let mut reader = BufReader::with_capacity(
        1 << 24,
        File::open(filename)
            .unwrap_or_else(|_| panic!("read aggregation vk file err, {}", filename)),
    );
    AggregationVerificationKey::read(&mut reader).expect("read aggregation vk err")
}
