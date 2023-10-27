use crate::bellman_ce::{Field, PrimeField, PrimeFieldRepr, ScalarEngine};
use std::io::{Error, ErrorKind, Read, Result};

pub fn read_field<R: Read, E: ScalarEngine>(mut reader: R) -> Result<E::Fr> {
    let mut repr = E::Fr::zero().into_repr();
    repr.read_le(&mut reader)?;
    let fr = E::Fr::from_repr(repr).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    Ok(fr)
}
