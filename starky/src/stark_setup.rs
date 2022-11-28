#![allow(non_snake_case)]
use crate::errors::Result;
use crate::merklehash_bn128::MerkleTree;
use crate::polsarray::PolsArray;
use crate::starkinfo::{self, StarkInfo};
use crate::types::{StarkStruct, PIL};
use crate::ElementDigest;

use winter_math::{fft, fields::f64::BaseElement, polynom, FieldElement, StarkField};

use winter_utils::{iter, transpose_slice};

pub fn interpolate_columns(columns: &Vec<Vec<BaseElement>>) -> Vec<Vec<BaseElement>> {
    let width = columns[0].len();
    let inv_twiddles = fft::get_inv_twiddles::<BaseElement>(width);
    let columns = iter!(columns)
        .map(|evaluations| {
            let mut column = evaluations.clone(); //TODO: can be opt
            fft::interpolate_poly(&mut column, &inv_twiddles);
            column
        })
        .collect();
    columns
}

pub fn evaluate_columns_over(
    columns: &Vec<Vec<BaseElement>>,
    offset: BaseElement,
    blowup_factor: usize,
) -> Vec<Vec<BaseElement>> {
    let width = columns[0].len();
    let twiddles = fft::get_twiddles::<BaseElement>(width);
    let columns = iter!(columns)
        .map(|poly| fft::evaluate_poly_with_offset(poly, &twiddles, offset, blowup_factor))
        .collect();
    columns
}

/// Interpolate on D_lde
/// 1. Interpret each register trace as evaluations of some polynomial f(x)
/// 2. Interpolate f(x) over a trace domain D_trace
/// 3. Evaluate f(x) over a larger evaluation domain D_lde
///
/// NOTE: for multiple columns, the output `columns` with pil-stark `buff` should be like this:
/// columns[i][j] == buff[j * num_cols + i]
pub fn interpolate_in_pil(
    columns: &Vec<Vec<BaseElement>>,
    blowup_factor: usize,
) -> Vec<Vec<BaseElement>> {
    let m = interpolate_columns(columns);
    let shift = BaseElement::from(49u32);
    let m = evaluate_columns_over(&m, shift, blowup_factor);
    m
}

#[derive(Default)]
pub struct StarkSetupResp {
    pub const_tree: MerkleTree,
    pub const_root: ElementDigest,
    pub starkinfo: StarkInfo,
}

/// STARK SETUP
///
///  calculate the trace polynomial over extended field, return the new polynomial's coefficient.
pub fn stark_setup_new(
    const_pol: &PolsArray,
    pil: &mut PIL,
    stark_struct: &StarkStruct,
) -> Result<StarkSetupResp> {
    let nBits = stark_struct.nBits;
    let nBitsExt = stark_struct.nBitsExt;

    let mut p: Vec<Vec<BaseElement>> = vec![Vec::new(); const_pol.nPols];
    for i in 0..const_pol.nPols {
        for j in 0..const_pol.n {
            p[i].push(const_pol.array[i][j])
        }
    }

    #[cfg(test)]
    crate::helper::pretty_print_matrix(&p);

    //extend and merkelize
    let m = interpolate_in_pil(&p, 1 << (nBitsExt - nBits));
    let const_tree = MerkleTree::merkelize(m, const_pol.nPols, const_pol.n << (nBitsExt - nBits))?;

    let starkinfo = starkinfo::StarkInfo::new(pil, stark_struct)?;
    Ok(StarkSetupResp {
        const_root: const_tree.root(),
        const_tree: const_tree,
        starkinfo: starkinfo,
    })
}

#[cfg(test)]
pub mod tests {
    use crate::polsarray::{PolKind, PolsArray};
    use crate::stark_setup::stark_setup_new;
    use crate::types::{load_json, StarkStruct, PIL};
    use winter_math::fft::{self, get_inv_twiddles};
    use winter_math::{
        fields::f64::BaseElement, get_power_series, log2, polynom, FieldElement, StarkField,
    };

    use super::interpolate_in_pil;
    use crate::poseidon_bn128::Fr;
    use ff::*;

    #[test]
    fn test_interpolate() {
        let nPols = 2;
        let nBits = 4;
        let n = 1 << nBits;
        let blowup_factor = 1 << 1;

        let mut columns: Vec<Vec<BaseElement>> = vec![Vec::new(); nPols];
        for i in 0..nPols {
            columns[i] = vec![BaseElement::ZERO; n];
            for j in 0..n {
                columns[i][j] = BaseElement::from(j as u32);
            }
        }

        let columns = interpolate_in_pil(&columns, blowup_factor);
        println!("size: {}", n);
        for i in 0..nPols {
            let r: Vec<_> = columns[i].iter().map(|e| e.as_int()).collect();
            println!("pol {} {:?}", i, r);
        }
    }

    #[test]
    fn test_stark_setup() {
        let mut pil = load_json::<PIL>("data/fib.pil.json.2").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant, 32);
        const_pol.load("data/fib.const.2").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.2").unwrap();
        let setup = stark_setup_new(&const_pol, &mut pil, &stark_struct).unwrap();
        let root: Fr = setup.const_root.into();
        let expect_root =
            "4658128321472362347225942316135505030498162093259225938328465623672244875764";
        assert_eq!(Fr::from_str(expect_root).unwrap(), root);
        crate::helper::pretty_print_matrix(&setup.const_tree.elements);
    }
}
