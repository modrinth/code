use std::io;

pub use meta::Metadata;
pub use profiles::{Profile, Profiles};
pub use settings::Settings;

mod meta;
pub mod profiles;
mod settings;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("I/O error while reading data: {0}")]
    IOError(#[from] io::Error),

    #[error("Daedalus error: {0}")]
    DaedalusError(#[from] daedalus::Error),

    #[error("Data format error: {0}")]
    FormatError(String),

    #[error("Attempted to access {0} without initializing it!")]
    InitializedError(String),

    #[error("Error while serializing/deserializing data")]
    SerdeError(#[from] serde_json::Error),
}
