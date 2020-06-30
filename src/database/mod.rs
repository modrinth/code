pub mod models;
mod mongo_database;

pub use models::Mod;
pub use models::Version;
pub use mongo_database::connect;
use thiserror::Error;

type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Impossible to find document")]
    NotFound(),
    #[error("BSON deserialization error")]
    BsonError(#[from] bson::de::Error),
    #[error("Local database error")]
    LocalDatabaseError(#[from] mongodb::error::Error),
}
