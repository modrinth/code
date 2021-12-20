use std::io;

pub use meta::Metadata;
pub use settings::Settings;

mod meta;
mod settings;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("I/O error while reading data: {0}")]
    IOError(#[from] io::Error),

    #[error("Daedalus error: {0}")]
    DaedalusError(#[from] daedalus::Error),

    #[error("Attempted to access {0} without initializing it!")]
    InitializedError(String),

    #[error("Error while serializing/deserializing data")]
    SerdeError(#[from] serde_json::Error),
}
