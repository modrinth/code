use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};

use crate::database::models::{DBAffiliateCodeId, DBUserId, DatabaseError};

#[derive(Debug)]
pub struct DBAffiliateCode {
    pub id: DBAffiliateCodeId,
    pub created_at: DateTime<Utc>,
    pub created_by: DBUserId,
    pub affiliate: DBUserId,
    pub source_name: String,
}

impl DBAffiliateCode {
    pub async fn get_by_id(
        id: DBAffiliateCodeId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBAffiliateCode>, DatabaseError> {
        let record = sqlx::query!(
            "SELECT id, created_at, created_by, affiliate, source_name
            FROM affiliate_codes WHERE id = $1",
            id as DBAffiliateCodeId
        )
        .fetch_optional(exec)
        .await?;

        Ok(record.map(|record| DBAffiliateCode {
            id: DBAffiliateCodeId(record.id),
            created_at: record.created_at,
            created_by: DBUserId(record.created_by),
            affiliate: DBUserId(record.affiliate),
            source_name: record.source_name,
        }))
    }

    pub async fn get_by_affiliate(
        affiliate: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBAffiliateCode>, DatabaseError> {
        let records = sqlx::query!(
            "SELECT id, created_at, created_by, affiliate, source_name
            FROM affiliate_codes WHERE affiliate = $1",
            affiliate as DBUserId
        )
        .fetch(exec)
        .map(|record| {
            let record = record?;
            Ok::<_, DatabaseError>(DBAffiliateCode {
                id: DBAffiliateCodeId(record.id),
                created_at: record.created_at,
                created_by: DBUserId(record.created_by),
                affiliate: DBUserId(record.affiliate),
                source_name: record.source_name,
            })
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(records)
    }

    pub async fn insert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, source_name)
            VALUES ($1, $2, $3, $4, $5)",
            self.id as DBAffiliateCodeId,
            self.created_at,
            self.created_by as DBUserId,
            self.affiliate as DBUserId,
            self.source_name
        )
        .execute(exec)
        .await?;
        Ok(())
    }

    pub async fn remove(
        id: DBAffiliateCodeId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<()>, DatabaseError> {
        let result = sqlx::query!(
            "DELETE FROM affiliate_codes WHERE id = $1",
            id as DBAffiliateCodeId
        )
        .execute(exec)
        .await?;

        if result.rows_affected() > 0 {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn update_source_name(
        id: DBAffiliateCodeId,
        source_name: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "UPDATE affiliate_codes SET source_name = $1 WHERE id = $2",
            source_name,
            id as DBAffiliateCodeId
        )
        .execute(exec)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBAffiliateCode>, DatabaseError> {
        let records = sqlx::query!(
            "SELECT id, created_at, created_by, affiliate, source_name
            FROM affiliate_codes ORDER BY created_at DESC"
        )
        .fetch(exec)
        .map(|record| {
            let record = record?;
            Ok::<_, DatabaseError>(DBAffiliateCode {
                id: DBAffiliateCodeId(record.id),
                created_at: record.created_at,
                created_by: DBUserId(record.created_by),
                affiliate: DBUserId(record.affiliate),
                source_name: record.source_name,
            })
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(records)
    }
}
