use crate::database::models::{
    ChargeId, DatabaseError, ProductPriceId, UserId, UserSubscriptionId,
};
use crate::models::billing::{ChargeStatus, ChargeType, PriceDuration};
use chrono::{DateTime, Utc};
use std::convert::{TryFrom, TryInto};

pub struct ChargeItem {
    pub id: ChargeId,
    pub user_id: UserId,
    pub price_id: ProductPriceId,
    pub amount: i64,
    pub currency_code: String,
    pub status: ChargeStatus,
    pub due: DateTime<Utc>,
    pub last_attempt: Option<DateTime<Utc>>,

    pub type_: ChargeType,
    pub subscription_id: Option<UserSubscriptionId>,
    pub subscription_interval: Option<PriceDuration>,
}

struct ChargeResult {
    id: i64,
    user_id: i64,
    price_id: i64,
    amount: i64,
    currency_code: String,
    status: String,
    due: DateTime<Utc>,
    last_attempt: Option<DateTime<Utc>>,
    charge_type: String,
    subscription_id: Option<i64>,
    subscription_interval: Option<String>,
}

impl TryFrom<ChargeResult> for ChargeItem {
    type Error = serde_json::Error;

    fn try_from(r: ChargeResult) -> Result<Self, Self::Error> {
        Ok(ChargeItem {
            id: ChargeId(r.id),
            user_id: UserId(r.user_id),
            price_id: ProductPriceId(r.price_id),
            amount: r.amount,
            currency_code: r.currency_code,
            status: ChargeStatus::from_string(&r.status),
            due: r.due,
            last_attempt: r.last_attempt,
            type_: ChargeType::from_string(&r.charge_type),
            subscription_id: r.subscription_id.map(UserSubscriptionId),
            subscription_interval: r
                .subscription_interval
                .map(|x| PriceDuration::from_string(&x)),
        })
    }
}

macro_rules! select_charges_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            ChargeResult,
            r#"
            SELECT id, user_id, price_id, amount, currency_code, status, due, last_attempt, charge_type, subscription_id, subscription_interval
            FROM charges
            "#
                + $predicate,
            $param
        )
    };
}

impl ChargeItem {
    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<ChargeId, DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO charges (id, user_id, price_id, amount, currency_code, charge_type, status, due, last_attempt, subscription_id, subscription_interval)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id)
            DO UPDATE
                SET status = EXCLUDED.status,
                    last_attempt = EXCLUDED.last_attempt,
                    due = EXCLUDED.due,
                    subscription_id = EXCLUDED.subscription_id,
                    subscription_interval = EXCLUDED.subscription_interval
            "#,
            self.id.0,
            self.user_id.0,
            self.price_id.0,
            self.amount,
            self.currency_code,
            self.type_.as_str(),
            self.status.as_str(),
            self.due,
            self.last_attempt,
            self.subscription_id.map(|x| x.0),
            self.subscription_interval.map(|x| x.as_str()),
        )
            .execute(&mut **transaction)
        .await?;

        Ok(self.id)
    }

    pub async fn get(
        id: ChargeId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<ChargeItem>, DatabaseError> {
        let id = id.0;
        let res = select_charges_with_predicate!("WHERE id = $1", id)
            .fetch_optional(exec)
            .await?;

        Ok(res.and_then(|r| r.try_into().ok()))
    }

    pub async fn get_from_user(
        user_id: UserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<ChargeItem>, DatabaseError> {
        let user_id = user_id.0;
        let res = select_charges_with_predicate!(
            "WHERE user_id = $1 ORDER BY due DESC",
            user_id
        )
        .fetch_all(exec)
        .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_open_subscription(
        user_subscription_id: UserSubscriptionId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<ChargeItem>, DatabaseError> {
        let user_subscription_id = user_subscription_id.0;
        let res = select_charges_with_predicate!(
            "WHERE subscription_id = $1 AND (status = 'open' OR status = 'cancelled')",
            user_subscription_id
        )
        .fetch_optional(exec)
        .await?;

        Ok(res.and_then(|r| r.try_into().ok()))
    }

    pub async fn get_chargeable(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<ChargeItem>, DatabaseError> {
        let now = Utc::now();

        let res = select_charges_with_predicate!("WHERE (status = 'open' AND due < $1) OR (status = 'failed' AND last_attempt < $1 - INTERVAL '2 days')", now)
            .fetch_all(exec)
            .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_unprovision(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<ChargeItem>, DatabaseError> {
        let now = Utc::now();

        let res =
            select_charges_with_predicate!("WHERE (status = 'cancelled' AND due < $1) OR (status = 'failed' AND last_attempt < $1 - INTERVAL '2 days')", now)
            .fetch_all(exec)
            .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn remove(
        id: ChargeId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            DELETE FROM charges
            WHERE id = $1
            ",
            id.0 as i64
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
