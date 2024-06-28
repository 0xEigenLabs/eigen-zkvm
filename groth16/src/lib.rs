pub mod api;
pub mod groth16;
pub mod json_utils;
mod template;

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
mod non_gpu_specific {
    pub use bellman_ce::pairing::ff;
    pub use ff::*;
    pub use franklin_crypto::bellman as bellman_ce;
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub use non_gpu_specific::*;
