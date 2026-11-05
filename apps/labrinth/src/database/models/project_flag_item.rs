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

use super::{
    DBProjectId, DBUserId, ProjectFlagId, ProjectFlagRuleId, ProjectFlagRunId,
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
    pub key: String,
    #[serde(default)]
    pub retain_if_removed: bool,
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

#[derive(Debug, Serialize)]
pub struct DBProjectFlagRunResult {
    pub run_id: Option<ProjectFlagRunId>,
    pub project_id: DBProjectId,
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
                AND (flags.active OR (flags.what->>'retain_if_removed')::bool = TRUE)
            WHERE projects.id = $1
            ORDER BY flags.what->>'key', flags.id
            "#,
            project_id.0,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err("fetching project flags")?;

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
    pub rules: BTreeMap<String, ProjectFlagRunRuleResult>,
    pub error: Option<String>,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectFlagRunResults {
    pub results: Vec<ProjectFlagRunResult>,
    pub total: i64,
}

impl DBProjectFlagRunResult {
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
        .wrap_err("counting project flag run results")?;
        let Some(run) = run else {
            return Ok(None);
        };
        let rows = sqlx::query!(
            r#"
            SELECT projects.id, projects.name AS title, projects.slug, projects.icon_url,
                results.rules AS "rules: sqlx::types::Json<RuleResults>",
                results.error, results.evaluated_at
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
        .wrap_err("fetching project flag run results")?;
        let mut rule_ids = rows
            .iter()
            .flat_map(|row| row.rules.0.keys())
            .map(|id| id.parse::<i64>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .wrap_err("parsing project flag result rule IDs")?;
        rule_ids.sort_unstable();
        rule_ids.dedup();
        let names = sqlx::query!(
            r#"SELECT id, name FROM project_flag_rules WHERE id = ANY($1)"#,
            &rule_ids,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err("fetching project flag rule names")?
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
        let results = serde_json::to_value(results)
            .wrap_err("serializing project flag run results")?;
        let mut transaction = pool
            .begin()
            .await
            .wrap_err("starting project flag results transaction")?;
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
        .wrap_err("locking project flag runs")?;
        sqlx::query!(
            r#"
            INSERT INTO project_flag_run_results
                (id, run_id, project_id, rules, error)
            SELECT results.id, results.run_id, results.project_id, results.rules, results.error
            FROM jsonb_to_recordset($1) AS results(
                id UUID, run_id UUID, project_id BIGINT, rules JSONB, error TEXT
            )
            LEFT JOIN project_flag_runs AS runs ON runs.id = results.run_id
            WHERE results.run_id IS NULL OR runs.status = 'running'
            ORDER BY results.run_id, results.project_id
            ON CONFLICT (run_id, project_id) DO UPDATE
            SET rules = EXCLUDED.rules,
                error = EXCLUDED.error, evaluated_at = CURRENT_TIMESTAMP
            "#,
            results,
        )
        .execute(&mut transaction)
        .await
        .wrap_err("persisting project flag run results")?;
        sqlx::query!(
            r#"
            UPDATE project_flag_runs AS runs
            SET status = 'done', completed_at = CURRENT_TIMESTAMP
            WHERE runs.id = ANY($1)
                AND runs.status = 'running'
                AND (
                    SELECT COUNT(*)
                    FROM project_flag_run_results AS results
                    WHERE results.run_id = runs.id
                ) >= runs.total_projects
            "#,
            &run_ids,
        )
        .execute(&mut transaction)
        .await
        .wrap_err("completing project flag runs")?;
        transaction
            .commit()
            .await
            .wrap_err("committing project flag run results")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBProjectFlagRun {
    pub id: ProjectFlagRunId,
    pub status: ProjectFlagRunStatus,
    pub total_projects: i64,
    pub num_errors: i64,
    pub started_by: Option<DBUserId>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub output: Option<ProjectFlagRunOutput>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyRunOutcome {
    Applied(DateTime<Utc>),
    NotFound,
    NotDone,
    AlreadyApplied,
    Incomplete,
}

impl DBProjectFlagRun {
    pub async fn delete(id: ProjectFlagRunId, pool: &PgPool) -> Result<bool> {
        let mut transaction = pool
            .begin()
            .await
            .wrap_err("starting project flag run deletion")?;
        let run = sqlx::query_scalar!(
            r#"SELECT id FROM project_flag_runs WHERE id = $1 FOR UPDATE"#,
            id.0,
        )
        .fetch_optional(&mut transaction)
        .await
        .wrap_err("locking project flag run for deletion")?;
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
        .wrap_err("locking projects for run deletion")?;
        sqlx::query!(
            r#"UPDATE project_flags SET run_id = NULL WHERE run_id = $1 AND active"#,
            id.0,
        ).execute(&mut transaction).await.wrap_err("detaching active project flags")?;
        let result = sqlx::query!(
            r#"
            DELETE FROM project_flag_runs
            WHERE id = $1
            "#,
            id.0,
        )
        .execute(&mut transaction)
        .await
        .wrap_err("deleting project flag run")?;
        transaction
            .commit()
            .await
            .wrap_err("committing project flag run deletion")?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn apply(
        id: ProjectFlagRunId,
        pool: &PgPool,
    ) -> Result<ApplyRunOutcome> {
        let mut transaction = pool
            .begin()
            .await
            .wrap_err("starting project flag apply transaction")?;
        let run = sqlx::query!(
            r#"
            SELECT status AS "status: ProjectFlagRunStatus", applied_at,
                total_projects, error
            FROM project_flag_runs
            WHERE id = $1
            FOR UPDATE
            "#,
            id.0,
        )
        .fetch_optional(&mut transaction)
        .await
        .wrap_err("locking project flag run for application")?;
        let Some(run) = run else {
            return Ok(ApplyRunOutcome::NotFound);
        };
        if run.applied_at.is_some() {
            return Ok(ApplyRunOutcome::AlreadyApplied);
        }
        if run.status != ProjectFlagRunStatus::Done {
            return Ok(ApplyRunOutcome::NotDone);
        }
        let results = sqlx::query!(
            r#"
            SELECT project_id, rules AS "rules: sqlx::types::Json<RuleResults>", error
            FROM project_flag_run_results
            WHERE run_id = $1
            ORDER BY project_id
            "#,
            id.0,
        )
        .fetch_all(&mut transaction)
        .await
        .wrap_err("loading project flag results for application")?;
        if run.error.is_some() || results.len() as i64 != run.total_projects {
            return Ok(ApplyRunOutcome::Incomplete);
        }
        let results = results
            .into_iter()
            .filter(|result| {
                result.error.is_none()
                    && result.rules.0.values().all(|rule| rule.error.is_none())
            })
            .collect::<Vec<_>>();
        let project_ids = results
            .iter()
            .map(|result| result.project_id)
            .collect::<Vec<_>>();
        sqlx::query!(
            r#"
            SELECT id FROM mods
            WHERE id = ANY($1)
            ORDER BY id
            FOR UPDATE
            "#,
            &project_ids,
        )
        .fetch_all(&mut transaction)
        .await
        .wrap_err("locking projects for project flag application")?;
        let mut flags = Vec::new();
        for result in results {
            let mut unique_flags = BTreeMap::<String, ProjectFlag>::new();
            for rule in result.rules.0.into_values() {
                for flag in rule.flags {
                    match unique_flags.entry(flag.key.clone()) {
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(flag);
                        }
                        std::collections::btree_map::Entry::Occupied(
                            mut entry,
                        ) => {
                            entry.get_mut().retain_if_removed |=
                                flag.retain_if_removed;
                        }
                    }
                }
            }
            for what in unique_flags.into_values() {
                flags.push(DBProjectFlag {
                    id: ProjectFlagId(random_base62(8) as i64),
                    run_id: Some(id),
                    project_id: DBProjectId(result.project_id),
                    what,
                    active: true,
                });
            }
        }
        let flags = serde_json::to_value(flags)
            .wrap_err("serializing applied project flags")?;
        sqlx::query!(
            r#"
            UPDATE project_flags SET active = FALSE
            WHERE project_id = ANY($1)
            "#,
            &project_ids,
        )
        .execute(&mut transaction)
        .await
        .wrap_err("deactivating previous project flags")?;
        sqlx::query!(
            r#"
            INSERT INTO project_flags (id, run_id, project_id, what, active)
            SELECT flags.id, flags.run_id, flags.project_id, flags.what, TRUE
            FROM jsonb_to_recordset($1) AS flags(
                id BIGINT, run_id UUID, project_id BIGINT, what JSONB
            )
            ORDER BY flags.project_id, flags.what->>'key'
            ON CONFLICT (project_id, (what->>'key')) DO UPDATE
            SET active = TRUE,
                run_id = EXCLUDED.run_id,
                what = EXCLUDED.what || jsonb_build_object(
                    'retain_if_removed',
                    COALESCE((project_flags.what->>'retain_if_removed')::bool, FALSE)
                    OR COALESCE((EXCLUDED.what->>'retain_if_removed')::bool, FALSE)
                )
            "#,
            flags,
        )
        .execute(&mut transaction)
        .await
        .wrap_err("upserting applied project flags")?;
        let applied_at = sqlx::query_scalar!(
            r#"
            UPDATE project_flag_runs SET applied_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING applied_at AS "applied_at!"
            "#,
            id.0,
        )
        .fetch_one(&mut transaction)
        .await
        .wrap_err("marking project flag run applied")?;
        transaction
            .commit()
            .await
            .wrap_err("committing project flag application")?;
        Ok(ApplyRunOutcome::Applied(applied_at))
    }

    pub async fn queued_project_ids(pool: &PgPool) -> Result<Vec<DBProjectId>> {
        let ids = sqlx::query_scalar!(
            r#"SELECT id FROM mods WHERE status = 'processing'"#,
        )
        .fetch_all(pool)
        .await
        .wrap_err("fetching queued project IDs")?;
        Ok(ids.into_iter().map(DBProjectId).collect())
    }

    pub async fn create(
        user_id: DBUserId,
        total_projects: i64,
        pool: &PgPool,
    ) -> Result<Self> {
        let run = Self {
            id: ProjectFlagRunId(uuid::Uuid::now_v7()),
            status: ProjectFlagRunStatus::Running,
            total_projects,
            num_errors: 0,
            started_by: Some(user_id),
            started_at: Utc::now(),
            completed_at: None,
            applied_at: None,
            output: None,
            error: None,
        };
        sqlx::query!(
            r#"
            INSERT INTO project_flag_runs (id, status, started_by, started_at, total_projects)
            VALUES ($1, 'running', $2, $3, $4)
            "#,
            run.id.0,
            user_id.0,
            run.started_at,
            total_projects,
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
        let status = ProjectFlagRunStatus::Done;
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
            SELECT id, status AS "status: ProjectFlagRunStatus", total_projects, started_by,
                started_at, completed_at, applied_at,
                output AS "output: sqlx::types::Json<ProjectFlagRunOutput>", error,
                (SELECT COUNT(*) FROM project_flag_run_results AS results
                    WHERE results.run_id = project_flag_runs.id AND results.error IS NOT NULL
                ) AS "num_errors!"
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
                total_projects: row.total_projects,
                num_errors: row.num_errors,
                started_by: row.started_by.map(DBUserId),
                started_at: row.started_at,
                completed_at: row.completed_at,
                applied_at: row.applied_at,
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
