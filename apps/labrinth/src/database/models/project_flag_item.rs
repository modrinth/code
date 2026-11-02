use std::collections::BTreeMap;

use ariadne::ids::random_base62;
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};

use crate::database::{PgPool, ReadOnlyPgPool};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use utoipa::ToSchema;
use validator::Validate;

use super::{DBUserId, ProjectFlagId, ProjectFlagRuleId, ProjectFlagRunId};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[serde(transparent)]
#[schema(value_type = String, example = "suspicious_url")]
pub struct PatternName(pub String);

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ProjectRuleParams {
    #[validate(length(min = 1, max = 65535))]
    pub expression: String,
    #[serde(default)]
    #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
    #[schema(value_type = BTreeMap<PatternName, String>)]
    pub scan_files_for: BTreeMap<PatternName, Regex>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBProjectFlagRule {
    pub id: ProjectFlagRuleId,
    pub name: String,
    pub created_by: Option<DBUserId>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<DBUserId>,
    pub updated_at: DateTime<Utc>,
    pub params: ProjectRuleParams,
}

impl DBProjectFlagRule {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, params AS "params: sqlx::types::Json<ProjectRuleParams>",
                created_by, created_at, updated_by, updated_at
            FROM project_flag_rules
            ORDER BY id
            "#,
        )
        .fetch_all(pool)
        .await
        .wrap_err("snapshotting project flag rules")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: ProjectFlagRuleId(row.id),
                name: row.name,
                params: row.params.0,
                created_by: row.created_by.map(DBUserId),
                created_at: row.created_at,
                updated_by: row.updated_by.map(DBUserId),
                updated_at: row.updated_at,
            })
            .collect())
    }

    pub async fn list(
        limit: i64,
        offset: i64,
        pool: &ReadOnlyPgPool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, params AS "params: sqlx::types::Json<ProjectRuleParams>",
                created_by, created_at, updated_by, updated_at
            FROM project_flag_rules
            ORDER BY created_at DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err("listing project flag rules")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: ProjectFlagRuleId(row.id),
                name: row.name,
                params: row.params.0,
                created_by: row.created_by.map(DBUserId),
                created_at: row.created_at,
                updated_by: row.updated_by.map(DBUserId),
                updated_at: row.updated_at,
            })
            .collect())
    }

    pub async fn get(
        id: ProjectFlagRuleId,
        pool: &ReadOnlyPgPool,
    ) -> Result<Option<Self>> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, params AS "params: sqlx::types::Json<ProjectRuleParams>",
                created_by, created_at, updated_by, updated_at
            FROM project_flag_rules
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&**pool)
        .await
        .wrap_err("fetching project flag rule")?;

        Ok(row.map(|row| Self {
            id: ProjectFlagRuleId(row.id),
            name: row.name,
            params: row.params.0,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        }))
    }

    pub async fn create(
        name: &str,
        params: &ProjectRuleParams,
        user_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Self> {
        let id = random_base62(8) as i64;
        let now = Utc::now();
        let row = sqlx::query!(
            r#"
            INSERT INTO project_flag_rules
                (id, name, params, created_by, created_at, updated_by, updated_at)
            VALUES ($1, $2, $3, $4, $5, $4, $5)
            RETURNING id, name, params AS "params: sqlx::types::Json<ProjectRuleParams>",
                created_by, created_at, updated_by, updated_at
            "#,
            id,
            name,
            sqlx::types::Json(params) as _,
            user_id.0,
            now,
        )
        .fetch_one(pool)
        .await
        .wrap_err("creating project flag rule")?;

        Ok(Self {
            id: ProjectFlagRuleId(row.id),
            name: row.name,
            params: row.params.0,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        })
    }

    pub async fn update(
        id: ProjectFlagRuleId,
        name: &str,
        params: &ProjectRuleParams,
        user_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        let now = Utc::now();
        let row = sqlx::query!(
            r#"
            UPDATE project_flag_rules
            SET name = $2, params = $3, updated_by = $4, updated_at = $5
            WHERE id = $1
            RETURNING id, name, params AS "params: sqlx::types::Json<ProjectRuleParams>",
                created_by, created_at, updated_by, updated_at
            "#,
            id.0,
            name,
            sqlx::types::Json(params) as _,
            user_id.0,
            now,
        )
        .fetch_optional(pool)
        .await
        .wrap_err("updating project flag rule")?;

        Ok(row.map(|row| Self {
            id: ProjectFlagRuleId(row.id),
            name: row.name,
            params: row.params.0,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        }))
    }

    pub async fn delete(id: ProjectFlagRuleId, pool: &PgPool) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM project_flag_rules
            WHERE id = $1
            "#,
            id.0,
        )
        .execute(pool)
        .await
        .wrap_err("deleting project flag rule")?;

        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectFlag {
    #[serde(default)]
    pub retain_if_removed: bool,
    #[serde(flatten)]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBProjectFlag {
    pub id: ProjectFlagId,
    pub rule_id: Option<ProjectFlagRuleId>,
    pub what: ProjectFlag,
}

#[derive(Debug, Serialize)]
pub struct DBProjectFlagRunResult {
    pub run_id: ProjectFlagRunId,
    pub project_id: super::DBProjectId,
    pub rule_id: ProjectFlagRuleId,
    pub flags: Vec<ProjectFlag>,
    pub log: Vec<String>,
    pub error: Option<String>,
}

impl DBProjectFlagRunResult {
    pub async fn upsert_many(results: Vec<Self>, pool: &PgPool) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }
        let results = results
            .into_iter()
            .map(|result| {
                ((result.run_id, result.project_id, result.rule_id), result)
            })
            .collect::<std::collections::HashMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>();
        let results = serde_json::to_value(results)
            .wrap_err("serializing project flag run results")?;
        sqlx::query!(
            r#"
            INSERT INTO project_flag_run_results
                (run_id, project_id, rule_id, flags, log, error)
            SELECT run_id, project_id, rule_id, flags, log, error
            FROM jsonb_to_recordset($1) AS results(
                run_id UUID, project_id BIGINT, rule_id BIGINT,
                flags JSONB, log TEXT[], error TEXT
            )
            ORDER BY run_id, project_id, rule_id
            ON CONFLICT (run_id, project_id, rule_id) DO UPDATE
            SET flags = EXCLUDED.flags, log = EXCLUDED.log,
                error = EXCLUDED.error, evaluated_at = CURRENT_TIMESTAMP
            "#,
            results,
        )
        .execute(pool)
        .await
        .wrap_err("persisting project flag run results")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBProjectFlagRun {
    pub id: ProjectFlagRunId,
    pub status: ProjectFlagRunStatus,
    pub started_by: Option<DBUserId>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<ProjectFlagRunOutput>,
    pub error: Option<String>,
}

impl DBProjectFlagRun {
    pub async fn queued_project_ids(
        pool: &PgPool,
    ) -> Result<Vec<super::DBProjectId>> {
        let ids = sqlx::query_scalar!(
            r#"SELECT id FROM mods WHERE status = 'processing'"#,
        )
        .fetch_all(pool)
        .await
        .wrap_err("fetching queued project IDs")?;
        Ok(ids.into_iter().map(super::DBProjectId).collect())
    }

    pub async fn create(user_id: DBUserId, pool: &PgPool) -> Result<Self> {
        let run = Self {
            id: ProjectFlagRunId(uuid::Uuid::now_v7()),
            status: ProjectFlagRunStatus::Running,
            started_by: Some(user_id),
            started_at: Utc::now(),
            completed_at: None,
            output: None,
            error: None,
        };
        sqlx::query!(
            r#"
            INSERT INTO project_flag_runs (id, status, started_by, started_at)
            VALUES ($1, 'running', $2, $3)
            "#,
            run.id.0,
            user_id.0,
            run.started_at,
        )
        .execute(pool)
        .await
        .wrap_err("creating project flag run")?;
        Ok(run)
    }

    pub async fn finish(
        &mut self,
        error: Option<String>,
        pool: &PgPool,
    ) -> Result<()> {
        let status = if error.is_some() {
            ProjectFlagRunStatus::Err
        } else {
            ProjectFlagRunStatus::Ok
        };
        let completed_at = Utc::now();
        sqlx::query!(
            r#"
            UPDATE project_flag_runs
            SET status = $2, completed_at = $3, error = $4
            WHERE id = $1
            "#,
            self.id.0,
            status as ProjectFlagRunStatus,
            completed_at,
            error,
        )
        .execute(pool)
        .await
        .wrap_err("finishing project flag run")?;
        self.status = status;
        self.completed_at = Some(completed_at);
        self.error = error;
        Ok(())
    }

    pub async fn list(
        limit: i64,
        offset: i64,
        pool: &ReadOnlyPgPool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, status AS "status: ProjectFlagRunStatus", started_by,
                started_at, completed_at,
                output AS "output: sqlx::types::Json<ProjectFlagRunOutput>", error
            FROM project_flag_runs
            ORDER BY started_at DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err("listing project flag runs")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: ProjectFlagRunId(row.id),
                status: row.status,
                started_by: row.started_by.map(DBUserId),
                started_at: row.started_at,
                completed_at: row.completed_at,
                output: row.output.map(|output| output.0),
                error: row.error,
            })
            .collect())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    sqlx::Type,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ProjectFlagRunStatus {
    Running,
    Ok,
    Err,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectFlagRunOutput {}
