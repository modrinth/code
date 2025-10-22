use crate::database::models::{
    DBChargeId, DBProductPriceId, DBUserId, DBUserSubscriptionId, DatabaseError,
};
use crate::models::billing::{
    ChargeStatus, ChargeType, PaymentPlatform, PriceDuration,
};
use chrono::{DateTime, Utc};
use std::convert::{TryFrom, TryInto};

#[derive(Clone)]
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

    pub tax_amount: i64,
    pub tax_platform_id: Option<String>,
    pub tax_last_updated: Option<DateTime<Utc>>,
    pub tax_transaction_version: Option<i32>,
    pub tax_platform_accounting_time: Option<DateTime<Utc>>,

    // Net is always in USD
    pub net: Option<i64>,
    pub tax_drift_loss: Option<i64>,
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
    tax_amount: i64,
    tax_platform_id: Option<String>,
    tax_last_updated: Option<DateTime<Utc>>,
    net: Option<i64>,
    tax_drift_loss: Option<i64>,
    tax_transaction_version: Option<i32>,
    tax_platform_accounting_time: Option<DateTime<Utc>>,
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
            tax_amount: r.tax_amount,
            tax_platform_id: r.tax_platform_id,
            net: r.net,
            tax_last_updated: r.tax_last_updated,
            tax_drift_loss: r.tax_drift_loss,
            tax_transaction_version: r.tax_transaction_version,
            tax_platform_accounting_time: r.tax_platform_accounting_time,
        })
    }
}

macro_rules! select_charges_with_predicate {
    ($predicate:tt $(, $( $param0:expr $(, $param:expr)* $(,)? )?)?) => {
        sqlx::query_as!(
            ChargeQueryResult,
            r#"
            SELECT
                charges.id, charges.user_id, charges.price_id, charges.amount, charges.currency_code, charges.status, charges.due, charges.last_attempt,
                charges.charge_type, charges.subscription_id, charges.tax_amount, charges.tax_platform_id,
                -- Workaround for https://github.com/launchbadge/sqlx/issues/3336
                charges.subscription_interval AS "subscription_interval?",
                charges.payment_platform,
                charges.payment_platform_id AS "payment_platform_id?",
                charges.parent_charge_id AS "parent_charge_id?",
                charges.net AS "net?",
				charges.tax_last_updated AS "tax_last_updated?",
				charges.tax_drift_loss AS "tax_drift_loss?",
                charges.tax_transaction_version AS "tax_transaction_version?",
                charges.tax_platform_accounting_time AS "tax_platform_accounting_time?"
            FROM charges
            "#
                + $predicate,
            $( $( $param0, $( $param ),* )? )?
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
            INSERT INTO charges (id, user_id, price_id, amount, currency_code, charge_type, status, due, last_attempt, subscription_id, subscription_interval, payment_platform, payment_platform_id, parent_charge_id, net, tax_amount, tax_platform_id, tax_last_updated, tax_drift_loss, tax_transaction_version, tax_platform_accounting_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
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
                    tax_amount = EXCLUDED.tax_amount,
                    tax_platform_id = EXCLUDED.tax_platform_id,
                    tax_last_updated = EXCLUDED.tax_last_updated,
                    price_id = EXCLUDED.price_id,
                    amount = EXCLUDED.amount,
                    currency_code = EXCLUDED.currency_code,
                    charge_type = EXCLUDED.charge_type,
					tax_drift_loss = EXCLUDED.tax_drift_loss,
					tax_transaction_version = EXCLUDED.tax_transaction_version,
					tax_platform_accounting_time = EXCLUDED.tax_platform_accounting_time
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
            self.tax_amount,
            self.tax_platform_id.as_deref(),
            self.tax_last_updated,
            self.tax_drift_loss,
            self.tax_transaction_version,
            self.tax_platform_accounting_time,
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
            "WHERE
			  subscription_id = $1
			  AND (status = 'open' OR status = 'expiring' OR status = 'cancelled' OR status = 'failed')
			ORDER BY due ASC LIMIT 1",
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
            INNER JOIN users_subscriptions us ON us.id = charges.subscription_id
            WHERE
                charges.charge_type = $1 AND
                (
                    (charges.status = 'cancelled' AND charges.due < NOW()) OR
                    (charges.status = 'expiring' AND charges.due < NOW()) OR
                    (charges.status = 'failed' AND charges.last_attempt < NOW() - INTERVAL '2 days')
                )
                AND us.status = 'provisioned'
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

    /// Returns all charges that need to have their tax amount updated.
    ///
    /// This only selects charges which are:
    /// - Open;
    /// - Haven't been updated in the last day;
    /// - Are due in more than 7 days;
    /// - Where the user has an email, because we can't notify users without an email about a price change.
    ///
    /// This also locks the charges.
    pub async fn get_updateable_lock(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        limit: i64,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let res = select_charges_with_predicate!(
			"
			INNER JOIN users u ON u.id = charges.user_id
			WHERE
			  status = 'open'
			  AND COALESCE(tax_last_updated, '-infinity' :: TIMESTAMPTZ) < NOW() - INTERVAL '1 day'
			  AND u.email IS NOT NULL
			  AND due - INTERVAL '7 days' > NOW()
              AND due - INTERVAL '30 days' < NOW() -- Due between 7 and 30 days from now
			ORDER BY COALESCE(tax_last_updated, '-infinity' :: TIMESTAMPTZ) ASC
			FOR NO KEY UPDATE SKIP LOCKED
			LIMIT $1
			",
			limit
		)
		.fetch_all(exec)
		.await?;

        Ok(res
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, serde_json::Error>>()?)
    }

    /// Returns all charges which are missing a tax identifier, that is, are succeeded and haven't been assigned a tax identifier yet.
    ///
    /// Charges are locked.
    pub async fn get_missing_tax_identifier_lock(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let res = select_charges_with_predicate!(
            "
			WHERE
			  status = 'succeeded'
			  AND tax_platform_id IS NULL
              AND payment_platform_id IS NOT NULL
			ORDER BY due ASC
			FOR NO KEY UPDATE SKIP LOCKED
            OFFSET $1
			LIMIT $2
			",
            offset,
            limit
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

pub struct CustomerCharge {
    pub stripe_customer_id: String,
    pub charge: DBCharge,
    pub product_tax_id: String,
}
