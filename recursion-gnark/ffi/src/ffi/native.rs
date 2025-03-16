#![allow(unused)]

//! FFI bindings for the Go code. The functions exported in this module are safe to call from Rust.
//! All C strings and other C memory should be freed in Rust, including C Strings returned by Go.
//! Although we cast to *mut c_char because the Go signatures can't be immutable, the Go functions
//! should not modify the strings.

use cfg_if::cfg_if;
use std::{
    ffi::{c_char, CStr, CString},
    mem::forget,
};

#[allow(warnings, clippy::all)]
mod bind {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bind::*;

enum ProofSystem {
    Plonk,
    Groth16,
}

/// Build Gnark proof over BLS12381 from BN254
/// Description
///   TODO: proof and public input are encoded with different serialization.
/// * `vk_path` - vk over BN254
/// * `output_dir` - directory including vk, proof, and public input over BLS12381
/// * `proof` - groth16 proof over BN254
/// * `public_input_json` - public input over BN254 in json
pub fn build_groth16(vk_path: &str, output_dir: &str, proof: &str, public_input_json: &str) {
    let c_output_dir = CString::new(output_dir).expect("CString::new output failed");
    let c_vk_path = CString::new(vk_path).expect("CString::new vk failed");
    let c_proof = CString::new(proof).expect("CString::new proof failed");
    let c_input = CString::new(public_input_json).expect("CString::new public input failed");
    unsafe {
        bind::BuildGroth16(
            c_vk_path.as_ptr() as *mut i8,
            c_output_dir.as_ptr() as *mut i8,
            c_proof.as_ptr() as *mut i8,
            c_input.as_ptr() as *mut i8,
        )
    }
}

/// Converts a C string into a Rust String.
///
/// # Safety
/// This function does not free the pointer, so the caller must ensure that the pointer is handled
/// correctly.
unsafe fn ptr_to_string_cloned(input: *mut c_char) -> String {
    CStr::from_ptr(input).to_owned().into_string().expect("CStr::into_string failed")
}

/// Converts a C string into a Rust String.
///
/// # Safety
/// This function frees the pointer, so the caller must ensure that the pointer is not used
/// after this function is called.
unsafe fn ptr_to_string_freed(input: *mut c_char) -> String {
    let string = ptr_to_string_cloned(input);
    bind::FreeString(input);
    string
}
