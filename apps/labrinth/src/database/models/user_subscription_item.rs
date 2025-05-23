use crate::database::models::{
    DBProductPriceId, DBUserId, DBUserSubscriptionId, DatabaseError,
};
use crate::models::billing::{
    PriceDuration, SubscriptionMetadata, SubscriptionStatus,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::convert::{TryFrom, TryInto};

pub struct DBUserSubscription {
    pub id: DBUserSubscriptionId,
    pub user_id: DBUserId,
    pub price_id: DBProductPriceId,
    pub interval: PriceDuration,
    pub created: DateTime<Utc>,
    pub status: SubscriptionStatus,
    pub metadata: Option<SubscriptionMetadata>,
}

struct UserSubscriptionQueryResult {
    id: i64,
    user_id: i64,
    price_id: i64,
    interval: String,
    pub created: DateTime<Utc>,
    pub status: String,
    pub metadata: serde_json::Value,
}

macro_rules! select_user_subscriptions_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            UserSubscriptionQueryResult,
            r#"
            SELECT
                us.id, us.user_id, us.price_id, us.interval, us.created, us.status, us.metadata
            FROM users_subscriptions us
            "#
                + $predicate,
            $param
        )
    };
}

impl TryFrom<UserSubscriptionQueryResult> for DBUserSubscription {
    type Error = serde_json::Error;

    fn try_from(r: UserSubscriptionQueryResult) -> Result<Self, Self::Error> {
        Ok(DBUserSubscription {
            id: DBUserSubscriptionId(r.id),
            user_id: DBUserId(r.user_id),
            price_id: DBProductPriceId(r.price_id),
            interval: PriceDuration::from_string(&r.interval),
            created: r.created,
            status: SubscriptionStatus::from_string(&r.status),
            metadata: serde_json::from_value(r.metadata)?,
        })
    }
}

impl DBUserSubscription {
    pub async fn get(
        id: DBUserSubscriptionId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBUserSubscription>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        ids: &[DBUserSubscriptionId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBUserSubscription>, DatabaseError> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results = select_user_subscriptions_with_predicate!(
            "WHERE us.id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch_all(exec)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_all_user(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBUserSubscription>, DatabaseError> {
        let user_id = user_id.0;
        let results = select_user_subscriptions_with_predicate!(
            "WHERE us.user_id = $1",
            user_id
        )
        .fetch_all(exec)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_all_servers(
        status: Option<SubscriptionStatus>,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBUserSubscription>, DatabaseError> {
        let status = status.map(|x| x.as_str());

        let results = select_user_subscriptions_with_predicate!(
            r#"
            INNER JOIN products_prices pp ON us.price_id = pp.id
            INNER JOIN products p ON p.metadata  @> '{"type": "pyro"}'
            WHERE $1::text IS NULL OR us.status = $1::text
            GROUP BY us.id
            "#,
            status
        )
        .fetch_all(exec)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO users_subscriptions (
                id, user_id, price_id, interval, created, status, metadata
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            ON CONFLICT (id)
            DO UPDATE
                SET interval = EXCLUDED.interval,
                    status = EXCLUDED.status,
                    price_id = EXCLUDED.price_id,
                    metadata = EXCLUDED.metadata
            ",
            self.id.0,
            self.user_id.0,
            self.price_id.0,
            self.interval.as_str(),
            self.created,
            self.status.as_str(),
            serde_json::to_value(&self.metadata)?,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
