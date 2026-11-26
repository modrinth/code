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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ContextField {
    Title,
    Summary,
    Description,
    Links,
    Slug,
    License,
    Categories,
    AdditionalCategories,
    GalleryTitles,
    GalleryDescriptions,
    VersionNames,
    VersionNumbers,
    VersionChangelogs,
    Filenames,
    ServerAddresses,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
)]
pub struct DecisionQuestion {
    pub question: String,
    pub include_context: Vec<ContextField>,
}

impl DecisionQuestion {
    pub fn canonicalized(&self) -> Self {
        let mut question = self.clone();
        question.include_context.sort();
        question.include_context.dedup();
        question
    }
}

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
    pub decision_questions: HashMap<String, DecisionQuestion>,
}

impl ProjectRuleParams {
    pub fn validate_decision_questions(&self) -> Result<()> {
        for (name, question) in &self.decision_questions {
            anyhow::ensure!(
                !question.question.trim().is_empty(),
                "decision question `{name}` must not be blank"
            );
            anyhow::ensure!(
                !question.include_context.is_empty(),
                "decision question `{name}` must include at least one context field"
            );
        }
        Ok(())
    }
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
    /// Keeps this violation blocking automatic approval after it stops being emitted.
    pub retain_when_resolved: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBProjectFlagRunResult {
    pub run_id: Option<ProjectFlagRunId>,
    pub project_id: DBProjectId,
    #[serde(default)]
    pub proposed_review_outcome: Option<ProjectReviewOutcome>,
    #[serde(rename = "rule_results")]
    pub rules: BTreeMap<String, ProjectFlagRuleResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PersistedProjectFlag {
    #[serde(flatten)]
    pub flag: ProjectFlag,
    /// False means no longer current; retained flags still count as violations.
    pub active: bool,
}

/// Combines current emissions with retained evidence, keyed by violation identity.
/// Current evidence wins; retention remains sticky across re-emissions.
pub(crate) fn effective_project_flags<'a>(
    current: impl IntoIterator<Item = &'a ProjectFlag>,
    persisted: impl IntoIterator<Item = &'a ProjectFlag>,
) -> Vec<PersistedProjectFlag> {
    let mut effective = BTreeMap::new();
    for flag in persisted {
        if flag.retain_when_resolved {
            effective.insert(
                flag.key.clone(),
                PersistedProjectFlag {
                    flag: flag.clone(),
                    active: false,
                },
            );
        }
    }
    for flag in current {
        let retained = effective
            .get(&flag.key)
            .is_some_and(|previous| previous.flag.retain_when_resolved);
        let mut flag = flag.clone();
        flag.retain_when_resolved |= retained;
        effective.insert(
            flag.key.clone(),
            PersistedProjectFlag { flag, active: true },
        );
    }
    effective.into_values().collect()
}

/// Approval application must still pass the existing project eligibility gates.
pub(crate) fn effective_project_flag_outcome<'a>(
    in_review: bool,
    flags: impl IntoIterator<Item = &'a PersistedProjectFlag>,
    has_error: bool,
) -> Option<ProjectReviewOutcome> {
    if !in_review || has_error {
        return None;
    }
    let mut outcome = Some(ProjectReviewOutcome::Approved);
    for flag in flags {
        match flag.flag.flag_outcome {
            FlagOutcome::Withheld => {
                outcome = Some(ProjectReviewOutcome::Withheld)
            }
            FlagOutcome::ManualReview
                if outcome != Some(ProjectReviewOutcome::Withheld) =>
            {
                outcome = None
            }
            FlagOutcome::ManualReview => {}
        }
    }
    outcome
}

impl DBProjectFlag {
    pub async fn get_project_flags(
        project_id: DBProjectId,
        pool: &ReadOnlyPgPool,
    ) -> Result<Option<Vec<PersistedProjectFlag>>> {
        Self::get_project_flags_with_executor(project_id, &**pool).await
    }

    pub(crate) async fn get_project_flags_with_executor<'e>(
        project_id: DBProjectId,
        executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
    ) -> Result<Option<Vec<PersistedProjectFlag>>> {
        let rows = sqlx::query!(
            r#"
            SELECT CASE WHEN flags.id IS NOT NULL THEN flags.what || jsonb_build_object('active', flags.active) END AS "what?: sqlx::types::Json<PersistedProjectFlag>"
            FROM mods AS projects
            LEFT JOIN project_flags AS flags ON flags.project_id = projects.id
                AND (flags.active OR (flags.what->>'retain_when_resolved')::bool = TRUE)
            WHERE projects.id = $1
            ORDER BY flags.what->>'key', flags.id
            "#,
            project_id.0,
        )
        .fetch_all(executor)
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
    /// Effective violations at evaluation time, including resolved retained evidence.
    pub effective_flags: Vec<PersistedProjectFlag>,
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
    pub num_skipped: i64,
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
                COUNT(*) FILTER (WHERE NOT skipped AND error IS NULL AND num_flags > 0) AS "num_flagged!",
                COUNT(*) FILTER (WHERE NOT skipped AND error IS NULL AND num_flags = 0) AS "num_no_flags!",
                COUNT(*) FILTER (WHERE NOT skipped AND error IS NOT NULL) AS "num_errors!",
                COUNT(*) FILTER (WHERE skipped) AS "num_skipped!"
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
                        num_skipped: row.num_skipped,
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
    ) -> Result<ProjectFlagRunResults> {
        let run = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "total!" FROM project_flag_run_results AS results
            INNER JOIN mods AS projects ON projects.id = results.project_id
            WHERE results.run_id = $1
        "#,
            run_id.0
        )
        .fetch_one(&**pool)
        .await
        .context("counting project flag run results")?;
        let rows = sqlx::query!(
            r#"
            SELECT projects.id, projects.name AS title, projects.slug, projects.icon_url,
                results.rule_results AS "rules: sqlx::types::Json<RuleResults>",
                results.num_flags, results.error, results.evaluated_at, results.proposed_review_outcome,
                results.effective_flags AS "effective_flags: sqlx::types::Json<Vec<PersistedProjectFlag>>"
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
        Ok(ProjectFlagRunResults {
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
                    effective_flags: row.effective_flags.0,
                    num_flags: row.num_flags,
                    error: row.error,
                    evaluated_at: row.evaluated_at,
                })
                .collect(),
            total: run.total,
        })
    }

    pub async fn upsert_many(results: Vec<Self>, pool: &PgPool) -> Result<()> {
        anyhow::ensure!(
            results.iter().all(|r| r.run_id.is_none()),
            "run results require a temporal activity guard"
        );
        Self::upsert_many_guarded(results, pool, None).await
    }

    pub(crate) async fn upsert_many_guarded(
        results: Vec<Self>,
        pool: &PgPool,
        guard: Option<(
            &temporalio_client::Client,
            &temporalio_sdk::activities::ActivityContext,
        )>,
    ) -> Result<()> {
        Self::upsert_many_checked(results, pool, |id| async move {
            let (client, ctx) = guard.context("run results require a temporal activity guard")?;
            if ctx.is_cancelled() {
                return Ok(false);
            }
            Ok(crate::workflow::project_flag::run::describe(client, id).await?
                .is_some_and(|run| run.status == crate::workflow::project_flag::run::RunStatus::Running))
        }).await
    }

    async fn upsert_many_checked<F>(
        results: Vec<Self>,
        pool: &PgPool,
        can_persist: impl Fn(ProjectFlagRunId) -> F + Send + Sync,
    ) -> Result<()>
    where
        F: std::future::Future<Output = Result<bool>> + Send,
    {
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
            effective_flags: Vec<PersistedProjectFlag>,
            skipped: bool,
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
        let results =
            run_results
                .into_values()
                .chain(standalone_results)
                .map(|result| ResultPayload {
                    id: uuid::Uuid::now_v7(),
                    result,
                    effective_flags: Vec::new(),
                    skipped: false,
                });
        let mut transaction = pool
            .begin()
            .await
            .context("starting project flag results transaction")?;
        for id in &run_ids {
            let id = ProjectFlagRunId(*id);
            crate::workflow::project_flag::run::lock_run(&mut transaction, id)
                .await?;
            if !can_persist(id).await? {
                return Ok(());
            }
        }
        let surviving_projects = sqlx::query!(
            r#"
            SELECT id, in_review FROM mods
            WHERE id = ANY($1)
            ORDER BY id
            FOR UPDATE
            "#,
            &project_ids,
        )
        .fetch_all(&mut transaction)
        .await
        .context("locking projects for project flag results")?;
        let retained = sqlx::query!(
            r#"SELECT project_id, what AS "what!: sqlx::types::Json<ProjectFlag>"
            FROM project_flags
            WHERE project_id = ANY($1) AND (what->>'retain_when_resolved')::bool = TRUE
            ORDER BY project_id, what->>'key', id"#,
            &project_ids,
        ).fetch_all(&mut transaction).await.context("loading retained project flag evidence")?;
        let mut retained_by_project: HashMap<i64, Vec<ProjectFlag>> =
            HashMap::new();
        for row in retained {
            retained_by_project
                .entry(row.project_id)
                .or_default()
                .push(row.what.0);
        }
        let results = results
            .filter_map(|mut payload| {
                let index = surviving_projects.binary_search_by_key(
                    &payload.result.project_id.0,
                    |project| project.id,
                );
                let Ok(index) = index else {
                    payload.result.run_id?;
                    payload.skipped = true;
                    payload.result.rules.clear();
                    payload.result.error = None;
                    payload.result.proposed_review_outcome = None;
                    return Some(payload);
                };
                let project = &surviving_projects[index];
                payload.effective_flags = effective_project_flags(
                    payload.result.rules.values().flat_map(|rule| &rule.flags),
                    retained_by_project.get(&project.id).into_iter().flatten(),
                );
                payload.result.proposed_review_outcome =
                    effective_project_flag_outcome(
                        project.in_review,
                        &payload.effective_flags,
                        payload.result.error.is_some(),
                    );
                Some(payload)
            })
            .collect::<Vec<_>>();
        let results = serde_json::to_value(results)
            .context("serializing project flag run results")?;
        sqlx::query!(
            r#"
            INSERT INTO project_flag_run_results
                (id, run_id, project_id, rule_results, num_flags, error, proposed_review_outcome, effective_flags, skipped)
            SELECT results.id, results.run_id, results.project_id, results.rule_results,
                jsonb_array_length(results.effective_flags)::BIGINT,
                results.error, results.proposed_review_outcome, results.effective_flags, results.skipped
            FROM jsonb_to_recordset($1) AS results(
                id UUID, run_id UUID, project_id BIGINT, rule_results JSONB, error TEXT,
                proposed_review_outcome TEXT, effective_flags JSONB, skipped BOOLEAN
            )
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

#[cfg(test)]
mod skipped_result_tests {
    use super::*;
    use crate::test::{
        api_v3::ApiV3,
        environment::{TestEnvironment, with_test_environment},
    };

    #[actix_rt::test]
    async fn deleted_projects_complete_idempotently_without_resurrecting_runs()
    {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let pool = &env.db.pool;
            let existing = sqlx::query_scalar!(r#"SELECT id FROM mods ORDER BY id LIMIT 1"#)
                .fetch_one(pool).await.unwrap();
            for projects in [vec![existing, -1], vec![-1, -2]] {
                let run_id = ProjectFlagRunId(uuid::Uuid::now_v7());
                let results = projects.into_iter().map(|id| DBProjectFlagRunResult {
                    run_id: Some(run_id), project_id: DBProjectId(id),
                    proposed_review_outcome: Some(ProjectReviewOutcome::Approved),
                    rules: BTreeMap::new(), error: None,
                }).collect::<Vec<_>>();
                assert!(DBProjectFlagRunResult::upsert_many(results.clone(), pool).await.is_err());
                let (first, retry) = tokio::join!(
                    DBProjectFlagRunResult::upsert_many_checked(results.clone(), pool, |_| async { Ok(true) }),
                    DBProjectFlagRunResult::upsert_many_checked(results.clone(), pool, |_| async { Ok(true) }),
                );
                first.unwrap(); retry.unwrap();
                let counts = DBProjectFlagRunResult::counts(&[run_id], pool).await.unwrap().remove(&run_id).unwrap();
                assert_eq!(counts.num_scanned, 2);
                let skipped = if results[0].project_id.0 == existing { 1 } else { 2 };
                assert_eq!(counts.num_skipped, skipped);
                assert_eq!(counts.num_no_flags, 2 - skipped);
                assert_eq!(counts.num_errors, 0);
                assert_eq!(counts.num_flagged, 0);
                let tombstones = sqlx::query!(r#"
                    SELECT rule_results, effective_flags, proposed_review_outcome, error, num_flags
                    FROM project_flag_run_results WHERE run_id = $1 AND skipped
                "#, run_id.0).fetch_all(pool).await.unwrap();
                assert_eq!(tombstones.len() as i64, skipped);
                for row in tombstones {
                    assert_eq!(row.rule_results, serde_json::json!({}));
                    assert_eq!(row.effective_flags, serde_json::json!([]));
                    assert!(row.proposed_review_outcome.is_none());
                    assert!(row.error.is_none());
                    assert_eq!(row.num_flags, 0);
                }
                sqlx::query!(r#"DELETE FROM project_flag_run_results WHERE run_id = $1"#, run_id.0)
                    .execute(pool).await.unwrap();
                DBProjectFlagRunResult::upsert_many_checked(results, pool, |_| async { Ok(false) }).await.unwrap();
                assert!(DBProjectFlagRunResult::counts(&[run_id], pool).await.unwrap().is_empty());
            }
        }).await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectFlagRunUser {
    pub id: ariadne::ids::UserId,
    pub username: String,
}

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

pub(crate) async fn freeze_project_flag_ruleset(
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

#[cfg(test)]
mod decision_question_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_named_questions_and_requires_explicit_safe_context() {
        for (question, fields, valid) in [
            (
                "Question?",
                json!(["title", "summary", "description", "links"]),
                true,
            ),
            ("  ", json!(["title"]), false),
            ("Question?", json!([]), false),
        ] {
            let params: ProjectRuleParams = serde_json::from_value(json!({
                "expression": "[]",
                "decision_questions": {"named_answer": {"question": question, "include_context": fields}},
            })).unwrap();
            let result = params.validate_decision_questions();
            assert_eq!(result.is_ok(), valid);
            if let Err(error) = result {
                assert!(error.to_string().contains("named_answer"));
            }
        }
        for value in [
            json!({"question": "Q", "include_context": ["users"]}),
            json!({"question": "Q", "include_context": ["Title"]}),
            json!({"question": "Q"}),
            json!("legacy prompt"),
        ] {
            assert!(serde_json::from_value::<DecisionQuestion>(value).is_err());
        }
    }
}
