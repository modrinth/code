//! Theseus error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    FSError(String),

    #[error("Serialization error (JSON): {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("Serialization error (Bincode): {0}")]
    EncodeError(#[from] bincode::error::DecodeError),

    #[error("Deserialization error (Bincode): {0}")]
    DecodeError(#[from] bincode::error::EncodeError),

    #[error("Error parsing URL: {0}")]
    URLError(#[from] url::ParseError),

    #[error("Database error: {0}")]
    DBError(#[from] sled::Error),

    #[error("Unable to read {0} from any source")]
    NoValueFor(String),

    #[error("Metadata error: {0}")]
    MetadataError(#[from] daedalus::Error),

    #[error("Minecraft authentication error: {0}")]
    HydraError(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Error launching Minecraft: {0}")]
    LauncherError(String),

    #[error("Error fetching URL: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Websocket error: {0}")]
    WSError(#[from] async_tungstenite::tungstenite::Error),

    #[error("Websocket closed before {0} could be received!")]
    WSClosedError(String),

    #[error("Incorrect Sha1 hash for download: {0} != {1}")]
    HashError(String, String),

    #[error("Paths stored in the database need to be valid UTF-8: {0}")]
    UTFError(std::path::PathBuf),

    #[error("Invalid input: {0}")]
    InputError(String),

    #[error(
        "Tried to access unloaded profile {0}, loading it probably failed"
    )]
    UnloadedProfileError(String),

    #[error("Profile {0} is not managed by Theseus!")]
    UnmanagedProfileError(String),

    #[error("Error: {0}")]
    OtherError(String),
}

pub type Result<T> = core::result::Result<T, Error>;
