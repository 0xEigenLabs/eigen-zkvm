//! Safe-ish interface for reading and writing specific types to the WASM runtime's memory,
//! modified from ark-circom
use num_bigint::BigInt;
use wasmer::Memory;

#[derive(Clone, Debug)]
pub struct SafeMemory {
    pub memory: Memory,
    pub prime: BigInt,
    // short_max: BigInt,
    // short_min: BigInt,
    // r_inv: BigInt,
    // n32: usize,
}

impl SafeMemory {
    /// Creates a new SafeMemory
    pub fn new(memory: Memory, _n32: usize, prime: BigInt) -> Self {
        // TODO: Figure out a better way to calculate these
        // let short_max = BigInt::from(0x8000_0000u64);
        // let short_min = BigInt::from_biguint(
        //     num_bigint::Sign::NoSign,
        //     BigUint::from_str(
        //         "21888242871839275222246405745257275088548364400416034343698204186575808495617",
        //     )
        //     .unwrap(),
        // ) - &short_max;
        // let r_inv = BigInt::from_str(
        //     "9915499612839321149637521777990102151350674507940716049588462388200839649614",
        // )
        // .unwrap();

        Self {
            memory,
            prime,
            // short_max,
            // short_min,
            // r_inv,
            // n32,
        }
    }
}
