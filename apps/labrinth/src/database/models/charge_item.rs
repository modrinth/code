use crate::database::models::{
    DBChargeId, DBProductPriceId, DBUserId, DBUserSubscriptionId, DatabaseError,
};
use crate::models::billing::{
    ChargeStatus, ChargeType, PaymentPlatform, PriceDuration,
};
use chrono::{DateTime, Utc};
use std::convert::{TryFrom, TryInto};

pub struct DBCharge {
    pub id: DBChargeId,
    pub user_id: DBUserId,
    pub price_id: DBProductPriceId,
    pub amount: i64,
    pub currency_code: String,
    pub status: ChargeStatus,
    pub due: DateTime<Utc>,
    pub last_attempt: Option<DateTime<Utc>>,

    pub type_: ChargeType,
    pub subscription_id: Option<DBUserSubscriptionId>,
    pub subscription_interval: Option<PriceDuration>,

    pub payment_platform: PaymentPlatform,
    pub payment_platform_id: Option<String>,

    pub parent_charge_id: Option<DBChargeId>,

    // Net is always in USD
    pub net: Option<i64>,
}

struct ChargeQueryResult {
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
    payment_platform: String,
    payment_platform_id: Option<String>,
    parent_charge_id: Option<i64>,
    net: Option<i64>,
}

impl TryFrom<ChargeQueryResult> for DBCharge {
    type Error = serde_json::Error;

    fn try_from(r: ChargeQueryResult) -> Result<Self, Self::Error> {
        Ok(DBCharge {
            id: DBChargeId(r.id),
            user_id: DBUserId(r.user_id),
            price_id: DBProductPriceId(r.price_id),
            amount: r.amount,
            currency_code: r.currency_code,
            status: ChargeStatus::from_string(&r.status),
            due: r.due,
            last_attempt: r.last_attempt,
            type_: ChargeType::from_string(&r.charge_type),
            subscription_id: r.subscription_id.map(DBUserSubscriptionId),
            subscription_interval: r
                .subscription_interval
                .map(|x| PriceDuration::from_string(&x)),
            payment_platform: PaymentPlatform::from_string(&r.payment_platform),
            payment_platform_id: r.payment_platform_id,
            parent_charge_id: r.parent_charge_id.map(DBChargeId),
            net: r.net,
        })
    }
}

macro_rules! select_charges_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            ChargeQueryResult,
            r#"
            SELECT
                id, user_id, price_id, amount, currency_code, status, due, last_attempt,
                charge_type, subscription_id,
                -- Workaround for https://github.com/launchbadge/sqlx/issues/3336
                subscription_interval AS "subscription_interval?",
                payment_platform,
                payment_platform_id AS "payment_platform_id?",
                parent_charge_id AS "parent_charge_id?",
                net AS "net?"
            FROM charges
            "#
                + $predicate,
            $param
        )
    };
}

impl DBCharge {
    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DBChargeId, DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO charges (id, user_id, price_id, amount, currency_code, charge_type, status, due, last_attempt, subscription_id, subscription_interval, payment_platform, payment_platform_id, parent_charge_id, net)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (id)
            DO UPDATE
                SET status = EXCLUDED.status,
                    last_attempt = EXCLUDED.last_attempt,
                    due = EXCLUDED.due,
                    subscription_id = EXCLUDED.subscription_id,
                    subscription_interval = EXCLUDED.subscription_interval,
                    payment_platform = EXCLUDED.payment_platform,
                    payment_platform_id = EXCLUDED.payment_platform_id,
                    parent_charge_id = EXCLUDED.parent_charge_id,
                    net = EXCLUDED.net,
                    price_id = EXCLUDED.price_id,
                    amount = EXCLUDED.amount,
                    currency_code = EXCLUDED.currency_code,
                    charge_type = EXCLUDED.charge_type
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
            self.payment_platform.as_str(),
            self.payment_platform_id.as_deref(),
            self.parent_charge_id.map(|x| x.0),
            self.net,
        )
            .execute(&mut **transaction)
        .await?;

        Ok(self.id)
    }

    pub async fn get(
        id: DBChargeId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBCharge>, DatabaseError> {
        let id = id.0;
        let res = select_charges_with_predicate!("WHERE id = $1", id)
            .fetch_optional(exec)
            .await?;

        Ok(res.and_then(|r| r.try_into().ok()))
    }

    pub async fn get_from_user(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
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

    pub async fn get_children(
        charge_id: DBChargeId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let charge_id = charge_id.0;
        let res = select_charges_with_predicate!(
            "WHERE parent_charge_id = $1",
            charge_id
        )
        .fetch_all(exec)
        .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_open_subscription(
        user_subscription_id: DBUserSubscriptionId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBCharge>, DatabaseError> {
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
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let charge_type = ChargeType::Subscription.as_str();
        let res = select_charges_with_predicate!(
            r#"
            WHERE
                charge_type = $1 AND
                (
                    (status = 'open' AND due < NOW()) OR
                    (status = 'failed' AND last_attempt < NOW() - INTERVAL '2 days')
                )
            "#,
            charge_type
        )
            .fetch_all(exec)
            .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_unprovision(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let charge_type = ChargeType::Subscription.as_str();
        let res = select_charges_with_predicate!(
            r#"
            WHERE
                charge_type = $1 AND
                (
                    (status = 'cancelled' AND due < NOW()) OR
                    (status = 'failed' AND last_attempt < NOW() - INTERVAL '2 days')
                )
            "#,
            charge_type
        )
            .fetch_all(exec)
            .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn get_cancellable(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let charge_type = ChargeType::Subscription.as_str();
        let res = select_charges_with_predicate!(
            r#"
            WHERE
                charge_type = $1 AND
                status = 'failed' AND due < NOW() - INTERVAL '30 days'
            "#,
            charge_type
        )
        .fetch_all(exec)
        .await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    pub async fn remove(
        id: DBChargeId,
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
