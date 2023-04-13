use thiserror::Error;

pub mod categories;
pub mod ids;
pub mod notification_item;
pub mod project_item;
pub mod report_item;
pub mod team_item;
pub mod thread_item;
pub mod user_item;
pub mod version_item;

pub use ids::*;
pub use project_item::Project;
pub use team_item::Team;
pub use team_item::TeamMember;
pub use thread_item::{Thread, ThreadMessage};
pub use user_item::User;
pub use version_item::Version;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] sqlx::error::Error),
    #[error("Error while trying to generate random ID")]
    RandomId,
    #[error("A database request failed")]
    Other(String),
    #[error("Error while parsing JSON: {0}")]
    Json(#[from] serde_json::Error),
}
