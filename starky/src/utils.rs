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
