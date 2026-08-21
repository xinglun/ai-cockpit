use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("invalid release input: {0}")]
    Invalid(String),
}
