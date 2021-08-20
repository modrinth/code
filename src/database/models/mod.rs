#![allow(dead_code)]
// TODO: remove attr once routes are created

use thiserror::Error;

pub mod categories;
pub mod ids;
pub mod notification_item;
pub mod project_item;
pub mod report_item;
pub mod team_item;
pub mod user_item;
pub mod version_item;

pub use ids::*;
pub use project_item::Project;
pub use team_item::Team;
pub use team_item::TeamMember;
pub use user_item::User;
pub use version_item::FileHash;
pub use version_item::Version;
pub use version_item::VersionFile;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    DatabaseError(#[from] sqlx::error::Error),
    #[error("Error while trying to generate random ID")]
    RandomIdError,
    #[error(
        "Invalid identifier: Category/version names must contain only ASCII \
             alphanumeric characters or '_-'."
    )]
    InvalidIdentifier(String),
    #[error("Invalid permissions bitflag!")]
    BitflagError,
    #[error("A database request failed")]
    Other(String),
}

impl ids::StatusId {
    pub async fn get_id<'a, E>(
        status: &crate::models::projects::ProjectStatus,
        exec: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM statuses
            WHERE status = $1
            ",
            status.as_str()
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ids::StatusId(r.id)))
    }
}

impl ids::SideTypeId {
    pub async fn get_id<'a, E>(
        side: &crate::models::projects::SideType,
        exec: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM side_types
            WHERE name = $1
            ",
            side.as_str()
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ids::SideTypeId(r.id)))
    }
}

impl ids::DonationPlatformId {
    pub async fn get_id<'a, E>(id: &str, exec: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM donation_platforms
            WHERE short = $1
            ",
            id
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ids::DonationPlatformId(r.id)))
    }
}

impl ids::ProjectTypeId {
    pub async fn get_id<'a, E>(project_type: String, exec: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM project_types
            WHERE name = $1
            ",
            project_type
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ids::ProjectTypeId(r.id)))
    }
}
