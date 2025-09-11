use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Walking directory: {0}")]
    WalkDir(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, ExtractError>;
