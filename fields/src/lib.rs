#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512))]
//#![cfg_attr(feature = "avx512", feature(stdsimd))]
extern crate rand;

pub mod arch;
pub mod field_gl;
#[cfg(test)]
mod field_gl_test;
pub mod packable;
pub mod packed;

pub use crate::ff::*;
pub use bellman_ce::pairing::ff;
pub use franklin_crypto::bellman as bellman_ce;
