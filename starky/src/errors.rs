use std::error;
use std::fmt;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EigenError>;

#[derive(Error, Debug)]
pub enum EigenError {
    #[error("invalid range proof, `{0}`")]
    InvalidValue(String),

    #[error("invalid range (expected {expected:?}, found {found:?})")]
    OutOfRangeError { expected: String, found: String },

    #[error("open file error")]
    FileError(#[from] std::io::Error),

    #[error("json serialization error")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unknown error, `{0}`")]
    Unknown(String),
}
