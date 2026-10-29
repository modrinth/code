use ariadne::ids::random_base62;
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};

use crate::database::{PgPool, ReadOnlyPgPool};
use serde::{Deserialize, Serialize};

use super::{
    DBUserId, ModerationFlagId, ModerationFlagRuleId, ModerationFlagRunId,
};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBModerationFlagRule {
    pub id: ModerationFlagRuleId,
    pub name: String,
    pub created_by: Option<DBUserId>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<DBUserId>,
    pub updated_at: DateTime<Utc>,
    pub expression: String,
}

impl DBModerationFlagRule {
    pub async fn list(
        limit: i64,
        offset: i64,
        pool: &ReadOnlyPgPool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, expression, created_by, created_at, updated_by, updated_at
            FROM moderation_flag_rules
            ORDER BY created_at DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err("listing moderation flag rules")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: ModerationFlagRuleId(row.id),
                name: row.name,
                expression: row.expression,
                created_by: row.created_by.map(DBUserId),
                created_at: row.created_at,
                updated_by: row.updated_by.map(DBUserId),
                updated_at: row.updated_at,
            })
            .collect())
    }

    pub async fn get(
        id: ModerationFlagRuleId,
        pool: &ReadOnlyPgPool,
    ) -> Result<Option<Self>> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, expression, created_by, created_at, updated_by, updated_at
            FROM moderation_flag_rules
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&**pool)
        .await
        .wrap_err("fetching moderation flag rule")?;

        Ok(row.map(|row| Self {
            id: ModerationFlagRuleId(row.id),
            name: row.name,
            expression: row.expression,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        }))
    }

    pub async fn create(
        name: &str,
        expression: &str,
        user_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Self> {
        let id = random_base62(8) as i64;
        let now = Utc::now();
        let row = sqlx::query!(
            r#"
            INSERT INTO moderation_flag_rules
                (id, name, expression, created_by, created_at, updated_by, updated_at)
            VALUES ($1, $2, $3, $4, $5, $4, $5)
            RETURNING id, name, expression, created_by, created_at, updated_by, updated_at
            "#,
            id,
            name,
            expression,
            user_id.0,
            now,
        )
        .fetch_one(pool)
        .await
        .wrap_err("creating moderation flag rule")?;

        Ok(Self {
            id: ModerationFlagRuleId(row.id),
            name: row.name,
            expression: row.expression,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        })
    }

    pub async fn update(
        id: ModerationFlagRuleId,
        name: &str,
        expression: &str,
        user_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        let now = Utc::now();
        let row = sqlx::query!(
            r#"
            UPDATE moderation_flag_rules
            SET name = $2, expression = $3, updated_by = $4, updated_at = $5
            WHERE id = $1
            RETURNING id, name, expression, created_by, created_at, updated_by, updated_at
            "#,
            id.0,
            name,
            expression,
            user_id.0,
            now,
        )
        .fetch_optional(pool)
        .await
        .wrap_err("updating moderation flag rule")?;

        Ok(row.map(|row| Self {
            id: ModerationFlagRuleId(row.id),
            name: row.name,
            expression: row.expression,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        }))
    }

    pub async fn delete(
        id: ModerationFlagRuleId,
        pool: &PgPool,
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM moderation_flag_rules
            WHERE id = $1
            "#,
            id.0,
        )
        .execute(pool)
        .await
        .wrap_err("deleting moderation flag rule")?;

        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationFlag {
    pub id: ModerationFlagId,
    pub rule_id: Option<ModerationFlagRuleId>,
    pub what: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationFlagRun {
    pub id: ModerationFlagRunId,
    pub status: FlagRunStatus,
    pub started_by: Option<DBUserId>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<FlagRunOutput>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum FlagRunStatus {
    Running,
    Ok,
    Err,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagRunOutput {}
