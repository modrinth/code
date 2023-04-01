//! Theseus error type
use crate::profile_create;
use tracing_error::InstrumentError;

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("Filesystem error: {0}")]
    FSError(String),

    #[error("Serialization error (JSON): {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("Error parsing UUID: {0}")]
    UUIDError(#[from] uuid::Error),

    #[error("Serialization error (Bincode): {0}")]
    EncodeError(#[from] bincode::error::EncodeError),

    #[error("Deserialization error (Bincode): {0}")]
    DecodeError(#[from] bincode::error::DecodeError),

    #[error("Error parsing URL: {0}")]
    URLError(#[from] url::ParseError),

    #[error("Database error: {0}")]
    DBError(#[from] sled::Error),

    #[error("Unable to read {0} from any source")]
    NoValueFor(String),

    #[error("Metadata error: {0}")]
    MetadataError(#[from] daedalus::Error),

    #[error("Minecraft authentication Hydra error: {0}")]
    HydraError(String),

    #[error("Minecraft authentication task error: {0}")]
    AuthTaskError(#[from] crate::state::AuthTaskError),

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

    #[error("Join handle error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Recv error: {0}")]
    RecvError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error(
        "Tried to access unloaded profile {0}, loading it probably failed"
    )]
    UnloadedProfileError(String),

    #[error("Profile {0} is not managed by Theseus!")]
    UnmanagedProfileError(String),

    #[error("Could not create profile: {0}")]
    ProfileCreationError(#[from] profile_create::ProfileCreationError),

    #[error("Error: {0}")]
    OtherError(String),
}

#[derive(Debug)]
pub struct Error {
    source: tracing_error::TracedError<ErrorKind>,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.source)
    }
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        Self {
            source: Into::<ErrorKind>::into(source).in_current_span(),
        }
    }
}

impl ErrorKind {
    pub fn as_error(self) -> Error {
        self.into()
    }
}

pub type Result<T> = core::result::Result<T, Error>;
