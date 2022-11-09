#![allow(non_snake_case)]
use crate::merklehash_bn128::MerkleTree;
use crate::polsarray::PolsArray;
use crate::types::{StarkStruct, PIL};
use crate::ElementDigest;

use winter_math::fft::{self, get_inv_twiddles};
use winter_math::{
    fields::f64::BaseElement, get_power_series, log2, polynom, FieldElement, StarkField,
};

use winter_utils::{iter, transpose_slice};

pub fn interpolate_columns(columns: &Vec<Vec<BaseElement>>) -> Vec<Vec<BaseElement>> {
    let num_rows = columns[0].len();
    let inv_twiddles = fft::get_inv_twiddles::<BaseElement>(num_rows);
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
    let num_rows = columns[0].len();
    let twiddles = fft::get_twiddles::<BaseElement>(num_rows);
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

pub struct StarkSetup {
    const_tree: MerkleTree,
    const_root: ElementDigest,
    stark_info: usize,
}

/// STARK SETUP
///
///  calculate the trace polynomial over extended field, return the new polynomial's coefficient.
pub fn stark_setup(const_pol: &PolsArray, pil: &PIL, stark_struct: &StarkStruct) {
    let nBits = stark_struct.nBits;
    let nBitsExt = stark_struct.nBitsExt;

    let mut p: Vec<Vec<BaseElement>> = vec![Vec::new(); const_pol.nPols];
    for i in 0..const_pol.nPols {
        for j in 0..const_pol.n {
            p[i].push(const_pol.array[i][j])
        }
    }

    let m = interpolate_in_pil(&p, 1 << (nBitsExt - nBits));
    /*
    println!("length {}", m[0].len() * m.len());
    for i in &m[0] {
        println!("{:?}\n", i.as_int());
    }
    */

    //const constTree = await MH.merkelize(constPolsArrayE, pil.nConstants, nExt);
    let const_tree = MerkleTree::merkelize(p).unwrap();
}

#[cfg(test)]
pub mod tests {
    use crate::polsarray::{PolKind, PolsArray};
    use crate::stark_setup::stark_setup;
    use crate::types::{load_json, StarkStruct, PIL};
    use winter_math::fft::{self, get_inv_twiddles};
    use winter_math::{
        fields::f64::BaseElement, get_power_series, log2, polynom, FieldElement, StarkField,
    };

    use super::interpolate_in_pil;

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
        let pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant, 32);
        const_pol.load("data/fib.const").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        stark_setup(&const_pol, &pil, &stark_struct);
    }
}
