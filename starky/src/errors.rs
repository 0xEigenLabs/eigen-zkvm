use std::error;
use std::fmt;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EigenError>;

#[derive(Error, Debug)]
pub enum EigenError {
    #[error("Invalid range proof, `{0}`")]
    InvalidValue(String),

    #[error("invalid range (expected {expected:?}, found {found:?})")]
    OutOfRangeError { expected: String, found: String },
    #[error("Unknown error, `{0}`")]
    Unknown(String),
}
