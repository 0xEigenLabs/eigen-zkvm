#[macro_use]
extern crate hex_literal;
pub mod api;
pub mod groth16;
pub mod json_utils;
mod template;

// #[cfg(not(any(feature = "cuda", feature = "opencl")))]
// pub use bellman_ce::pairing::ff;
// #[cfg(not(any(feature = "cuda", feature = "opencl")))]
// pub use ff::*;
// #[cfg(not(any(feature = "cuda", feature = "opencl")))]
// pub use franklin_crypto::bellman as bellman_ce;

// #[cfg(any(feature = "cuda", feature = "opencl"))]
pub mod witness;
// #[cfg(any(feature = "cuda", feature = "opencl"))]
pub mod circuit;
// #[cfg(any(feature = "cuda", feature = "opencl"))]
pub mod r1cs_file;
// #[cfg(any(feature = "cuda", feature = "opencl"))]
pub mod reader;

// #[cfg(any(feature = "cuda", feature = "opencl"))]
pub use bellperson;
