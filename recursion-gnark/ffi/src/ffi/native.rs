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

pub fn test(data_dir: &str, proof: &str) {
    let c_data_dir = CString::new(data_dir).expect("CString::new failed");
    let c_proof = CString::new(proof).expect("CString::new failed");
    unsafe { bind::BuildGroth16(c_data_dir.as_ptr() as *mut i8, c_proof.as_ptr() as *mut i8) }
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
