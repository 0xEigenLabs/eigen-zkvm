use thiserror::Error;

pub use anyhow::{bail, Result};

#[derive(Error, Debug)]
pub enum DslError {
    #[error("circom compiler error, '{0}'")]
    CircomCompileError(String),

    #[error("Unknown error, `{0}`")]
    Unknown(String),
}

impl From<String> for DslError {
    fn from(e: String) -> Self {
        DslError::Unknown(e)
    }
}
