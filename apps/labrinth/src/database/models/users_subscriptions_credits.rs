use crate::database::models::{DBUserId, DBUserSubscriptionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::query_scalar;

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
        E: sqlx::PgExecutor<'a>,
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

    pub async fn insert_many(
        exec: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subscription_ids: &[DBUserSubscriptionId],
        user_ids: &[DBUserId],
        creditor_ids: &[DBUserId],
        days: &[i32],
        previous_dues: &[DateTime<Utc>],
        next_dues: &[DateTime<Utc>],
    ) -> sqlx::Result<()> {
        debug_assert_eq!(subscription_ids.len(), user_ids.len());
        debug_assert_eq!(user_ids.len(), creditor_ids.len());
        debug_assert_eq!(creditor_ids.len(), days.len());
        debug_assert_eq!(days.len(), previous_dues.len());
        debug_assert_eq!(previous_dues.len(), next_dues.len());

        let subs: Vec<i64> = subscription_ids.iter().map(|x| x.0).collect();
        let users: Vec<i64> = user_ids.iter().map(|x| x.0).collect();
        let creditors: Vec<i64> = creditor_ids.iter().map(|x| x.0).collect();

        sqlx::query!(
            r#"
            INSERT INTO users_subscriptions_credits
              (subscription_id, user_id, creditor_id, days, previous_due, next_due)
            SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::bigint[], $4::int[], $5::timestamptz[], $6::timestamptz[])
            "#,
            &subs[..],
            &users[..],
            &creditors[..],
            &days[..],
            &previous_dues[..],
            &next_dues[..],
        )
        .execute(&mut **exec)
        .await?;

        Ok(())
    }
}
