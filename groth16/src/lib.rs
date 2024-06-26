#[macro_use]
extern crate hex_literal;
pub mod api;
pub mod groth16;
pub mod json_utils;
mod template;

// #[cfg(any(feature = "cuda", feature = "opencl"))]
// mod gpu_specific {
    pub mod witness;
    pub mod circuit;
    pub mod r1cs_file;
    pub mod reader;
    
    pub use bellperson;
// }

// #[cfg(any(feature = "cuda", feature = "opencl"))]
// pub use gpu_specific::*;

// #[cfg(not(any(feature = "cuda", feature = "opencl")))]
// mod non_gpu_specific {
//     pub use bellman_ce::pairing::ff;
//     pub use ff::*;
//     pub use franklin_crypto::bellman as bellman_ce;
// }

// #[cfg(not(any(feature = "cuda", feature = "opencl")))]
// pub use non_gpu_specific::*;