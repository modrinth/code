use crate::database::models::{DatabaseError, ProductPriceId, UserId, UserSubscriptionId};
use crate::models::billing::{PriceDuration, SubscriptionStatus};
use chrono::{DateTime, Utc};
use itertools::Itertools;

pub struct UserSubscriptionItem {
    pub id: UserSubscriptionId,
    pub user_id: UserId,
    pub price_id: ProductPriceId,
    pub interval: PriceDuration,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub last_charge: Option<DateTime<Utc>>,
    pub status: SubscriptionStatus,
}

struct UserSubscriptionResult {
    id: i64,
    user_id: i64,
    price_id: i64,
    interval: String,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub last_charge: Option<DateTime<Utc>>,
    pub status: String,
}

macro_rules! select_user_subscriptions_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            UserSubscriptionResult,
            r#"
            SELECT
                id, user_id, price_id, interval, created, expires, last_charge, status
            FROM users_subscriptions
            "#
                + $predicate,
            $param
        )
    };
}

impl From<UserSubscriptionResult> for UserSubscriptionItem {
    fn from(r: UserSubscriptionResult) -> Self {
        UserSubscriptionItem {
            id: UserSubscriptionId(r.id),
            user_id: UserId(r.user_id),
            price_id: ProductPriceId(r.price_id),
            interval: PriceDuration::from_string(&r.interval),
            created: r.created,
            expires: r.expires,
            last_charge: r.last_charge,
            status: SubscriptionStatus::from_string(&r.status),
        }
    }
}

impl UserSubscriptionItem {
    pub async fn get(
        id: UserSubscriptionId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<UserSubscriptionItem>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        ids: &[UserSubscriptionId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<UserSubscriptionItem>, DatabaseError> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results =
            select_user_subscriptions_with_predicate!("WHERE id = ANY($1::bigint[])", ids_ref)
                .fetch_all(exec)
                .await?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_all_user(
        user_id: UserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<UserSubscriptionItem>, DatabaseError> {
        let user_id = user_id.0;
        let results = select_user_subscriptions_with_predicate!("WHERE user_id = $1", user_id)
            .fetch_all(exec)
            .await?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_all_expired(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<UserSubscriptionItem>, DatabaseError> {
        let now = Utc::now();
        let results = select_user_subscriptions_with_predicate!("WHERE expires < $1", now)
            .fetch_all(exec)
            .await?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO users_subscriptions (
                id, user_id, price_id, interval, created, expires, last_charge, status
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8
            )
            ON CONFLICT (id)
            DO UPDATE
                SET interval = EXCLUDED.interval,
                    expires = EXCLUDED.expires,
                    last_charge = EXCLUDED.last_charge,
                    status = EXCLUDED.status
            ",
            self.id.0,
            self.user_id.0,
            self.price_id.0,
            self.interval.as_str(),
            self.created,
            self.expires,
            self.last_charge,
            self.status.as_str(),
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove(
        id: UserSubscriptionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            DELETE FROM users_subscriptions
            WHERE id = $1
            ",
            id.0 as i64
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
