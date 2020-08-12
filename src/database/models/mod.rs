#![allow(unused)]
// TODO: remove attr once routes are created

use thiserror::Error;

pub mod categories;
pub mod ids;
pub mod mod_item;
pub mod team_item;
pub mod version_item;

pub use ids::*;
pub use mod_item::Mod;
pub use team_item::Team;
pub use team_item::TeamMember;
pub use version_item::FileHash;
pub use version_item::Version;
pub use version_item::VersionFile;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database")]
    DatabaseError(#[from] sqlx::error::Error),
    #[error("Error while trying to generate random ID")]
    RandomIdError,
    #[error(
        "Invalid identifier: Category/version names must contain only ASCII \
             alphanumeric characters or '_-'."
    )]
    InvalidIdentifier(String),
}
