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

    #[error("poseidon hash error`{0}`")]
    PoseidonHashError(String),

    #[error("merkle tree error`{0}`")]
    MerkleTreeError(String),

    #[error("degree should be equal, but `{0}` != `{1}`")]
    MustEqualDegreeError(usize, usize),

    #[error("expression error, msg `{0}`")]
    ExpressionError(String),

    #[error("invalid op, msg `{0}`")]
    InvalidOperator(String),

    #[error("verify FRI proof failed")]
    FRIVerifierFailed,

    #[error("Fr::from_expr error")]
    PFDecodeError(#[from] crate::ff::PrimeFieldDecodingError),

    #[error("WasmRuntime error, exit `{0}`")]
    WasmerRuntimeError(#[from] wasmer::RuntimeError),

    #[error("WasmRuntime init error")]
    InstantiationError(#[from] wasmer::InstantiationError),

    #[error("parse bigint error")]
    ParseBigIntError(#[from] num_bigint::ParseBigIntError),

    #[error("Unknown error, `{0}`")]
    Unknown(String),
}

impl From<String> for EigenError {
    fn from(e: String) -> Self {
        EigenError::Unknown(e)
    }
}
