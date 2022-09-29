#![cfg(not(target_arch = "wasm32"))]
use franklin_crypto::bellman::pairing::{
    bn256::{Bn256 as NodeEngine, Fr},
    CurveAffine, Engine,
};
use franklin_crypto::bellman::plonk::{better_better_cs::setup::VerificationKey, domains::Domain};
use handlebars::to_json;
use handlebars::Handlebars;
use recursive_aggregation_circuit::circuit::RecursiveAggregationCircuitBn256;
use std::collections::HashMap;

pub use crate::aggregation::Config;

pub use recursive_aggregation_circuit::circuit;

use franklin_crypto::bellman::pairing::{
    bn256::Bn256,
    ff::{PrimeField, PrimeFieldRepr, ScalarEngine},
};

pub(crate) fn render_scalar_to_hex<F: PrimeField>(el: &F) -> String {
    let mut buff = vec![];
    let repr = el.into_repr();
    repr.write_be(&mut buff).unwrap();

    format!("0x{}", hex::encode(buff))
}

#[cfg(not(feature = "wasm"))]
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

pub(crate) fn rendered_key(
    recursive_vk: VerificationKey<NodeEngine, RecursiveAggregationCircuitBn256<'static>>,
) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();

    let domain_size = recursive_vk.n.next_power_of_two().to_string();
    map.insert("domain_size".to_owned(), to_json(domain_size));

    let num_inputs = recursive_vk.num_inputs.to_string();
    map.insert("num_inputs".to_owned(), to_json(num_inputs));

    let domain = Domain::<Fr>::new_for_size(recursive_vk.n.next_power_of_two() as u64).unwrap();
    let omega = domain.generator;
    map.insert("omega".to_owned(), to_json(render_scalar_to_hex(&omega)));

    for (i, c) in recursive_vk.gate_setup_commitments.iter().enumerate() {
        let rendered = render_g1_affine_to_hex::<NodeEngine>(c);

        for (j, rendered) in rendered.iter().enumerate() {
            map.insert(
                format!("gate_setup_commitment_{}_{}", i, j),
                to_json(rendered),
            );
        }
    }

    for (i, c) in recursive_vk.gate_selectors_commitments.iter().enumerate() {
        let rendered = render_g1_affine_to_hex::<NodeEngine>(c);

        for (j, rendered) in rendered.iter().enumerate() {
            map.insert(
                format!("gate_selector_commitment_{}_{}", i, j),
                to_json(rendered),
            );
        }
    }

    for (i, c) in recursive_vk.permutation_commitments.iter().enumerate() {
        let rendered = render_g1_affine_to_hex::<NodeEngine>(c);

        for (j, rendered) in rendered.iter().enumerate() {
            map.insert(
                format!("permutation_commitment_{}_{}", i, j),
                to_json(rendered),
            );
        }
    }

    for (i, c) in recursive_vk.non_residues.into_iter().enumerate() {
        let rendered = render_scalar_to_hex(&c);
        map.insert(format!("permutation_non_residue_{}", i), to_json(&rendered));
    }

    let rendered = render_g2_affine_to_hex(&recursive_vk.g2_elements[1]);

    map.insert("g2_x_x_c0".to_owned(), to_json(&rendered[0]));
    map.insert("g2_x_x_c1".to_owned(), to_json(&rendered[1]));
    map.insert("g2_x_y_c0".to_owned(), to_json(&rendered[2]));
    map.insert("g2_x_y_c1".to_owned(), to_json(&rendered[3]));

    // to_json(map)
    map
}

fn render_g1_affine_to_hex<E: Engine>(point: &E::G1Affine) -> [String; 2] {
    if point.is_zero() {
        return ["0x0".to_owned(), "0x0".to_owned()];
    }

    let (x, y) = point.into_xy_unchecked();
    [render_scalar_to_hex(&x), render_scalar_to_hex(&y)]
}

fn render_g2_affine_to_hex(point: &<NodeEngine as Engine>::G2Affine) -> [String; 4] {
    if point.is_zero() {
        return [
            "0x0".to_owned(),
            "0x0".to_owned(),
            "0x0".to_owned(),
            "0x0".to_owned(),
        ];
    }

    let (x, y) = point.into_xy_unchecked();

    [
        render_scalar_to_hex(&x.c0),
        render_scalar_to_hex(&x.c1),
        render_scalar_to_hex(&y.c0),
        render_scalar_to_hex(&y.c1),
    ]
}

pub fn create_verifier_contract_from_template(
    config: Config,
    template: &str,
    render_to_path: &str,
) {
    let mut template_params = HashMap::new();

    template_params.insert(
        "vk_tree_root".to_string(),
        to_json(render_scalar_to_hex(&config.vk_tree_root)),
    );

    template_params.insert(
        "individual_input_num".to_string(),
        to_json(&config.individual_input_num),
    );
    // template_params.insert("vk_max_index".to_string(), to_json(config.vk_max_index));

    // TODO: improve?
    let key_details = rendered_key(config.aggregation_vk);
    for (k, v) in key_details {
        template_params.insert(k, to_json(v));
    }

    let res = Handlebars::new()
        .render_template(template, &template_params)
        .expect("failed to render Verifiers.sol template");
    std::fs::write(render_to_path, res).expect("failed to wrtie Verifier.sol");
    log::info!("Verifier contract successfully generated");
}

pub fn create_verifier_contract(config: Config, template_filepath: &str, render_to_path: &str) {
    let template =
        std::fs::read_to_string(template_filepath).expect("failed to read Verifier template file");
    create_verifier_contract_from_template(config, &template, render_to_path)
}

pub fn create_verifier_contract_from_default_template(config: Config, render_to_path: &str) {
    let template = include_str!("./VerifierTemplate.sol");
    create_verifier_contract_from_template(config, template, render_to_path)
}
