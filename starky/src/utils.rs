use crate::Randomizable;
use core::{convert::TryInto, fmt::Debug};
use rand::prelude::*;

/// Returns a single random value of the specified type.
///
/// # Panics
/// Panics if:
/// * A valid value requires over 32 bytes.
/// * A valid value could not be generated after 1000 tries.
pub fn rand_value<R: Randomizable>() -> R {
    for _ in 0..1000 {
        let bytes = rand::thread_rng().gen::<[u8; 32]>();
        if let Some(value) = R::from_random_bytes(&bytes[..R::VALUE_SIZE]) {
            return value;
        }
    }

    panic!("failed generate a random field element");
}

pub fn log2(VV: u64) -> u64 {
    /*
    ( ( ( V & 0xFFFF0000 ) !== 0 ? ( V &= 0xFFFF0000, 16 ) : 0 ) |
      ( ( V & 0xFF00FF00 ) !== 0 ? ( V &= 0xFF00FF00, 8 ) : 0 ) |
      ( ( V & 0xF0F0F0F0 ) !== 0 ? ( V &= 0xF0F0F0F0, 4 ) : 0 ) |
      ( ( V & 0xCCCCCCCC ) !== 0 ? ( V &= 0xCCCCCCCC, 2 ) : 0 ) |
      ( ( V & 0xAAAAAAAA ) !== 0 ) );
    */
    let mut result = 0u64;
    let mut V = VV;
    if (V & 0xFFFF0000) != 0 {
        V &= 0xFFFF0000;
        result |= 16;
    };
    if (V & 0xFF00FF00) != 0 {
        V &= 0xFF00FF00;
        result |= 8;
    };
    if (V & 0xF0F0F0F0) != 0 {
        V &= 0xF0F0F0F0;
        result |= 4;
    };
    if (V & 0xCCCCCCCC) != 0 {
        V &= 0xCCCCCCCC;
        result |= 2;
    };
    if (V & 0xAAAAAAAA) != 0 {
        result |= 1
    };
    result
}

#[test]
fn test_log2() {
    assert_eq!(log2(2), 1);
    assert_eq!(log2(16), 4);
    assert_eq!(log2(17), 4);
    assert_eq!(log2(19), 4);
}
