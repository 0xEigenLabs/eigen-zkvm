#![cfg(not(target_arch = "wasm32"))]
// refers to https://github.com/matter-labs/recursive_aggregation_circuit/blob/master/src/circuit/mod.rs
#![allow(clippy::needless_range_loop)]
use anyhow::Result;
use crate::{bellman_ce, utils};
use bellman_ce::{
    kate_commitment::{Crs, CrsForMonomialForm},
    pairing::bn256,
    pairing::bn256::{Bn256, Fr},
    pairing::ff::{PrimeField, PrimeFieldRepr, ScalarEngine},
    pairing::{CurveAffine, Engine},
    worker::Worker,
    {Field, SynthesisError},
};

use bellman_ce::plonk::better_better_cs::{
    cs::{
        Circuit as NewCircuit, PlonkCsWidth4WithNextStepAndCustomGatesParams, ProvingAssembly,
        Setup, TrivialAssembly, Width4MainGateWithDNext,
    },
    proof::Proof as NewProof,
    setup::VerificationKey,
    verifier::verify as core_verify,
};

use bellman_ce::plonk::{
    better_cs::cs::PlonkCsWidth4WithNextStepParams,
    better_cs::keys::{read_fr_vec, write_fr_vec},
    better_cs::keys::{Proof as OldProof, VerificationKey as OldVerificationKey},
    commitments::transcript::keccak_transcript::RollingKeccakTranscript,
};

use franklin_crypto::plonk::circuit::{
    bigint::field::RnsParameters,
    verifier_circuit::affine_point_wrapper::aux_data::{AuxData, BN256AuxData},
    verifier_circuit::data_structs::IntoLimbedWitness,
    Width4WithCustomGates,
};
use franklin_crypto::rescue::bn256::Bn256RescueParams;

use recursive_aggregation_circuit::circuit::{
    create_recursive_circuit_setup, create_recursive_circuit_vk_and_setup, create_vks_tree,
    make_aggregate, make_public_input_and_limbed_aggregate, RecursiveAggregationCircuitBn256,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ethabi::ethereum_types::U256;
use itertools::Itertools;
use profiler_macro::time_profiler;
use serde::{ser::SerializeSeq, Serialize, Serializer};
use std::io::{Read, Write};

pub mod ethereum_serializer {
    use super::*;
    use ethabi::ethereum_types::U256;

    pub fn serialize_g1(point: &<Bn256 as Engine>::G1Affine) -> (U256, U256) {
        if point.is_zero() {
            return (U256::zero(), U256::zero());
        }
        let uncompressed = point.into_uncompressed();

        let uncompressed_slice = uncompressed.as_ref();

        // bellman serializes points as big endian and in the form x, y
        // ethereum expects the same order in memory
        let x = U256::from_big_endian(&uncompressed_slice[0..32]);
        let y = U256::from_big_endian(&uncompressed_slice[32..64]);

        (x, y)
    }

    pub fn serialize_g2(point: &<Bn256 as Engine>::G2Affine) -> ((U256, U256), (U256, U256)) {
        let uncompressed = point.into_uncompressed();

        let uncompressed_slice = uncompressed.as_ref();

        // bellman serializes points as big endian and in the form x1*u, x0, y1*u, y0
        // ethereum expects the same order in memory
        let x_1 = U256::from_big_endian(&uncompressed_slice[0..32]);
        let x_0 = U256::from_big_endian(&uncompressed_slice[32..64]);
        let y_1 = U256::from_big_endian(&uncompressed_slice[64..96]);
        let y_0 = U256::from_big_endian(&uncompressed_slice[96..128]);

        ((x_1, x_0), (y_1, y_0))
    }

    pub fn serialize_fe(field_element: &<Bn256 as ScalarEngine>::Fr) -> U256 {
        let mut be_bytes = [0u8; 32];
        field_element
            .into_repr()
            .write_be(&mut be_bytes[..])
            .expect("get new root BE bytes");
        U256::from_big_endian(&be_bytes[..])
    }
}

pub struct Config {
    pub aggregation_vk: VerificationKey<Bn256, RecursiveAggregationCircuitBn256<'static>>, // TODO: fix type
    pub vk_tree_root: Fr,
    //    pub vk_max_index: u8,
    pub individual_input_num: usize,
}

// notice the life time in RecursiveAggregationCircuit is related to  series of param groups
// for most cases we could make the params static
type AggregationCircuitProof<'a> = NewProof<Bn256, RecursiveAggregationCircuitBn256<'a>>;

pub type AggregationVerificationKey<'a> =
    VerificationKey<Bn256, RecursiveAggregationCircuitBn256<'a>>;

pub struct AggregatedProof {
    pub proof: AggregationCircuitProof<'static>,
    pub individual_vk_inputs: Vec<bn256::Fr>, // flatten Vec<Vec<bn256::Fr>> into Vec<bn256::Fr>
    pub individual_num_inputs: usize,
    pub individual_vk_idxs: Vec<usize>,
    pub aggr_limbs: Vec<bn256::Fr>,
}

fn read_usize_vec<R: Read>(mut reader: R) -> std::io::Result<Vec<usize>> {
    let num_elements = reader.read_u64::<LittleEndian>()?;
    let mut elements = vec![];
    for _ in 0..num_elements {
        let el = reader.read_u64::<LittleEndian>()?;
        elements.push(el as usize);
    }

    Ok(elements)
}

fn write_usize_vec<W: Write>(p: &[usize], mut writer: W) -> std::io::Result<()> {
    writer.write_u64::<LittleEndian>(p.len() as u64)?;
    for p in p.iter() {
        writer.write_u64::<LittleEndian>(*p as u64)?;
    }
    Ok(())
}

impl AggregatedProof {
    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        self.proof.write(&mut writer)?;
        write_fr_vec(&self.individual_vk_inputs, &mut writer)?;
        write_fr_vec(&self.aggr_limbs, &mut writer)?;
        write_usize_vec(&self.individual_vk_idxs, &mut writer)?;
        writer.write_u64::<LittleEndian>(self.individual_num_inputs as u64)?;
        Ok(())
    }

    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let proof = AggregationCircuitProof::<'static>::read(&mut reader)?;
        let vk_inputs = read_fr_vec::<bn256::Fr, _>(&mut reader)?;
        let aggr_limbs = read_fr_vec::<bn256::Fr, _>(&mut reader)?;
        let vk_idexs = read_usize_vec(&mut reader)?;
        let num_inputs = reader.read_u64::<LittleEndian>()? as usize;

        Ok(Self {
            proof,
            individual_vk_inputs: vk_inputs,
            individual_num_inputs: num_inputs,
            individual_vk_idxs: vk_idexs,
            aggr_limbs,
        })
    }
}

impl Serialize for AggregatedProof {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(5))?;
        let (input, serialized_proof) = serialize_new_proof(&self.proof);
        seq.serialize_element(&input)?;
        seq.serialize_element(&serialized_proof)?;
        let vk_inputs: Vec<U256> = self
            .individual_vk_inputs
            .iter()
            .map(ethereum_serializer::serialize_fe)
            .collect();
        seq.serialize_element(&self.individual_vk_idxs)?;
        seq.serialize_element(&vk_inputs)?;
        let subproofs_limbs: Vec<U256> = self
            .aggr_limbs
            .iter()
            .map(ethereum_serializer::serialize_fe)
            .collect();
        assert_eq!(subproofs_limbs.len(), 16);
        seq.serialize_element(&subproofs_limbs)?;

        seq.end()
    }
}

pub fn serialize_new_proof<C: NewCircuit<bn256::Bn256>>(
    proof: &NewProof<bn256::Bn256, C>,
) -> (Vec<U256>, Vec<U256>) {
    let mut inputs = vec![];
    for input in proof.inputs.iter() {
        inputs.push(ethereum_serializer::serialize_fe(input));
    }
    let mut serialized_proof = vec![];

    for c in proof.state_polys_commitments.iter() {
        let (x, y) = ethereum_serializer::serialize_g1(c);
        serialized_proof.push(x);
        serialized_proof.push(y);
    }

    let (x, y) =
        ethereum_serializer::serialize_g1(&proof.copy_permutation_grand_product_commitment);
    serialized_proof.push(x);
    serialized_proof.push(y);

    for c in proof.quotient_poly_parts_commitments.iter() {
        let (x, y) = ethereum_serializer::serialize_g1(c);
        serialized_proof.push(x);
        serialized_proof.push(y);
    }

    for c in proof.state_polys_openings_at_z.iter() {
        serialized_proof.push(ethereum_serializer::serialize_fe(c));
    }

    for (_, _, c) in proof.state_polys_openings_at_dilations.iter() {
        serialized_proof.push(ethereum_serializer::serialize_fe(c));
    }

    assert_eq!(proof.gate_setup_openings_at_z.len(), 0);

    for (_, c) in proof.gate_selectors_openings_at_z.iter() {
        serialized_proof.push(ethereum_serializer::serialize_fe(c));
    }

    for c in proof.copy_permutation_polys_openings_at_z.iter() {
        serialized_proof.push(ethereum_serializer::serialize_fe(c));
    }

    serialized_proof.push(ethereum_serializer::serialize_fe(
        &proof.copy_permutation_grand_product_opening_at_z_omega,
    ));
    serialized_proof.push(ethereum_serializer::serialize_fe(
        &proof.quotient_poly_opening_at_z,
    ));
    serialized_proof.push(ethereum_serializer::serialize_fe(
        &proof.linearization_poly_opening_at_z,
    ));

    let (x, y) = ethereum_serializer::serialize_g1(&proof.opening_proof_at_z);
    serialized_proof.push(x);
    serialized_proof.push(y);

    let (x, y) = ethereum_serializer::serialize_g1(&proof.opening_proof_at_z_omega);
    serialized_proof.push(x);
    serialized_proof.push(y);

    (inputs, serialized_proof)
}

// only support depth<8. different depths don't really make performance different
const VK_TREE_DEPTH: usize = 7;

// recursively prove multiple proofs, and aggregate them into one
#[time_profiler("agg_plonk_prove")]
pub fn prove(
    big_crs: Crs<Bn256, CrsForMonomialForm>,
    old_proofs: Vec<OldProof<Bn256, PlonkCsWidth4WithNextStepParams>>,
    old_vk: OldVerificationKey<Bn256, PlonkCsWidth4WithNextStepParams>,
) -> Result<AggregatedProof> {
    let num_proofs_to_check = old_proofs.len();
    assert!(num_proofs_to_check > 0);
    assert!(num_proofs_to_check < 256);
    let mut individual_vk_inputs = Vec::new();
    let num_inputs = old_proofs[0].num_inputs;
    for p in &old_proofs {
        for input_value in p.input_values.clone() {
            individual_vk_inputs.push(input_value);
        }
        assert_eq!(p.num_inputs, num_inputs, "proofs num_inputs mismatch!");
    }

    let worker = Worker::new();
    let rns_params = RnsParameters::<Bn256, <Bn256 as Engine>::Fq>::new_for_field(68, 110, 4);
    let rescue_params = Bn256RescueParams::new_checked_2_into_1();

    let mut g2_bases = [<<Bn256 as Engine>::G2Affine as CurveAffine>::zero(); 2];
    g2_bases.copy_from_slice(&big_crs.g2_monomial_bases.as_ref()[..]);
    let aux_data = BN256AuxData::new();

    //notice we have only 1 vk now
    let vks = old_proofs.iter().map(|_| old_vk.clone()).collect_vec();
    let individual_vk_idxs = old_proofs.iter().map(|_| 0usize).collect_vec();
    let (_, (vks_tree, all_witness_values)) = create_vks_tree(&[old_vk], VK_TREE_DEPTH)?;
    let vks_tree_root = vks_tree.get_commitment();

    let proof_ids = individual_vk_idxs.clone();

    let mut queries = vec![];
    for proof_id in 0..num_proofs_to_check {
        let vk = &vks[individual_vk_idxs[proof_id]];

        let leaf_values = vk
            .into_witness_for_params(&rns_params)
            .expect("must transform into limbed witness");

        let values_per_leaf = leaf_values.len();
        let intra_leaf_indexes_to_query: Vec<_> =
            ((proof_id * values_per_leaf)..((proof_id + 1) * values_per_leaf)).collect();
        let q = vks_tree.produce_query(intra_leaf_indexes_to_query, &all_witness_values);

        assert_eq!(q.values(), &leaf_values[..]);

        queries.push(q.path().to_vec());
    }

    let aggregate = make_aggregate(&old_proofs, &vks, &rescue_params, &rns_params)?;

    let (_, limbed_aggreagate) = make_public_input_and_limbed_aggregate(
        vks_tree_root,
        &proof_ids,
        &old_proofs,
        &aggregate,
        &rns_params,
    );

    let circuit = RecursiveAggregationCircuitBn256 {
        num_proofs_to_check,
        num_inputs,
        vk_tree_depth: VK_TREE_DEPTH,
        vk_root: Some(vks_tree_root),
        vk_witnesses: Some(vks),
        vk_auth_paths: Some(queries),
        proof_ids: Some(proof_ids),
        proofs: Some(old_proofs),

        rescue_params: &rescue_params,
        rns_params: &rns_params,
        aux_data,
        transcript_params: &rescue_params,

        g2_elements: Some(g2_bases),

        _m: std::marker::PhantomData,
    };

    // quick_check_if_satisfied
    let mut cs = TrivialAssembly::<Bn256, Width4WithCustomGates, Width4MainGateWithDNext>::new();
    circuit.synthesize(&mut cs).expect("should synthesize");
    log::trace!("Raw number of gates: {}", cs.n());
    cs.finalize();
    log::trace!("Padded number of gates: {}", cs.n());
    assert!(cs.is_satisfied());
    log::trace!("satisfied {}", cs.is_satisfied());
    assert_eq!(cs.num_inputs, 1);

    let setup: Setup<Bn256, RecursiveAggregationCircuitBn256> =
        create_recursive_circuit_setup(num_proofs_to_check, num_inputs, VK_TREE_DEPTH)?;

    let mut assembly = ProvingAssembly::<
        Bn256,
        PlonkCsWidth4WithNextStepAndCustomGatesParams,
        Width4MainGateWithDNext,
    >::new();
    circuit.synthesize(&mut assembly).expect("must synthesize");
    assembly.finalize();

    let proof = assembly.create_proof::<_, RollingKeccakTranscript<<Bn256 as ScalarEngine>::Fr>>(
        &worker, &setup, &big_crs, None,
    )?;

    Ok(AggregatedProof {
        proof,
        individual_vk_inputs,
        individual_num_inputs: num_inputs,
        individual_vk_idxs,
        aggr_limbs: limbed_aggreagate,
    })
}

fn verify_subproof_limbs(
    proof: &AggregatedProof,
    vk: &VerificationKey<Bn256, RecursiveAggregationCircuitBn256>,
) -> Result<bool> {
    let mut rns_params = RnsParameters::<Bn256, <Bn256 as Engine>::Fq>::new_for_field(68, 110, 4);

    //keep the behavior same as recursive_aggregation_circuit
    rns_params.set_prefer_single_limb_allocation(true);

    let aggr_limbs_nums: Vec<utils::BigUint> =
        proof.aggr_limbs.iter().map(utils::fe_to_biguint).collect();
    //we need 4 Fr to build 2 G1Affine ...
    let num_consume = rns_params.num_limbs_for_in_field_representation;
    assert_eq!(num_consume * 4, aggr_limbs_nums.len());

    let mut start = 0;
    let pg_x = utils::witness_to_field(&aggr_limbs_nums[start..start + num_consume], &rns_params);
    start += num_consume;
    let pg_y = utils::witness_to_field(&aggr_limbs_nums[start..start + num_consume], &rns_params);
    start += num_consume;
    let px_x = utils::witness_to_field(&aggr_limbs_nums[start..start + num_consume], &rns_params);
    start += num_consume;
    let px_y = utils::witness_to_field(&aggr_limbs_nums[start..start + num_consume], &rns_params);

    let pair_with_generator =
        bn256::G1Affine::from_xy_checked(pg_x, pg_y).map_err(|_| SynthesisError::Unsatisfiable)?;
    let pair_with_x =
        bn256::G1Affine::from_xy_checked(px_x, px_y).map_err(|_| SynthesisError::Unsatisfiable)?;

    let valid = Bn256::final_exponentiation(&Bn256::miller_loop(&[
        (&pair_with_generator.prepare(), &vk.g2_elements[0].prepare()),
        (&pair_with_x.prepare(), &vk.g2_elements[1].prepare()),
    ]))
    .ok_or(SynthesisError::Unsatisfiable)?
        == <Bn256 as Engine>::Fqk::one();

    Ok(valid)
}

// verify a aggregation proof by using a corresponding verification key
#[time_profiler("agg_plonk_verify")]
pub fn verify(
    vk: VerificationKey<Bn256, RecursiveAggregationCircuitBn256>,
    aggregated_proof: AggregatedProof,
) -> Result<bool> {
    let mut inputs = Vec::new();
    for chunk in aggregated_proof
        .individual_vk_inputs
        .chunks(aggregated_proof.individual_num_inputs)
    {
        inputs.push(chunk);
    }
    log::trace!("individual_inputs: {:#?}", inputs);
    //notice in PlonkCore.sol the aggregate pairs from subproofs and recursive proofs are combined: 1 * inner + challenge * outer
    //and only one verify on pairing has been run to save some gas
    //here we just verify them respectively
    let valid = core_verify::<_, _, RollingKeccakTranscript<<Bn256 as ScalarEngine>::Fr>>(
        &vk,
        &aggregated_proof.proof,
        None,
    )?;
    if !valid {
        return Ok(valid);
    }
    log::trace!("aggregated proof is valid");
    verify_subproof_limbs(&aggregated_proof, &vk)
}

// export a verification key for a recursion circuit
pub fn export_vk(
    num_proofs_to_check: usize,
    num_inputs: usize,
    big_crs: &Crs<Bn256, CrsForMonomialForm>,
) -> Result<VerificationKey<Bn256, RecursiveAggregationCircuitBn256>> {
    let (recursive_circuit_vk, _recursive_circuit_setup) = create_recursive_circuit_vk_and_setup(
        num_proofs_to_check,
        num_inputs,
        VK_TREE_DEPTH,
        big_crs,
    )?;
    Ok(recursive_circuit_vk)
}

// hash the vk_tree root, proof_indexes, proofs' inputs and aggregated points
pub fn get_aggregated_input(
    old_proofs: Vec<OldProof<Bn256, PlonkCsWidth4WithNextStepParams>>,
    old_vk: OldVerificationKey<Bn256, PlonkCsWidth4WithNextStepParams>,
) -> Result<bn256::Fr> {
    let num_proofs_to_check = old_proofs.len();
    assert!(num_proofs_to_check > 0);
    assert!(num_proofs_to_check < 256);
    let num_inputs = old_proofs[0].num_inputs;
    for p in &old_proofs {
        assert_eq!(p.num_inputs, num_inputs, "proofs num_inputs mismatch!");
    }

    let rns_params = RnsParameters::<Bn256, <Bn256 as Engine>::Fq>::new_for_field(68, 110, 4);
    let rescue_params = Bn256RescueParams::new_checked_2_into_1();

    let vks = old_proofs.iter().map(|_| old_vk.clone()).collect_vec();
    let proof_ids = (0..num_proofs_to_check).map(|_| 0usize).collect_vec();

    let (_, (vks_tree, _)) = create_vks_tree(&[old_vk], VK_TREE_DEPTH)?;
    let vks_tree_root = vks_tree.get_commitment();

    let aggregate = make_aggregate(&old_proofs, &vks, &rescue_params, &rns_params)?;

    let (expected_input, _) = make_public_input_and_limbed_aggregate(
        vks_tree_root,
        &proof_ids,
        &old_proofs,
        &aggregate,
        &rns_params,
    );

    Ok(expected_input)
}

pub fn get_vk_tree_root_hash(
    old_vk: OldVerificationKey<Bn256, PlonkCsWidth4WithNextStepParams>,
) -> Result<bn256::Fr> {
    let (_, (vks_tree, _)) = create_vks_tree(&vec![old_vk], VK_TREE_DEPTH)?;
    Ok(vks_tree.get_commitment())
}
