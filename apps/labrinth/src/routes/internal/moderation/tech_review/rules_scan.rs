use std::collections::{BTreeMap, HashMap};

use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ariadne::ids::base62_impl::to_base62;
use bytes::Bytes;
use eyre::{Result, eyre};
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use utoipa::{PartialSchema, ToSchema};
use xredis::RedisPool;

use super::rules::{DelphiRuleEffect, DelphiRuleOutput};
use crate::routes::internal::delphi::tech_review_queue::{
    self, TechReviewRemovalReason,
};
use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool, PgTransaction, ReadOnlyPgPool,
        models::{
            DBProjectId, DelphiReportIssueDetailsId, DelphiRuleId,
            delphi_report_item::DelphiSeverity,
        },
    },
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
    util::{cel, error::Context},
};

const RULE_SCAN_LOCK_ID: i64 = 0x6465_6c70_6869_7275;
const PROGRESS_INTERVAL: usize = 50;
pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_rule_schema)
        .service(get_detail_rule_input)
        .service(scan_rules);
}

#[derive(Serialize)]
struct RuleScanEvent<'a> {
    phase: &'a str,
    revision: i64,
    scanned: usize,
    total: usize,
    effects: usize,
}

#[derive(Serialize)]
struct RuleScanErrorEvent<'a> {
    message: &'a str,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleInput {
    pub schema_version: u32,
    pub trace: RuleTrace,
    pub file_traces: Vec<RuleTrace>,
    pub scan: RuleScan,
    pub artifact: RuleArtifact,
    pub project: RuleProject,
    pub version: RuleVersion,
    pub file: RuleFile,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleTrace {
    pub key: String,
    pub issue_type: String,
    pub severity: DelphiSeverity,
    pub jar: Option<String>,
    pub file_path: String,
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleScan {
    pub delphi_version: i32,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleArtifact {
    pub size: Option<i32>,
    pub hashes: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleProject {
    pub id: Option<String>,
    pub types: Vec<String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleVersion {
    pub id: Option<String>,
    pub loaders: Vec<String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleFile {
    pub id: Option<String>,
}

struct CompiledRule {
    id: DelphiRuleId,
    name: String,
    expression: String,
    on_issue_types: Vec<String>,
    program: cel::Program,
}

impl CompiledRule {
    fn applies_to_issue_type(&self, issue_type: &str) -> bool {
        self.on_issue_types.is_empty()
            || self
                .on_issue_types
                .iter()
                .any(|rule_issue_type| rule_issue_type == issue_type)
    }
}

struct MaterializedEffect {
    detail_id: i64,
    rule_id: DelphiRuleId,
    effect: DelphiRuleEffect,
}

struct ScanSummary {
    revision: i64,
    scanned: usize,
    total: usize,
    effects: usize,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct DelphiRuleSchemaResponse {
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub components: BTreeMap<String, serde_json::Value>,
}

/// Get the schemas for the CEL context and output values.
#[utoipa::path(
    context_path = "/moderation/tech-review",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK, body = DelphiRuleSchemaResponse))
)]
#[get("/rules/schema")]
pub async fn get_rule_schema(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DelphiRuleSchemaResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let mut schemas = Vec::new();
    <RuleInput as ToSchema>::schemas(&mut schemas);
    <Option<DelphiRuleOutput> as ToSchema>::schemas(&mut schemas);

    Ok(web::Json(DelphiRuleSchemaResponse {
        input: schema_to_value(<RuleInput as PartialSchema>::schema())?,
        output: schema_to_value(
            <Option<DelphiRuleOutput> as PartialSchema>::schema(),
        )?,
        components: schemas
            .into_iter()
            .map(|(name, schema)| Ok((name, schema_to_value(schema)?)))
            .collect::<Result<_, ApiError>>()?,
    }))
}

/// Get the exact CEL input for a Delphi issue detail.
#[utoipa::path(
    context_path = "/moderation/tech-review",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses(
        (status = OK, body = RuleInput),
        (status = NOT_FOUND, description = "Detail not found")
    )
)]
#[get("/rules/details/{detail_id}/input")]
pub async fn get_detail_rule_input(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(DelphiReportIssueDetailsId,)>,
) -> Result<web::Json<RuleInput>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let (detail_id,) = path.into_inner();
    let detail = sqlx::query!(
                r#"
        SELECT
            detail.key,
            issue.issue_type,
            detail.severity AS "severity: DelphiSeverity",
            detail.jar,
            detail.file_path,
            detail.data AS "data: Json<HashMap<String, serde_json::Value>>",
            COALESCE(file_traces.traces, '[]'::jsonb)
                AS "file_traces!: Json<Vec<RuleTrace>>",
            report.delphi_version,
            file.size AS "size?",
            file.id AS "file_id?",
            version.id AS "version_id?",
            version.mod_id AS "project_id?",
            COALESCE(version_metadata.project_types, ARRAY[]::text[])
                AS "project_types!: Vec<String>",
            COALESCE(version_metadata.loaders, ARRAY[]::text[])
                AS "loaders!: Vec<String>",
            COALESCE(file_hashes.hashes, '{}'::jsonb)
                AS "hashes!: Json<BTreeMap<String, String>>"
        FROM delphi_report_issue_details detail
        INNER JOIN delphi_report_issues issue ON issue.id = detail.issue_id
        INNER JOIN delphi_reports report ON report.id = issue.report_id
        LEFT JOIN LATERAL (
            SELECT
                jsonb_agg(
                    jsonb_build_object(
                        'key', file_detail.key,
                        'issue_type', file_issue.issue_type,
                        'severity', file_detail.severity,
                        'jar', file_detail.jar,
                        'file_path', file_detail.file_path,
                        'data', file_detail.data
                    )
                    ORDER BY file_detail.id
                ) AS traces
            FROM delphi_report_issues file_issue
            INNER JOIN delphi_report_issue_details file_detail
                ON file_detail.issue_id = file_issue.id
            WHERE file_issue.report_id = issue.report_id
                AND file_detail.file_path = detail.file_path
        ) file_traces ON TRUE
        LEFT JOIN files file ON file.id = report.file_id
        LEFT JOIN versions version ON version.id = file.version_id
        LEFT JOIN LATERAL (
            SELECT
                ARRAY_AGG(
                    DISTINCT project_type.name::text
                    ORDER BY project_type.name::text
                ) FILTER (WHERE project_type.name IS NOT NULL)
                    AS project_types,
                ARRAY_AGG(
                    DISTINCT loader.loader
                    ORDER BY loader.loader
                ) AS loaders
            FROM loaders_versions loader_version
            INNER JOIN loaders loader
                ON loader.id = loader_version.loader_id
            LEFT JOIN loaders_project_types loader_project_type
                ON loader_project_type.joining_loader_id = loader_version.loader_id
            LEFT JOIN project_types project_type
                ON project_type.id = loader_project_type.joining_project_type_id
            WHERE loader_version.version_id = version.id
        ) version_metadata ON TRUE
        LEFT JOIN LATERAL (
            SELECT
                jsonb_object_agg(algorithm, encode(hash, 'hex')) AS hashes
            FROM hashes
            WHERE hashes.file_id = file.id
        ) file_hashes ON TRUE
        WHERE detail.id = $1
        "#,
        detail_id as DelphiReportIssueDetailsId,
    )
    .fetch_optional(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch delphi rule input")?
    .wrap_not_found_err("delphi rule input not found")?;

    Ok(web::Json(RuleInput {
        schema_version: 1,
        trace: RuleTrace {
            key: detail.key,
            issue_type: detail.issue_type,
            severity: detail.severity,
            jar: detail.jar,
            file_path: detail.file_path,
            data: detail.data.0,
        },
        file_traces: detail.file_traces.0,
        scan: RuleScan {
            delphi_version: detail.delphi_version,
        },
        artifact: RuleArtifact {
            size: detail.size,
            hashes: detail.hashes.0,
        },
        project: RuleProject {
            id: detail.project_id.map(|id| to_base62(id as u64)),
            types: detail.project_types,
        },
        version: RuleVersion {
            id: detail.version_id.map(|id| to_base62(id as u64)),
            loaders: detail.loaders,
        },
        file: RuleFile {
            id: detail.file_id.map(|id| to_base62(id as u64)),
        },
    }))
}

fn schema_to_value<T: Serialize>(
    schema: T,
) -> Result<serde_json::Value, ApiError> {
    serde_json::to_value(schema).map_err(|error| {
        ApiError::Internal(
            eyre!(error).wrap_err("failed to serialize Delphi rule schema"),
        )
    })
}

/// Re-evaluate every Delphi issue detail and atomically publish a new rule revision.
#[utoipa::path(
    context_path = "/moderation/tech-review",
    tag = "moderation",
    security(("bearer_auth" = [])),
    responses((status = OK), (status = CONFLICT))
)]
#[post("/rules/scan")]
pub async fn scan_rules(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin delphi rule scan")?;

    sqlx::query!("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut transaction)
        .await
        .map_err(|error| {
            ApiError::Internal(
                eyre!(error)
                    .wrap_err("failed to set delphi rule scan isolation"),
            )
        })?;

    let acquired = sqlx::query_scalar!(
        "SELECT pg_try_advisory_xact_lock($1)",
        RULE_SCAN_LOCK_ID,
    )
    .fetch_one(&mut transaction)
    .await
    .map_err(|error| {
        ApiError::Internal(
            eyre!(error).wrap_err("failed to acquire delphi rule scan lock"),
        )
    })?
    .unwrap_or(false);

    if !acquired {
        return Err(ApiError::Conflict(eyre!(
            "a delphi rule scan is already running"
        )));
    }

    let (sender, receiver) = mpsc::unbounded_channel();
    actix_web::rt::spawn(async move {
        match run_scan(transaction, &sender).await {
            Ok(summary) => {
                let event = RuleScanEvent {
                    phase: "complete",
                    revision: summary.revision,
                    scanned: summary.scanned,
                    total: summary.total,
                    effects: summary.effects,
                };
                if let Ok(data) = serde_json::to_string(&event) {
                    let _ = sender.send(Bytes::from(format!(
                        "event: complete\ndata: {data}\n\n"
                    )));
                }
            }
            Err(error) => {
                tracing::error!(error = ?error, "delphi rule scan failed");
                let message = format!("{error:#}");
                let event = RuleScanErrorEvent { message: &message };
                if let Ok(data) = serde_json::to_string(&event) {
                    let _ = sender.send(Bytes::from(format!(
                        "event: failed\ndata: {data}\n\n"
                    )));
                }
            }
        }
    });

    let stream =
        UnboundedReceiverStream::new(receiver).map(Ok::<_, std::io::Error>);

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream))
}

async fn run_scan(
    mut transaction: PgTransaction<'static>,
    sender: &mpsc::UnboundedSender<Bytes>,
) -> Result<ScanSummary> {
    sqlx::query!("LOCK TABLE delphi_rules IN SHARE MODE")
        .execute(&mut transaction)
        .await
        .wrap_err("failed to lock delphi rules")?;
    sqlx::query!("LOCK TABLE delphi_report_issue_details IN SHARE MODE")
        .execute(&mut transaction)
        .await
        .wrap_err("failed to lock delphi issue details")?;

    let current_revision = sqlx::query_scalar!(
        "SELECT revision FROM delphi_rule_revisions LIMIT 1 FOR UPDATE",
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_err("failed to fetch the current delphi rule revision")?;
    let revision = current_revision
        .checked_add(1)
        .ok_or_else(|| eyre!("delphi rule revision overflowed"))?;

    let rules = fetch_compiled_rules(&mut transaction).await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM delphi_report_issue_details
        "#,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_err("failed to count delphi issue details")? as usize;

    let mut details = sqlx::query!(
        r#"
        SELECT
            detail.id,
            detail.key,
            issue.issue_type,
            detail.severity AS "severity: DelphiSeverity",
            detail.jar,
            detail.file_path,
            detail.data AS "data: Json<HashMap<String, serde_json::Value>>",
            COALESCE(file_traces.traces, '[]'::jsonb)
                AS "file_traces!: Json<Vec<RuleTrace>>",
            report.delphi_version,
            file.size AS "size?",
            file.id AS "file_id?",
            version.id AS "version_id?",
            version.mod_id AS "project_id?",
            COALESCE(version_metadata.project_types, ARRAY[]::text[])
                AS "project_types!: Vec<String>",
            COALESCE(version_metadata.loaders, ARRAY[]::text[])
                AS "loaders!: Vec<String>",
            COALESCE(file_hashes.hashes, '{}'::jsonb)
                AS "hashes!: Json<BTreeMap<String, String>>"
        FROM delphi_report_issue_details detail
        INNER JOIN delphi_report_issues issue ON issue.id = detail.issue_id
        INNER JOIN delphi_reports report ON report.id = issue.report_id
        LEFT JOIN LATERAL (
            SELECT
                jsonb_agg(
                    jsonb_build_object(
                        'key', file_detail.key,
                        'issue_type', file_issue.issue_type,
                        'severity', file_detail.severity,
                        'jar', file_detail.jar,
                        'file_path', file_detail.file_path,
                        'data', file_detail.data
                    )
                    ORDER BY file_detail.id
                ) AS traces
            FROM delphi_report_issues file_issue
            INNER JOIN delphi_report_issue_details file_detail
                ON file_detail.issue_id = file_issue.id
            WHERE file_issue.report_id = issue.report_id
                AND file_detail.file_path = detail.file_path
        ) file_traces ON TRUE
        LEFT JOIN files file ON file.id = report.file_id
        LEFT JOIN versions version ON version.id = file.version_id
        LEFT JOIN (
            SELECT
                loader_version.version_id,
                ARRAY_AGG(
                    DISTINCT project_type.name::text
                    ORDER BY project_type.name::text
                ) FILTER (WHERE project_type.name IS NOT NULL)
                    AS project_types,
                ARRAY_AGG(
                    DISTINCT loader.loader
                    ORDER BY loader.loader
                ) AS loaders
            FROM loaders_versions loader_version
            INNER JOIN loaders loader
                ON loader.id = loader_version.loader_id
            LEFT JOIN loaders_project_types loader_project_type
                ON loader_project_type.joining_loader_id = loader_version.loader_id
            LEFT JOIN project_types project_type
                ON project_type.id = loader_project_type.joining_project_type_id
            GROUP BY loader_version.version_id
        ) version_metadata
            ON version_metadata.version_id = version.id
        LEFT JOIN (
            SELECT
                file_id,
                jsonb_object_agg(algorithm, encode(hash, 'hex')) AS hashes
            FROM hashes
            GROUP BY file_id
        ) file_hashes ON file_hashes.file_id = file.id
        ORDER BY detail.id
        "#,
    )
    .fetch(&mut transaction);

    let mut effects = Vec::new();
    let mut scanned = 0;
    let event = RuleScanEvent {
        phase: "scanning",
        revision,
        scanned: 0,
        total,
        effects: 0,
    };
    if let Ok(data) = serde_json::to_string(&event) {
        let _ = sender
            .send(Bytes::from(format!("event: progress\ndata: {data}\n\n")));
    }

    while let Some(detail) = details
        .try_next()
        .await
        .wrap_err("failed to fetch a delphi issue detail")?
    {
        let detail_id = detail.id;
        let input = RuleInput {
            schema_version: 1,
            trace: RuleTrace {
                key: detail.key,
                issue_type: detail.issue_type,
                severity: detail.severity,
                jar: detail.jar,
                file_path: detail.file_path,
                data: detail.data.0,
            },
            file_traces: detail.file_traces.0,
            scan: RuleScan {
                delphi_version: detail.delphi_version,
            },
            artifact: RuleArtifact {
                size: detail.size,
                hashes: detail.hashes.0,
            },
            project: RuleProject {
                id: detail.project_id.map(|id| to_base62(id as u64)),
                types: detail.project_types,
            },
            version: RuleVersion {
                id: detail.version_id.map(|id| to_base62(id as u64)),
                loaders: detail.loaders,
            },
            file: RuleFile {
                id: detail.file_id.map(|id| to_base62(id as u64)),
            },
        };

        for rule in &rules {
            if !rule.applies_to_issue_type(&input.trace.issue_type) {
                continue;
            }

            let effect = evaluate_rule(
                &rule.program,
                &rule.expression,
                &input,
            )
                .wrap_err_with(|| {
                    format!(
                        "failed to evaluate delphi rule '{}' for detail {detail_id}",
                        rule.name
                    )
                })?;
            if let Some(effect) = effect {
                effects.push(MaterializedEffect {
                    detail_id,
                    rule_id: rule.id,
                    effect,
                });
                break;
            }
        }

        scanned += 1;
        if scanned % PROGRESS_INTERVAL == 0 || scanned == total {
            let event = RuleScanEvent {
                phase: "scanning",
                revision,
                scanned,
                total,
                effects: effects.len(),
            };
            if let Ok(data) = serde_json::to_string(&event) {
                let _ = sender.send(Bytes::from(format!(
                    "event: progress\ndata: {data}\n\n"
                )));
            }
            tokio::task::yield_now().await;
        }
    }
    drop(details);

    let event = RuleScanEvent {
        phase: "publishing",
        revision,
        scanned: total,
        total,
        effects: effects.len(),
    };
    if let Ok(data) = serde_json::to_string(&event) {
        let _ = sender
            .send(Bytes::from(format!("event: progress\ndata: {data}\n\n")));
    }

    insert_materialized_effects(revision, &effects, &mut transaction).await?;

    let affected_projects = sqlx::query!(
        r#"
        WITH project_membership AS (
            SELECT
                detail.project_id,
                BOOL_OR(
                    detail.status IN ('pending', 'unsafe')
                    AND detail.severity != 'hidden'
                ) AS old_needs_review,
                BOOL_OR(
                    detail.status IN ('pending', 'unsafe')
                    AND COALESCE(
                        new_effect.severity,
                        detail.original_severity
                    ) != 'hidden'
                ) AS new_needs_review
            FROM delphi_issue_details_with_statuses detail
            LEFT JOIN delphi_rule_effects new_effect
                ON new_effect.revision = $1
                AND new_effect.detail_id = detail.id
            GROUP BY detail.project_id
        )
        SELECT
            project_id AS "project_id!: DBProjectId",
            new_needs_review AS "new_needs_review!"
        FROM project_membership
        WHERE old_needs_review IS DISTINCT FROM new_needs_review
        "#,
        revision,
    )
    .fetch_all(&mut transaction)
    .await
    .wrap_err("failed to fetch projects affected by delphi rule changes")?;

    sqlx::query!(
        "UPDATE delphi_rules SET revision = $1 WHERE NOT delete_on_next_revision",
        revision,
    )
    .execute(&mut transaction)
    .await
    .wrap_err("failed to update delphi rule revisions")?;
    sqlx::query!("UPDATE delphi_rule_revisions SET revision = $1", revision)
        .execute(&mut transaction)
        .await
        .wrap_err("failed to publish the delphi rule revision")?;

    tech_review_queue::add_projects(
        &affected_projects
            .iter()
            .filter(|project| project.new_needs_review)
            .map(|project| project.project_id)
            .collect::<Vec<_>>(),
        &mut transaction,
    )
    .await?;
    tech_review_queue::remove_projects(
        &affected_projects
            .iter()
            .filter(|project| !project.new_needs_review)
            .map(|project| project.project_id)
            .collect::<Vec<_>>(),
        TechReviewRemovalReason::RulesChanged,
        &mut transaction,
    )
    .await?;

    sqlx::query!(
        "DELETE FROM delphi_rule_effects WHERE revision <> $1",
        revision,
    )
    .execute(&mut transaction)
    .await
    .wrap_err("failed to delete old delphi rule effects")?;
    sqlx::query!("DELETE FROM delphi_rules WHERE delete_on_next_revision")
        .execute(&mut transaction)
        .await
        .wrap_err("failed to delete retired delphi rules")?;

    transaction
        .commit()
        .await
        .wrap_err("failed to commit the delphi rule scan")?;

    Ok(ScanSummary {
        revision,
        scanned: total,
        total,
        effects: effects.len(),
    })
}

pub(crate) async fn materialize_current_rule_effects(
    detail_ids: &[DelphiReportIssueDetailsId],
    transaction: &mut PgTransaction<'_>,
) -> Result<()> {
    if detail_ids.is_empty() {
        return Ok(());
    }

    let revision = sqlx::query_scalar!(
        "SELECT revision FROM delphi_rule_revisions LIMIT 1",
    )
    .fetch_one(&mut *transaction)
    .await
    .wrap_err("failed to fetch the current delphi rule revision")?;
    let rules = fetch_compiled_rules(transaction).await?;

    if rules.is_empty() {
        return Ok(());
    }

    let details = sqlx::query!(
        r#"
        SELECT
            detail.id,
            detail.key,
            issue.issue_type,
            detail.severity AS "severity: DelphiSeverity",
            detail.jar,
            detail.file_path,
            detail.data AS "data: Json<HashMap<String, serde_json::Value>>",
            COALESCE(file_traces.traces, '[]'::jsonb)
                AS "file_traces!: Json<Vec<RuleTrace>>",
            report.delphi_version,
            file.size AS "size?",
            file.id AS "file_id?",
            version.id AS "version_id?",
            version.mod_id AS "project_id?",
            COALESCE(version_metadata.project_types, ARRAY[]::text[])
                AS "project_types!: Vec<String>",
            COALESCE(version_metadata.loaders, ARRAY[]::text[])
                AS "loaders!: Vec<String>",
            COALESCE(file_hashes.hashes, '{}'::jsonb)
                AS "hashes!: Json<BTreeMap<String, String>>"
        FROM delphi_report_issue_details detail
        INNER JOIN delphi_report_issues issue ON issue.id = detail.issue_id
        INNER JOIN delphi_reports report ON report.id = issue.report_id
        LEFT JOIN LATERAL (
            SELECT
                jsonb_agg(
                    jsonb_build_object(
                        'key', file_detail.key,
                        'issue_type', file_issue.issue_type,
                        'severity', file_detail.severity,
                        'jar', file_detail.jar,
                        'file_path', file_detail.file_path,
                        'data', file_detail.data
                    )
                    ORDER BY file_detail.id
                ) AS traces
            FROM delphi_report_issues file_issue
            INNER JOIN delphi_report_issue_details file_detail
                ON file_detail.issue_id = file_issue.id
            WHERE file_issue.report_id = issue.report_id
                AND file_detail.file_path = detail.file_path
        ) file_traces ON TRUE
        LEFT JOIN files file ON file.id = report.file_id
        LEFT JOIN versions version ON version.id = file.version_id
        LEFT JOIN LATERAL (
            SELECT
                ARRAY_AGG(
                    DISTINCT project_type.name::text
                    ORDER BY project_type.name::text
                ) FILTER (WHERE project_type.name IS NOT NULL)
                    AS project_types,
                ARRAY_AGG(
                    DISTINCT loader.loader
                    ORDER BY loader.loader
                ) AS loaders
            FROM loaders_versions loader_version
            INNER JOIN loaders loader
                ON loader.id = loader_version.loader_id
            LEFT JOIN loaders_project_types loader_project_type
                ON loader_project_type.joining_loader_id = loader_version.loader_id
            LEFT JOIN project_types project_type
                ON project_type.id = loader_project_type.joining_project_type_id
            WHERE loader_version.version_id = version.id
        ) version_metadata ON TRUE
        LEFT JOIN LATERAL (
            SELECT
                jsonb_object_agg(algorithm, encode(hash, 'hex')) AS hashes
            FROM hashes
            WHERE hashes.file_id = file.id
        ) file_hashes ON TRUE
        WHERE detail.id = ANY($1::bigint[])
        ORDER BY detail.id
        "#,
        &detail_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
    )
    .fetch_all(&mut *transaction)
    .await
    .wrap_err("failed to fetch new delphi issue details")?;

    let mut effects = Vec::new();
    for detail in details {
        let input = RuleInput {
            schema_version: 1,
            trace: RuleTrace {
                key: detail.key,
                issue_type: detail.issue_type,
                severity: detail.severity,
                jar: detail.jar,
                file_path: detail.file_path,
                data: detail.data.0,
            },
            file_traces: detail.file_traces.0,
            scan: RuleScan {
                delphi_version: detail.delphi_version,
            },
            artifact: RuleArtifact {
                size: detail.size,
                hashes: detail.hashes.0,
            },
            project: RuleProject {
                id: detail.project_id.map(|id| to_base62(id as u64)),
                types: detail.project_types,
            },
            version: RuleVersion {
                id: detail.version_id.map(|id| to_base62(id as u64)),
                loaders: detail.loaders,
            },
            file: RuleFile {
                id: detail.file_id.map(|id| to_base62(id as u64)),
            },
        };

        for rule in &rules {
            if !rule.applies_to_issue_type(&input.trace.issue_type) {
                continue;
            }

            let effect = evaluate_rule(&rule.program, &rule.expression, &input)
                .wrap_err_with(|| {
                    format!(
                        "failed to evaluate delphi rule '{}' for detail {}",
                        rule.name, detail.id
                    )
                })?;
            if let Some(effect) = effect {
                effects.push(MaterializedEffect {
                    detail_id: detail.id,
                    rule_id: rule.id,
                    effect,
                });
                break;
            }
        }
    }

    insert_materialized_effects(revision, &effects, transaction).await
}

async fn fetch_compiled_rules(
    transaction: &mut PgTransaction<'_>,
) -> Result<Vec<CompiledRule>> {
    let rules = sqlx::query!(
        r#"
        SELECT id AS "id!: DelphiRuleId", name, rule, on_issue_types
        FROM delphi_rules
        WHERE NOT delete_on_next_revision
        ORDER BY priority DESC, id
        "#,
    )
    .fetch_all(&mut *transaction)
    .await
    .wrap_err("failed to fetch delphi rules")?;

    tokio::task::spawn_blocking(move || {
        rules
            .into_iter()
            .map(|rule| {
                let program =
                    cel::Program::compile(&rule.rule).map_err(|error| {
                        eyre!(
                            "failed to compile delphi rule '{}': {error}",
                            rule.name
                        )
                    })?;
                Ok(CompiledRule {
                    id: rule.id,
                    name: rule.name,
                    expression: rule.rule,
                    on_issue_types: rule.on_issue_types,
                    program,
                })
            })
            .collect()
    })
    .await
    .wrap_err("failed to join cel compilation task")?
}

async fn insert_materialized_effects(
    revision: i64,
    effects: &[MaterializedEffect],
    transaction: &mut PgTransaction<'_>,
) -> Result<()> {
    if effects.is_empty() {
        return Ok(());
    }

    let detail_ids = effects
        .iter()
        .map(|effect| effect.detail_id)
        .collect::<Vec<_>>();
    let rule_ids = effects
        .iter()
        .map(|effect| effect.rule_id.0)
        .collect::<Vec<_>>();
    let severities = effects
        .iter()
        .map(|effect| effect.effect.severity)
        .collect::<Vec<_>>();

    sqlx::query!(
        r#"
        INSERT INTO delphi_rule_effects (
            revision,
            detail_id,
            rule_id,
            severity
        )
        SELECT $1, effect.*
        FROM UNNEST(
            $2::BIGINT[],
            $3::BIGINT[],
            $4::delphi_severity[]
        ) AS effect(detail_id, rule_id, severity)
        "#,
        revision,
        &detail_ids,
        &rule_ids,
        &severities as &[DelphiSeverity],
    )
    .execute(&mut *transaction)
    .await
    .wrap_err("failed to insert delphi rule effects")?;

    Ok(())
}

pub(super) fn evaluate_rule(
    program: &cel::Program,
    expression: &str,
    input: &RuleInput,
) -> Result<Option<DelphiRuleEffect>> {
    evaluate_rule_inner(program, input)
        .wrap_err_with(|| {
            let input = serde_json::to_string(input).unwrap_or_else(|error| {
                format!("<failed to serialize CEL input: {error}>")
            });
            format!("CEL input: {input}")
        })
        .wrap_err_with(|| format!("CEL expression: {expression}"))
}

fn evaluate_rule_inner(
    program: &cel::Program,
    input: &RuleInput,
) -> Result<Option<DelphiRuleEffect>> {
    let mut context = cel::Context::default();
    context
        .add_variable("schema_version", input.schema_version)
        .wrap_err("failed to add `schema_version` to cel context")?;
    context
        .add_variable("trace", &input.trace)
        .wrap_err("failed to add `trace` to cel context")?;
    context
        .add_variable("file_traces", &input.file_traces)
        .wrap_err("failed to add `file_traces` to cel context")?;
    context
        .add_variable("scan", &input.scan)
        .wrap_err("failed to add `scan` to cel context")?;
    context
        .add_variable("artifact", &input.artifact)
        .wrap_err("failed to add `artifact` to cel context")?;
    context
        .add_variable("project", &input.project)
        .wrap_err("failed to add `project` to cel context")?;
    context
        .add_variable("version", &input.version)
        .wrap_err("failed to add `version` to cel context")?;
    context
        .add_variable("file", &input.file)
        .wrap_err("failed to add `file` to cel context")?;

    let value = program
        .execute(&context)
        .wrap_err("failed to execute cel expression")?;
    let value = value.json().map_err(|error| {
        eyre!("failed to convert cel result to json: {error}")
    })?;

    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(severity) => {
            let severity =
                serde_json::from_value(serde_json::Value::String(severity))
                    .wrap_err("cel expression returned an invalid severity")?;
            Ok(Some(DelphiRuleEffect { severity }))
        }
        value => serde_json::from_value(value)
            .map(Some)
            .wrap_err("cel expression returned an invalid rule effect"),
    }
}
