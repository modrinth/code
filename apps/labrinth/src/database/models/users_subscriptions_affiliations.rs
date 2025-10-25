use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::models::{
    DBAffiliateCodeId, DBChargeId, DBUserSubscriptionId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBUsersSubscriptionsAffiliations {
    pub id: i64,
    pub subscription_id: DBUserSubscriptionId,
    pub affiliate_code: DBAffiliateCodeId,
    pub deactivated_at: Option<DateTime<Utc>>,
}

impl DBUsersSubscriptionsAffiliations {
    pub async fn insert<'a, E>(&mut self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let id = sqlx::query_scalar!(
            "
            INSERT INTO users_subscriptions_affiliations
                (subscription_id, affiliate_code, deactivated_at)
            VALUES ($1, $2, $3)
            RETURNING id
            ",
            self.subscription_id.0,
            self.affiliate_code.0,
            self.deactivated_at,
        )
        .fetch_one(exec)
        .await?;

        self.id = id;
        Ok(())
    }

    pub async fn update<'a, E>(&mut self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        sqlx::query!(
            "UPDATE users_subscriptions_affiliations
            SET subscription_id = $1, affiliate_code = $2, deactivated_at = $3
            WHERE id = $4",
            self.subscription_id.0,
            self.affiliate_code.0,
            self.deactivated_at,
            self.id
        )
        .execute(exec)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBUsersSubscriptionsAffiliationsPayouts {
    pub id: i64,
    pub charge_id: DBChargeId,
    pub subscription_id: DBUserSubscriptionId,
    pub affiliate_code: DBAffiliateCodeId,
    pub payout_value_id: i64,
}

impl DBUsersSubscriptionsAffiliationsPayouts {
    pub async fn insert<'a, E>(&mut self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let id = sqlx::query_scalar!(
            "
            INSERT INTO users_subscriptions_affiliations_payouts
                (charge_id, subscription_id, affiliate_code, payout_value_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            ",
            self.charge_id.0,
            self.subscription_id.0,
            self.affiliate_code.0,
            self.payout_value_id,
        )
        .fetch_one(exec)
        .await?;

        self.id = id;
        Ok(())
    }
}
