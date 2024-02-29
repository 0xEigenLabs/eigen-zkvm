#![allow(clippy::needless_range_loop)]
#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![feature(stdarch_x86_avx512)]

pub mod polsarray;
pub mod polutils;
pub mod stark_verifier_circom;
pub mod stark_verifier_circom_bn128;
pub mod stark_verifier_circom_bls12381;
pub mod traits;
pub mod types;

pub mod arch;

pub mod linearhash;
pub mod linearhash_bls12381;
pub mod linearhash_bn128;

pub mod field_bn128;
mod poseidon_bn128;
mod poseidon_bn128_constants;
mod poseidon_bn128_constants_opt;
pub mod poseidon_bn128_opt;
mod poseidon_constants_avx2;
mod poseidon_constants_avx512;
mod poseidon_constants_opt;
pub mod poseidon_opt;

mod field_bls12381;
mod poseidon_bls12381;
mod poseidon_bls12381_constants;
mod poseidon_bls12381_constants_opt;
pub mod poseidon_bls12381_opt;

pub mod merklehash;
pub mod merklehash_bls12381;

pub mod merklehash_bn128;

mod digest;
pub use digest::ElementDigest;

pub mod constant;
mod expressionops;
pub mod f3g;
pub mod f5g;
pub mod fft;
pub mod fft_p;
mod fft_worker;
mod fri;
pub mod helper;
mod interpreter;
pub mod stark_gen;
pub mod stark_setup;
pub mod stark_verify;
pub mod starkinfo;
mod starkinfo_Z;
mod starkinfo_codegen;
mod starkinfo_cp_prover;
mod starkinfo_cp_ver;
mod starkinfo_fri_prover;
mod starkinfo_fri_ver;
mod starkinfo_map;
pub mod transcript;
pub mod transcript_bls12381;
pub mod transcript_bn128;

pub mod pil2circom;
pub mod prove;
pub mod serializer;
pub mod zkin_join;

pub mod dev;

#[macro_use]
extern crate serde_json;

extern crate ff;
extern crate lazy_static;
extern crate log;
