#![allow(dead_code)]
// TODO: remove attr once routes are created

use chrono::{DateTime, Utc};
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
pub use version_item::Version;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] sqlx::error::Error),
    #[error("Error while trying to generate random ID")]
    RandomId,
    #[error("A database request failed")]
    Other(String),
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
    pub async fn get_id<'a, E>(
        id: &str,
        exec: E,
    ) -> Result<Option<Self>, DatabaseError>
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
    pub async fn get_id<'a, E>(
        project_type: String,
        exec: E,
    ) -> Result<Option<Self>, DatabaseError>
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

        Ok(result.map(|r| ProjectTypeId(r.id)))
    }
}

pub fn convert_postgres_date(input: &str) -> DateTime<Utc> {
    let mut result = DateTime::parse_from_str(input, "%Y-%m-%d %T.%f%#z");

    if result.is_err() {
        result = DateTime::parse_from_str(input, "%Y-%m-%d %T%#z")
    }

    result
        .map(|x| x.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
