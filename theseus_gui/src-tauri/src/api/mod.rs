use serde::Serialize;
use thiserror::Error;

pub mod profile;
pub mod profile_create;

type Result<T> = std::result::Result<T, TheseusGuiError>;

#[derive(Error, Debug, Serialize)]
pub enum TheseusGuiError {

    #[error("Error creating profile")]
    ProfileCreationError

}