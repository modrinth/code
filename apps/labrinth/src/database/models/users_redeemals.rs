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
    Processing,
    Processed,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Processing => "processing",
            Status::Processed => "processed",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "pending" => Status::Pending,
            "processing" => Status::Processing,
            "processed" => Status::Processed,
            _ => Status::default(),
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
    pub last_attempt: Option<DateTime<Utc>>,
    pub n_attempts: i32,
    pub status: Status,
}

impl UserRedeemal {
    pub async fn get_pending<'a, E>(
        exec: E,
        limit: i64,
    ) -> sqlx::Result<Vec<UserRedeemal>>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let redeemals = query!(
            r#"SELECT * FROM users_redeemals WHERE status = $1 LIMIT $2"#,
            Status::Pending.as_str(),
            limit
        )
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| UserRedeemal {
            id: row.id,
            user_id: DBUserId(row.user_id),
            offer: Offer::from_str_or_default(&row.offer),
            redeemed: row.redeemed,
            last_attempt: row.last_attempt,
            n_attempts: row.n_attempts,
            status: Status::from_str_or_default(&row.status),
        })
        .collect();

        Ok(redeemals)
    }

    pub async fn update_stuck_5_minutes<'a, E>(exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        query!(
            r#"
            UPDATE users_redeemals
            SET status = $1
            WHERE
              status = $2
              AND NOW() - last_attempt > INTERVAL '5 minutes'
            "#,
            Status::Pending.as_str(),
            Status::Processing.as_str(),
        )
        .execute(exec)
        .await?;

        Ok(())
    }

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
            r#"INSERT INTO users_redeemals
            (user_id, offer, redeemed, status, last_attempt, n_attempts)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            self.user_id.0,
            self.offer.as_str(),
            self.redeemed,
            self.status.as_str(),
            self.last_attempt,
            self.n_attempts,
        );

        let id = query.fetch_one(exec).await?;

        self.id = id;

        Ok(())
    }

    /// Updates `status`, `last_attempt`, and `n_attempts` only if `status` is currently pending.
    /// Returns `true` if the status was updated, `false` otherwise.
    pub async fn update_status_if_pending<'a, E>(
        &self,
        exec: E,
    ) -> sqlx::Result<bool>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let query = query!(
            r#"UPDATE users_redeemals
            SET
              status = $3,
              last_attempt = $4,
              n_attempts = $5
            WHERE id = $1 AND status = $2
            "#,
            self.id,
            Status::Pending.as_str(),
            self.status.as_str(),
            self.last_attempt,
            self.n_attempts,
        );

        let query_result = query.execute(exec).await?;

        Ok(query_result.rows_affected() > 0)
    }

    pub async fn update<'a, E>(&self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let query = query!(
            r#"UPDATE users_redeemals
            SET
                offer = $2,
                status = $3,
                redeemed = $4,
                last_attempt = $5,
                n_attempts = $6
            WHERE id = $1
            "#,
            self.id,
            self.offer.as_str(),
            self.status.as_str(),
            self.redeemed,
            self.last_attempt,
            self.n_attempts,
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
