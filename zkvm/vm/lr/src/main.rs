#![no_std]
extern crate alloc;
use alloc::{vec, vec::Vec};
use runtime::get_prover_input;

fn simple_linear_regression(values: &[(f64, f64)]) -> (f64, f64) {
    let (x, y): (Vec<f64>, Vec<f64>) = values.iter().cloned().unzip();

    let x_mean = mean(&x);
    let y_mean = mean(&y);

    let numerator: f64 = values
        .iter()
        .map(|&(x, y)| (x - x_mean) * (y - y_mean))
        .sum();
    let denominator: f64 = x.iter().map(|&x| (x - x_mean) * (x - x_mean)).sum();

    let slope = numerator / denominator;
    let y_intercept = y_mean - slope * x_mean;

    (y_intercept, slope)
}

fn mean(data: &[f64]) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}

#[no_mangle]
pub fn main() {
    /*
    let size = get_prover_input(0) as u32;
   
    let mut line = vec![];
    for i in (1..(size+1)).step_by(2) {
        let (x, y) = (
            get_prover_input(i) as i32,
            get_prover_input(i+1) as i32 
        );
        line.push((f64::from(1), f64::from(y)));
    }
    */
    
    let line = vec![(1.0, 1.0), (2.0, 2.0)];
    let (y_intercept, slope) = simple_linear_regression(&line);
}
