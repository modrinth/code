use crate::database::models::{DBUserId, DBUserSubscriptionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, query_scalar};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBUserSubscriptionCredit {
    pub id: i32,
    pub subscription_id: DBUserSubscriptionId,
    pub user_id: DBUserId,
    pub creditor_id: DBUserId,
    pub days: i32,
    pub previous_due: DateTime<Utc>,
    pub next_due: DateTime<Utc>,
    pub created: DateTime<Utc>,
}

impl DBUserSubscriptionCredit {
    /// Inserts this audit entry and sets its id.
    pub async fn insert<'a, E>(&mut self, exec: E) -> sqlx::Result<()>
    where
        E: PgExecutor<'a>,
    {
        let id = query_scalar!(
            r#"
            INSERT INTO users_subscriptions_credits
              (subscription_id, user_id, creditor_id, days, previous_due, next_due)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            self.subscription_id.0,
            self.user_id.0,
            self.creditor_id.0,
            self.days,
            self.previous_due,
            self.next_due,
        )
        .fetch_one(exec)
        .await?;

        self.id = id;
        Ok(())
    }
}
