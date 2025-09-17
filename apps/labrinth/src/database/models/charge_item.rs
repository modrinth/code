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
    tax_amount: i64,
    tax_platform_id: Option<String>,
    tax_last_updated: Option<DateTime<Utc>>,
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
            tax_amount: r.tax_amount,
            tax_platform_id: r.tax_platform_id,
            net: r.net,
            tax_last_updated: r.tax_last_updated,
        })
    }
}

macro_rules! select_charges_with_predicate {
    ($predicate:tt, $param:ident) => {
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
				charges.tax_last_updated AS "tax_last_updated?"
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
            INSERT INTO charges (id, user_id, price_id, amount, currency_code, charge_type, status, due, last_attempt, subscription_id, subscription_interval, payment_platform, payment_platform_id, parent_charge_id, net, tax_amount, tax_platform_id, tax_last_updated)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
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
            self.tax_amount,
            self.tax_platform_id.as_deref(),
            self.tax_last_updated,
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
            "WHERE subscription_id = $1 AND (status = 'open' OR status = 'expiring' OR status = 'cancelled' OR status = 'failed')",
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
                    (status = 'expiring' AND due < NOW()) OR
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

    /// Returns all charges which are missing a tax identifier, that is, are 1. succeeded, 2. have a tax amount and
    /// 3. haven't been assigned a tax identifier yet.
    ///
    /// Charges are locked.
    pub async fn get_missing_tax_identifier_lock(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        limit: i64,
    ) -> Result<Vec<DBCharge>, DatabaseError> {
        let res = select_charges_with_predicate!(
            "
			WHERE
			  status = 'succeeded'
			  AND tax_platform_id IS NULL
			  AND tax_amount <> 0
			ORDER BY due ASC
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

/// Returns open charges, alongside customer information, which are missing tax amount, ordered
/// by when they were last updated.
pub async fn get_missing_tax_with_limit(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    limit: i64,
) -> Result<Vec<CustomerCharge>, DatabaseError> {
    struct QueryResult {
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
        stripe_customer_id: String,
        product_tax_id: String,
    }

    impl QueryResult {
        /// Destructures the query result into the charge, Stripe CustomerID and Anrok product ID.
        pub fn into_all(self) -> (ChargeQueryResult, String, String) {
            (
                ChargeQueryResult {
                    id: self.id,
                    user_id: self.user_id,
                    price_id: self.price_id,
                    amount: self.amount,
                    currency_code: self.currency_code,
                    status: self.status,
                    due: self.due,
                    last_attempt: self.last_attempt,
                    charge_type: self.charge_type,
                    subscription_id: self.subscription_id,
                    subscription_interval: self.subscription_interval,
                    payment_platform: self.payment_platform,
                    payment_platform_id: self.payment_platform_id,
                    parent_charge_id: self.parent_charge_id,
                    tax_amount: self.tax_amount,
                    tax_platform_id: self.tax_platform_id,
                    tax_last_updated: self.tax_last_updated,
                    net: self.net,
                },
                self.stripe_customer_id,
                self.product_tax_id,
            )
        }
    }

    sqlx::query_as!(
		QueryResult,
		r#"
		SELECT
		  c.id, c.user_id, c.price_id, c.amount, c.currency_code, c.status, c.due, c.last_attempt,
		  c.charge_type, c.subscription_id, c.tax_amount, c.tax_platform_id,
		  -- Workaround for https://github.com/launchbadge/sqlx/issues/3336
		  c.subscription_interval AS "subscription_interval?",
		  c.payment_platform,
		  c.payment_platform_id AS "payment_platform_id?",
		  c.parent_charge_id AS "parent_charge_id?",
		  c.net AS "net?",
		  c.tax_last_updated AS "tax_last_updated?",
		  u.stripe_customer_id AS "stripe_customer_id!",
		  pti.tax_processor_id AS "product_tax_id!"
		FROM charges c
		INNER JOIN users u ON u.id = c.user_id
		INNER JOIN products_prices pp ON pp.id = c.price_id
		INNER JOIN products p ON p.id = pp.product_id
		INNER JOIN products_tax_identifiers pti ON pti.product_id = p.id
		WHERE
		  c.status = 'open'
		  AND c.tax_amount = 0
		  AND u.stripe_customer_id IS NOT NULL
		  AND COALESCE(tax_last_updated, '-infinity' :: TIMESTAMPTZ) + INTERVAL '10 minutes' < NOW()
		ORDER BY COALESCE(c.tax_last_updated, '-infinity' :: TIMESTAMPTZ) DESC
		LIMIT $1
		"#,
		limit
	)
	.fetch_all(exec)
	.await?
	.into_iter()
	.map(|r| {
		let (charge_query_result, stripe_customer_id, product_tax_id) = r.into_all();

		charge_query_result.try_into().map_err(|e: serde_json::Error| {
			DatabaseError::SchemaError(e.to_string())
		}).map(move |charge| CustomerCharge {
			charge,
			stripe_customer_id,
			product_tax_id,
		})
	})
	.collect::<Result<Vec<_>, DatabaseError>>()
}
