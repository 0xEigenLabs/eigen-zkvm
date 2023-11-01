#![allow(clippy::unit_arg)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate hex_literal;
extern crate byteorder;
extern crate franklin_crypto;
extern crate itertools;
extern crate num_bigint;
extern crate num_traits;
extern crate rand;

pub mod arch;

pub mod circom_circuit;
pub mod errors;
pub mod field_gl;
pub mod r1cs_file;
pub mod reader;
pub mod witness;

pub mod utils;

pub use bellman_ce::pairing::ff;
pub use ff::*;
pub use franklin_crypto::bellman as bellman_ce;

#[cfg(test)]
mod field_gl_test;
// mod packed;

#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;

#[cfg(all(test, target_arch = "wasm32"))]
extern crate wasm_bindgen_test;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
