use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result};
use ariadne::ids::random_base62;
use chrono::{DateTime, Utc};

use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::projects::ProjectReviewOutcome;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use utoipa::ToSchema;
use validator::Validate;

use super::{
    DBProjectFlagRuleId, DBProjectId, DBUserId, ProjectFlagId, ProjectFlagRunId,
};

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
    #[serde(default)]
    pub decision_questions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBProjectFlagModule {
    pub name: String,
    pub script: String,
}

impl DBProjectFlagModule {
    pub async fn list(
        limit: i64,
        offset: i64,
        pool: &ReadOnlyPgPool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"SELECT name, script FROM project_flag_modules ORDER BY name LIMIT $1 OFFSET $2"#,
            limit,
            offset,
        )
        .fetch_all(&**pool)
        .await
        .context("listing project flag modules")?;
        Ok(rows
            .into_iter()
            .map(|row| Self {
                name: row.name,
                script: row.script,
            })
            .collect())
    }

    pub async fn get(
        name: &str,
        pool: &ReadOnlyPgPool,
    ) -> Result<Option<Self>> {
        let row = sqlx::query!(
            r#"SELECT name, script FROM project_flag_modules WHERE name = $1"#,
            name,
        )
        .fetch_optional(&**pool)
        .await
        .context("fetching project flag module")?;
        Ok(row.map(|row| Self {
            name: row.name,
            script: row.script,
        }))
    }

    pub async fn create(
        name: &str,
        script: &str,
        pool: &PgPool,
    ) -> Result<Self> {
        let row = sqlx::query!(
            r#"INSERT INTO project_flag_modules (name, script) VALUES ($1, $2) RETURNING name, script"#,
            name,
            script,
        )
        .fetch_one(pool)
        .await
        .context("creating project flag module")?;
        Ok(Self {
            name: row.name,
            script: row.script,
        })
    }

    pub async fn update(
        current_name: &str,
        name: &str,
        script: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        let row = sqlx::query!(
            r#"UPDATE project_flag_modules SET name = $2, script = $3 WHERE name = $1 RETURNING name, script"#,
            current_name,
            name,
            script,
        )
        .fetch_optional(pool)
        .await
        .context("updating project flag module")?;
        Ok(row.map(|row| Self {
            name: row.name,
            script: row.script,
        }))
    }

    pub async fn delete(name: &str, pool: &PgPool) -> Result<bool> {
        let result = sqlx::query!(
            r#"DELETE FROM project_flag_modules WHERE name = $1"#,
            name,
        )
        .execute(pool)
        .await
        .context("deleting project flag module")?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBProjectFlagRule {
    pub id: DBProjectFlagRuleId,
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
        .context("snapshotting project flag rules")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: DBProjectFlagRuleId(row.id),
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
        .context("listing project flag rules")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: DBProjectFlagRuleId(row.id),
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
        id: DBProjectFlagRuleId,
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
        .context("fetching project flag rule")?;

        Ok(row.map(|row| Self {
            id: DBProjectFlagRuleId(row.id),
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
        .context("creating project flag rule")?;

        Ok(Self {
            id: DBProjectFlagRuleId(row.id),
            name: row.name,
            params: row.params.0,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        })
    }

    pub async fn update(
        id: DBProjectFlagRuleId,
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
        .context("updating project flag rule")?;

        Ok(row.map(|row| Self {
            id: DBProjectFlagRuleId(row.id),
            name: row.name,
            params: row.params.0,
            created_by: row.created_by.map(DBUserId),
            created_at: row.created_at,
            updated_by: row.updated_by.map(DBUserId),
            updated_at: row.updated_at,
        }))
    }

    pub async fn delete(
        id: DBProjectFlagRuleId,
        pool: &PgPool,
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM project_flag_rules
            WHERE id = $1
            "#,
            id.0,
        )
        .execute(pool)
        .await
        .context("deleting project flag rule")?;

        Ok(result.rows_affected() > 0)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FlagOutcome {
    Withheld,
    #[default]
    ManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectFlag {
    pub key: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub flag_outcome: FlagOutcome,
    #[serde(default)]
    pub retain_if_inactive: bool,
    #[serde(flatten)]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBProjectFlag {
    pub id: ProjectFlagId,
    pub run_id: Option<ProjectFlagRunId>,
    pub project_id: DBProjectId,
    pub what: ProjectFlag,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectFlagRuleResult {
    pub flags: Vec<ProjectFlag>,
    pub log: Vec<String>,
    pub error: Option<String>,
}

type RuleResults = BTreeMap<String, ProjectFlagRuleResult>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBProjectFlagRunResult {
    pub run_id: Option<ProjectFlagRunId>,
    pub project_id: DBProjectId,
    #[serde(default)]
    pub proposed_review_outcome: Option<ProjectReviewOutcome>,
    #[serde(rename = "rule_results")]
    pub rules: BTreeMap<String, ProjectFlagRuleResult>,
    pub error: Option<String>,
}

impl DBProjectFlag {
    pub async fn get_project_flags(
        project_id: DBProjectId,
        pool: &ReadOnlyPgPool,
    ) -> Result<Option<Vec<ProjectFlag>>> {
        let rows = sqlx::query!(
            r#"
            SELECT flags.what AS "what?: sqlx::types::Json<ProjectFlag>"
            FROM mods AS projects
            LEFT JOIN project_flags AS flags ON flags.project_id = projects.id
                AND (flags.active OR (flags.what->>'retain_if_inactive')::bool = TRUE)
            WHERE projects.id = $1
            ORDER BY flags.what->>'key', flags.id
            "#,
            project_id.0,
        )
        .fetch_all(&**pool)
        .await
        .context("fetching project flags")?;

        if rows.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            rows.into_iter()
                .filter_map(|row| row.what.map(|what| what.0))
                .collect(),
        ))
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectFlagRunResultProject {
    pub id: crate::models::ids::ProjectId,
    pub title: String,
    pub slug: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectFlagRunRuleResult {
    pub name: Option<String>,
    #[serde(flatten)]
    pub result: ProjectFlagRuleResult,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectFlagRunResult {
    pub project: ProjectFlagRunResultProject,
    /// Proposed review outcome at evaluation time; null for manual review, errors, projects not in review, or legacy results.
    pub proposed_review_outcome: Option<ProjectReviewOutcome>,
    pub rules: BTreeMap<String, ProjectFlagRunRuleResult>,
    pub num_flags: i64,
    pub error: Option<String>,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectFlagRunResults {
    pub results: Vec<ProjectFlagRunResult>,
    pub total: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectFlagRunCounts {
    pub num_flags: i64,
    pub num_scanned: i64,
    pub num_flagged: i64,
    pub num_no_flags: i64,
    pub num_errors: i64,
}

impl DBProjectFlagRunResult {
    pub async fn counts(
        run_ids: &[ProjectFlagRunId],
        pool: &PgPool,
    ) -> Result<HashMap<ProjectFlagRunId, ProjectFlagRunCounts>> {
        let ids = run_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        let rows = sqlx::query!(
            r#"
            SELECT run_id AS "run_id!", COUNT(*) AS "num_scanned!",
                COALESCE(SUM(num_flags), 0)::bigint AS "num_flags!",
                COUNT(*) FILTER (WHERE error IS NULL AND num_flags > 0) AS "num_flagged!",
                COUNT(*) FILTER (WHERE error IS NULL AND num_flags = 0) AS "num_no_flags!",
                COUNT(*) FILTER (WHERE error IS NOT NULL) AS "num_errors!"
            FROM project_flag_run_results
            WHERE run_id = ANY($1)
            GROUP BY run_id
            "#,
            &ids,
        )
        .fetch_all(pool)
        .await
        .context("aggregating project flag run results")?;
        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    ProjectFlagRunId(row.run_id),
                    ProjectFlagRunCounts {
                        num_flags: row.num_flags,
                        num_scanned: row.num_scanned,
                        num_flagged: row.num_flagged,
                        num_no_flags: row.num_no_flags,
                        num_errors: row.num_errors,
                    },
                )
            })
            .collect())
    }

    pub async fn list(
        run_id: ProjectFlagRunId,
        limit: i64,
        offset: i64,
        pool: &ReadOnlyPgPool,
    ) -> Result<Option<ProjectFlagRunResults>> {
        let run = sqlx::query!(
            r#"
            SELECT (
                SELECT COUNT(*)
                FROM project_flag_run_results
                WHERE run_id = runs.id
            ) AS "total!"
            FROM project_flag_runs AS runs
            WHERE runs.id = $1
            "#,
            run_id.0,
        )
        .fetch_optional(&**pool)
        .await
        .context("counting project flag run results")?;
        let Some(run) = run else {
            return Ok(None);
        };
        let rows = sqlx::query!(
            r#"
            SELECT projects.id, projects.name AS title, projects.slug, projects.icon_url,
                results.rule_results AS "rules: sqlx::types::Json<RuleResults>",
                results.num_flags, results.error, results.evaluated_at, results.proposed_review_outcome
            FROM project_flag_run_results AS results
            INNER JOIN mods AS projects ON projects.id = results.project_id
            WHERE results.run_id = $1
            ORDER BY results.project_id
            LIMIT $2 OFFSET $3
            "#,
            run_id.0,
            limit,
            offset,
        )
        .fetch_all(&**pool)
        .await
        .context("fetching project flag run results")?;
        let mut rule_ids = rows
            .iter()
            .flat_map(|row| row.rules.0.keys())
            .map(|id| id.parse::<i64>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("parsing project flag result rule IDs")?;
        rule_ids.sort_unstable();
        rule_ids.dedup();
        let names = sqlx::query!(
            r#"SELECT id, name FROM project_flag_rules WHERE id = ANY($1)"#,
            &rule_ids,
        )
        .fetch_all(&**pool)
        .await
        .context("fetching project flag rule names")?
        .into_iter()
        .map(|row| (row.id.to_string(), row.name))
        .collect::<BTreeMap<_, _>>();
        Ok(Some(ProjectFlagRunResults {
            results: rows
                .into_iter()
                .map(|row| ProjectFlagRunResult {
                    project: ProjectFlagRunResultProject {
                        id: DBProjectId(row.id).into(),
                        title: row.title,
                        slug: row.slug,
                        icon_url: row.icon_url,
                    },
                    rules: row
                        .rules
                        .0
                        .into_iter()
                        .map(|(id, result)| {
                            let name = names.get(&id).cloned();
                            (id, ProjectFlagRunRuleResult { name, result })
                        })
                        .collect(),
                    proposed_review_outcome: row.proposed_review_outcome.map(
                        |outcome| ProjectReviewOutcome::from_string(&outcome),
                    ),
                    num_flags: row.num_flags,
                    error: row.error,
                    evaluated_at: row.evaluated_at,
                })
                .collect(),
            total: run.total,
        }))
    }

    pub async fn upsert_many(results: Vec<Self>, pool: &PgPool) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }
        let project_ids = results
            .iter()
            .map(|result| result.project_id.0)
            .collect::<Vec<_>>();
        let mut run_ids = results
            .iter()
            .filter_map(|result| result.run_id.map(|id| id.0))
            .collect::<Vec<_>>();
        run_ids.sort_unstable();
        run_ids.dedup();
        #[derive(Serialize)]
        struct ResultPayload {
            id: uuid::Uuid,
            #[serde(flatten)]
            result: DBProjectFlagRunResult,
        }

        let mut run_results = std::collections::HashMap::new();
        let mut standalone_results = Vec::new();
        for mut result in results {
            if result.error.is_none() {
                let errors = result
                    .rules
                    .iter()
                    .filter_map(|(rule_id, rule)| {
                        rule.error
                            .as_ref()
                            .map(|error| format!("{rule_id}: {error}"))
                    })
                    .collect::<Vec<_>>();
                if !errors.is_empty() {
                    result.error = Some(errors.join("\n"));
                }
            }
            if let Some(run_id) = result.run_id {
                run_results.insert((run_id, result.project_id), result);
            } else {
                standalone_results.push(result);
            }
        }
        let results = run_results
            .into_values()
            .chain(standalone_results)
            .map(|result| ResultPayload {
                id: uuid::Uuid::now_v7(),
                result,
            })
            .collect::<Vec<_>>();
        let mut transaction = pool
            .begin()
            .await
            .context("starting project flag results transaction")?;
        sqlx::query!(
            r#"
            SELECT id
            FROM project_flag_runs
            WHERE id = ANY($1)
            ORDER BY id
            FOR UPDATE
            "#,
            &run_ids,
        )
        .fetch_all(&mut transaction)
        .await
        .context("locking project flag runs")?;
        let surviving_projects = sqlx::query_scalar!(
            r#"
            SELECT id FROM mods
            WHERE id = ANY($1)
            ORDER BY id
            FOR KEY SHARE
            "#,
            &project_ids,
        )
        .fetch_all(&mut transaction)
        .await
        .context("locking projects for project flag results")?;
        let results = results
            .into_iter()
            .filter(|payload| {
                surviving_projects
                    .binary_search(&payload.result.project_id.0)
                    .is_ok()
            })
            .collect::<Vec<_>>();
        let results = serde_json::to_value(results)
            .context("serializing project flag run results")?;
        sqlx::query!(
            r#"
            INSERT INTO project_flag_run_results
                (id, run_id, project_id, rule_results, num_flags, error, proposed_review_outcome)
            SELECT results.id, results.run_id, results.project_id, results.rule_results,
                (SELECT COALESCE(SUM(jsonb_array_length(rule.value->'flags')), 0)::BIGINT
                    FROM jsonb_each(results.rule_results) AS rule),
                results.error, results.proposed_review_outcome
            FROM jsonb_to_recordset($1) AS results(
                id UUID, run_id UUID, project_id BIGINT, rule_results JSONB, error TEXT,
                proposed_review_outcome TEXT
            )
            LEFT JOIN project_flag_runs AS runs ON runs.id = results.run_id
            WHERE results.run_id IS NULL OR runs.status = 'running'
            ORDER BY results.run_id, results.project_id
            ON CONFLICT (run_id, project_id) DO NOTHING
            "#,
            results,
        )
        .execute(&mut transaction)
        .await
        .context("persisting project flag run results")?;
        transaction
            .commit()
            .await
            .context("committing project flag run results")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectFlagRunUser {
    pub id: ariadne::ids::UserId,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBProjectFlagRun {
    pub id: ProjectFlagRunId,
    pub ruleset_id: Option<uuid::Uuid>,
    pub status: ProjectFlagRunStatus,
    pub total_projects: i64,
    pub started_by: Option<ProjectFlagRunUser>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<ProjectFlagRunUser>,
    pub output: Option<ProjectFlagRunOutput>,
    pub error: Option<String>,
}

impl DBProjectFlagRun {
    pub async fn num_scanning(pool: &ReadOnlyPgPool) -> Result<i64> {
        sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(GREATEST(runs.total_projects - (
                SELECT COUNT(*) FROM project_flag_run_results WHERE run_id = runs.id
            ), 0)), 0)::BIGINT AS "num_scanning!"
            FROM project_flag_runs AS runs WHERE runs.status = 'running'
            "#,
        )
        .fetch_one(&**pool)
        .await
        .context("counting unfinished project flag scans")
    }

    pub async fn delete(id: ProjectFlagRunId, pool: &PgPool) -> Result<bool> {
        let mut transaction = pool
            .begin()
            .await
            .context("starting project flag run deletion")?;
        let run = sqlx::query_scalar!(
            r#"SELECT id FROM project_flag_runs WHERE id = $1 FOR UPDATE"#,
            id.0,
        )
        .fetch_optional(&mut transaction)
        .await
        .context("locking project flag run for deletion")?;
        if run.is_none() {
            return Ok(false);
        }
        sqlx::query!(
            r#"
            SELECT id FROM mods
            WHERE id IN (SELECT project_id FROM project_flags WHERE run_id = $1)
            ORDER BY id
            FOR UPDATE
            "#,
            id.0,
        )
        .fetch_all(&mut transaction)
        .await
        .context("locking projects for run deletion")?;
        sqlx::query!(
            r#"UPDATE project_flags SET run_id = NULL WHERE run_id = $1 AND active"#,
            id.0,
        ).execute(&mut transaction).await.context("detaching active project flags")?;
        let result = sqlx::query!(
            r#"
            DELETE FROM project_flag_runs
            WHERE id = $1
            "#,
            id.0,
        )
        .execute(&mut transaction)
        .await
        .context("deleting project flag run")?;
        transaction
            .commit()
            .await
            .context("committing project flag run deletion")?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn queued_project_ids(pool: &PgPool) -> Result<Vec<DBProjectId>> {
        let ids =
            sqlx::query_scalar!(r#"SELECT id FROM mods WHERE in_review"#,)
                .fetch_all(pool)
                .await
                .context("fetching queued project IDs")?;
        Ok(ids.into_iter().map(DBProjectId).collect())
    }

    pub async fn create(
        user_id: DBUserId,
        project_ids: &[DBProjectId],
        pool: &PgPool,
    ) -> Result<Self> {
        let mut project_ids =
            project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        project_ids.sort_unstable();
        project_ids.dedup();
        let total_projects = i64::try_from(project_ids.len())
            .context("counting project flag run jobs")?;
        let username = sqlx::query_scalar!(
            r#"SELECT username FROM users WHERE id = $1"#,
            user_id.0,
        )
        .fetch_one(pool)
        .await
        .context("fetching project flag run starter")?;
        let mut run = Self {
            id: ProjectFlagRunId(uuid::Uuid::now_v7()),
            ruleset_id: None,
            status: ProjectFlagRunStatus::Running,
            total_projects,
            started_by: Some(ProjectFlagRunUser {
                id: user_id.into(),
                username,
            }),
            started_at: Utc::now(),
            completed_at: None,
            applied_at: None,
            applied_by: None,
            output: None,
            error: None,
        };
        let mut transaction = pool
            .begin()
            .await
            .context("starting project flag run creation")?;
        let ruleset_id =
            freeze_project_flag_ruleset(user_id, &mut transaction).await?;
        run.ruleset_id = Some(ruleset_id);
        sqlx::query!(
            r#"
            INSERT INTO project_flag_runs (id, status, started_by, started_at, total_projects, ruleset_id)
            VALUES ($1, 'running', $2, $3, $4, $5)
            "#,
            run.id.0,
            user_id.0,
            run.started_at,
            total_projects,
            ruleset_id,
        )
        .execute(&mut transaction)
        .await
        .context("creating project flag run")?;
        sqlx::query!(
            r#"
            INSERT INTO project_flag_run_snapshots (run_id, project_ids)
            VALUES ($1, $2)
        "#,
            run.id.0,
            &project_ids
        )
        .execute(&mut transaction)
        .await
        .context("persisting project flag run snapshot")?;
        transaction
            .commit()
            .await
            .context("committing project flag run creation")?;
        Ok(run)
    }

    pub async fn list(
        limit: i64,
        offset: i64,
        pool: &ReadOnlyPgPool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT runs.id, runs.ruleset_id, runs.status AS "status: ProjectFlagRunStatus", runs.total_projects,
                starter.id AS "started_by_id?", starter.username AS "started_by_username?",
                applier.id AS "applied_by_id?", applier.username AS "applied_by_username?",
                runs.started_at, runs.completed_at, runs.applied_at,
                runs.output AS "output: sqlx::types::Json<ProjectFlagRunOutput>", runs.error
            FROM project_flag_runs AS runs
            LEFT JOIN users AS starter ON starter.id = runs.started_by
            LEFT JOIN users AS applier ON applier.id = runs.applied_by
            ORDER BY runs.started_at DESC, runs.id DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&**pool)
        .await
        .context("listing project flag runs")?;

        Ok(rows
            .into_iter()
            .map(|row| Self {
                id: ProjectFlagRunId(row.id),
                ruleset_id: row.ruleset_id,
                status: row.status,
                total_projects: row.total_projects,
                started_by: row.started_by_id.zip(row.started_by_username).map(
                    |(id, username)| ProjectFlagRunUser {
                        id: DBUserId(id).into(),
                        username,
                    },
                ),
                started_at: row.started_at,
                completed_at: row.completed_at,
                applied_at: row.applied_at,
                applied_by: row.applied_by_id.zip(row.applied_by_username).map(
                    |(id, username)| ProjectFlagRunUser {
                        id: DBUserId(id).into(),
                        username,
                    },
                ),
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
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectFlagRunOutput {}

pub async fn project_flag_ruleset_snapshot(
    pool: &PgPool,
    ruleset_id: uuid::Uuid,
) -> Result<Option<crate::workflow::project_flag::run::RunSnapshot>> {
    let row = sqlx::query!(
        r#"
        SELECT rules, modules FROM project_flag_rulesets WHERE id = $1
    "#,
        ruleset_id
    )
    .fetch_optional(pool)
    .await
    .context("loading project flag ruleset")?;
    row.map(|row| {
        Ok(crate::workflow::project_flag::run::RunSnapshot {
            rules: serde_json::from_value(row.rules)
                .context("decoding deployed rules")?,
            modules: serde_json::from_value(row.modules)
                .context("decoding deployed modules")?,
        })
    })
    .transpose()
}

async fn freeze_project_flag_ruleset(
    user_id: DBUserId,
    transaction: &mut sqlx_tracing::Transaction<'_, sqlx::Postgres>,
) -> Result<uuid::Uuid> {
    let row = sqlx::query!(r#"
        SELECT
            COALESCE((SELECT jsonb_agg(to_jsonb(rules) ORDER BY rules.id) FROM project_flag_rules AS rules), '[]'::jsonb) AS "rules!",
            COALESCE((SELECT jsonb_object_agg(name, script) FROM project_flag_modules), '{}'::jsonb) AS "modules!"
    "#).fetch_one(&mut *transaction).await.context("freezing staging project flag content")?;
    let snapshot = crate::workflow::project_flag::run::RunSnapshot {
        rules: serde_json::from_value(row.rules.clone())
            .context("decoding staging rules")?,
        modules: serde_json::from_value(row.modules.clone())
            .context("decoding staging modules")?,
    };
    let hash = snapshot.content_hash()?.to_hex().to_string();

    let inserted = sqlx::query_scalar!(r#"
        INSERT INTO project_flag_rulesets (id, content_hash, rules, modules, created_by)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (content_hash) DO NOTHING
        RETURNING id
    "#, uuid::Uuid::now_v7(), hash, row.rules, row.modules, user_id.0)
        .fetch_optional(&mut *transaction).await.context("persisting immutable project flag ruleset")?;
    if let Some(id) = inserted {
        return Ok(id);
    }
    sqlx::query_scalar!(
        r#"SELECT id FROM project_flag_rulesets WHERE content_hash = $1"#,
        hash,
    )
    .fetch_one(&mut *transaction)
    .await
    .context("loading existing immutable project flag ruleset")
}
