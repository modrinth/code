//! Theseus error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    FSError(String),

    #[error("Serialization error (JSON): {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("Serialization error (Bincode): {0}")]
    BincodeError(#[from] bincode::Error),

    #[error("Database error: {0}")]
    DBError(#[from] sled::Error),

    #[error("Unable to read {0} from any source")]
    NoValueFor(String),

    #[error("Metadata error: {0}")]
    MetadataError(#[from] daedalus::Error),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Error launching Minecraft: {0}")]
    LauncherError(String),

    #[error("Error fetching URL: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Incorrect Sha1 hash for download: {0} != {1}")]
    HashError(String, String),

    #[error("Paths stored in the database need to be valid UTF-8: {0}")]
    UTFError(std::path::PathBuf),

    #[error("Invalid input: {0}")]
    InputError(String),

    #[error("Error: {0}")]
    OtherError(String),
}

pub type Result<T> = core::result::Result<T, Error>;
