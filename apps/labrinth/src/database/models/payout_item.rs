use crate::models::payouts::{PayoutMethodType, PayoutStatus};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{DBPayoutId, DBUserId, DatabaseError};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBPayout {
    pub id: DBPayoutId,
    pub user_id: DBUserId,
    pub created: DateTime<Utc>,
    pub status: PayoutStatus,
    pub amount: Decimal,

    pub fee: Option<Decimal>,
    pub method: Option<PayoutMethodType>,
    pub method_address: Option<String>,
    pub platform_id: Option<String>,
}

impl DBPayout {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO payouts (
                id, amount, fee, user_id, status, method, method_address, platform_id
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8
            )
            ",
            self.id.0,
            self.amount,
            self.fee,
            self.user_id.0,
            self.status.as_str(),
            self.method.map(|x| x.as_str()),
            self.method_address,
            self.platform_id,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        id: DBPayoutId,
        executor: E,
    ) -> Result<Option<DBPayout>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        DBPayout::get_many(&[id], executor)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        payout_ids: &[DBPayoutId],
        exec: E,
    ) -> Result<Vec<DBPayout>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        let results = sqlx::query!(
            "
            SELECT id, user_id, created, amount, status, method, method_address, platform_id, fee
            FROM payouts
            WHERE id = ANY($1)
            ",
            &payout_ids.into_iter().map(|x| x.0).collect::<Vec<_>>()
        )
        .fetch(exec)
        .map_ok(|r| DBPayout {
            id: DBPayoutId(r.id),
            user_id: DBUserId(r.user_id),
            created: r.created,
            status: PayoutStatus::from_string(&r.status),
            amount: r.amount,
            method: r.method.map(|x| PayoutMethodType::from_string(&x)),
            method_address: r.method_address,
            platform_id: r.platform_id,
            fee: r.fee,
        })
        .try_collect::<Vec<DBPayout>>()
        .await?;

        Ok(results)
    }

    pub async fn get_all_for_user(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBPayoutId>, DatabaseError> {
        let results = sqlx::query!(
            "
            SELECT id
            FROM payouts
            WHERE user_id = $1
            ",
            user_id.0
        )
        .fetch_all(exec)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| DBPayoutId(r.id))
            .collect::<Vec<_>>())
    }
}
