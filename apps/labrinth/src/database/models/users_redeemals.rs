use crate::database::models::DBUserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_scalar};
use std::fmt;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Offer {
    #[default]
    Medal,
}

impl Offer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Offer::Medal => "medal",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "medal" => Offer::Medal,
            _ => Offer::Medal,
        }
    }
}

impl fmt::Display for Offer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Pending,
    Redeemed,
    Expired,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Redeemed => "redeemed",
            Status::Expired => "expired",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "pending" => Status::Pending,
            "redeemed" => Status::Redeemed,
            "expired" => Status::Expired,
            _ => Status::Pending,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct UserRedeemal {
    pub id: i32,
    pub user_id: DBUserId,
    pub offer: Offer,
    pub redeemed: DateTime<Utc>,
    pub status: Status,
}

impl UserRedeemal {
    pub async fn exists_by_user_and_offer<'a, E>(
        exec: E,
        user_id: DBUserId,
        offer: Offer,
    ) -> sqlx::Result<bool>
    where
        E: sqlx::PgExecutor<'a>,
    {
        query_scalar!(
            r#"SELECT
              EXISTS (
                SELECT
                  1
                FROM
                  users_redeemals
                WHERE
                  user_id = $1
                  AND offer = $2
               ) AS "exists!"
            "#,
            user_id.0,
            offer.as_str(),
        )
        .fetch_one(exec)
        .await
    }

    pub async fn insert<'a, E>(&mut self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let query = query_scalar!(
            r#"
          INSERT INTO users_redeemals
          (user_id, offer, redeemed, status)
          VALUES ($1, $2, $3, $4)
          RETURNING id"#,
            self.user_id.0,
            self.offer.as_str(),
            self.redeemed,
            self.status.as_str(),
        );

        let id = query.fetch_one(exec).await?;

        self.id = id;

        Ok(())
    }

    pub async fn update<'a, E>(&self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let query = query!(
            r#"
          UPDATE users_redeemals
          SET
            offer = $2,
            status = $3,
            redeemed = $4
          WHERE id = $1
          "#,
            self.id,
            self.offer.as_str(),
            self.status.as_str(),
            self.redeemed,
        );

        query.execute(exec).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct RedeemalLookupFields {
    pub user_id: DBUserId,
    pub redeemal_status: Option<Status>,
}

impl RedeemalLookupFields {
    /// Returns the redeemal status of a user for an offer, while looking up the user
    /// itself. **This expects a single redeemal per user/offer pair**.
    ///
    /// If the returned value is `Ok(None)`, the user doesn't exist.
    ///
    /// If the returned value is `Ok(Some(fields))`, but `redeemal_status` is `None`,
    /// the user exists and has not redeemed the offer.
    pub async fn redeemal_status_by_username_and_offer<'a, E>(
        exec: E,
        user_username: &str,
        offer: Offer,
    ) -> sqlx::Result<Option<RedeemalLookupFields>>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let maybe_row = query!(
            r#"
          SELECT
            users.id,
            users_redeemals.status AS "status: Option<String>"
          FROM
            users
          LEFT JOIN
            users_redeemals ON users_redeemals.user_id = users.id
              AND users_redeemals.offer = $2
          WHERE
            users.username = $1
          ORDER BY
            users_redeemals.redeemed DESC
          LIMIT 1
          "#,
            user_username,
            offer.as_str(),
        )
        .fetch_optional(exec)
        .await?;

        // If no row was returned, the user doesn't exist.
        // If a row NULL status was returned, the user exists but has no redeemed the offer.

        Ok(maybe_row.map(|row| RedeemalLookupFields {
            user_id: DBUserId(row.id),
            redeemal_status: row
                .status
                .as_deref()
                .map(Status::from_str_or_default),
        }))
    }
}
