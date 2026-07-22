use std::collections::{BTreeMap, HashMap};

use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ariadne::ids::base62_impl::to_base62;
use bytes::Bytes;
use eyre::{Context as _, Result, eyre};
use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;
use sqlx::types::Json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use utoipa::{PartialSchema, ToSchema};

use super::rules::DelphiRuleEffect;
use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool, PgTransaction, models::delphi_report_item::DelphiSeverity,
        redis::RedisPool,
    },
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
};

const RULE_SCAN_LOCK_ID: i64 = 0x6465_6c70_6869_7275;
const PROGRESS_INTERVAL: usize = 50;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_rule_schema).service(scan_rules);
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

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct RuleInput {
    pub(super) schema_version: u32,
    pub(super) trace: RuleTrace,
    pub(super) scan: RuleScan,
    pub(super) artifact: RuleArtifact,
    pub(super) scope: RuleScope,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct RuleTrace {
    pub(super) key: String,
    pub(super) issue_type: String,
    pub(super) severity: DelphiSeverity,
    pub(super) jar: Option<String>,
    pub(super) file_path: String,
    pub(super) data: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct RuleScan {
    pub(super) delphi_version: i32,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct RuleArtifact {
    pub(super) size: Option<i32>,
    pub(super) hashes: BTreeMap<String, String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct RuleScope {
    pub(super) project_id: Option<String>,
    pub(super) version_id: Option<String>,
    pub(super) file_id: Option<String>,
}

struct CompiledRule {
    id: i64,
    program: cel::Program,
}

struct MaterializedEffect {
    detail_id: i64,
    rule_id: i64,
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

/// Get the schemas for the CEL input and output values.
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
    .await?;

    let mut schemas = Vec::new();
    <RuleInput as ToSchema>::schemas(&mut schemas);
    <Option<DelphiRuleEffect> as ToSchema>::schemas(&mut schemas);

    Ok(web::Json(DelphiRuleSchemaResponse {
        input: schema_to_value(<RuleInput as PartialSchema>::schema())?,
        output: schema_to_value(
            <Option<DelphiRuleEffect> as PartialSchema>::schema(),
        )?,
        components: schemas
            .into_iter()
            .map(|(name, schema)| Ok((name, schema_to_value(schema)?)))
            .collect::<Result<_, ApiError>>()?,
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
    .await?;

    let mut transaction = crate::util::error::Context::wrap_internal_err(
        pool.begin().await,
        "failed to begin delphi rule scan",
    )?;

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
        return Err(ApiError::Conflict(
            "a delphi rule scan is already running".to_string(),
        ));
    }

    let (sender, receiver) = mpsc::unbounded_channel();
    actix_web::rt::spawn(async move {
        match run_scan(transaction, &sender).await {
            Ok(summary) => {
                send_event(
                    &sender,
                    "complete",
                    &RuleScanEvent {
                        phase: "complete",
                        revision: summary.revision,
                        scanned: summary.scanned,
                        total: summary.total,
                        effects: summary.effects,
                    },
                );
            }
            Err(error) => {
                tracing::error!(error = ?error, "delphi rule scan failed");
                send_event(
                    &sender,
                    "failed",
                    &RuleScanErrorEvent {
                        message: &error.to_string(),
                    },
                );
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

    let rules = sqlx::query!(
        r#"
        SELECT id, rule
        FROM delphi_rules
        WHERE NOT delete_on_next_revision
        ORDER BY id
        "#,
    )
    .fetch_all(&mut transaction)
    .await
    .wrap_err("failed to fetch delphi rules")?
    .into_iter()
    .map(|rule| {
        let program = cel::Program::compile(&rule.rule).map_err(|error| {
            eyre!("failed to compile delphi rule {}: {error}", rule.id)
        })?;
        Ok(CompiledRule {
            id: rule.id,
            program,
        })
    })
    .collect::<Result<Vec<_>>>()?;

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) AS \"count!\" FROM delphi_report_issue_details",
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
            report.delphi_version,
            file.size AS "size?",
            file.id AS "file_id?",
            version.id AS "version_id?",
            version.mod_id AS "project_id?",
            COALESCE(file_hashes.hashes, '{}'::jsonb)
                AS "hashes!: Json<BTreeMap<String, String>>"
        FROM delphi_report_issue_details detail
        INNER JOIN delphi_report_issues issue ON issue.id = detail.issue_id
        INNER JOIN delphi_reports report ON report.id = issue.report_id
        LEFT JOIN files file ON file.id = report.file_id
        LEFT JOIN versions version ON version.id = file.version_id
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
    send_progress(sender, "scanning", revision, 0, total, 0);

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
            scan: RuleScan {
                delphi_version: detail.delphi_version,
            },
            artifact: RuleArtifact {
                size: detail.size,
                hashes: detail.hashes.0,
            },
            scope: RuleScope {
                project_id: detail.project_id.map(to_public_id),
                version_id: detail.version_id.map(to_public_id),
                file_id: detail.file_id.map(to_public_id),
            },
        };

        for rule in &rules {
            let effect = evaluate_rule(&rule.program, &input)
                .wrap_err_with(|| {
                    format!(
                        "failed to evaluate delphi rule {} for detail {detail_id}",
                        rule.id
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
            send_progress(
                sender,
                "scanning",
                revision,
                scanned,
                total,
                effects.len(),
            );
            tokio::task::yield_now().await;
        }
    }
    drop(details);

    send_progress(sender, "publishing", revision, total, total, effects.len());

    let detail_ids = effects
        .iter()
        .map(|effect| effect.detail_id)
        .collect::<Vec<_>>();
    let rule_ids = effects
        .iter()
        .map(|effect| effect.rule_id)
        .collect::<Vec<_>>();
    let severities = effects
        .iter()
        .map(|effect| effect.effect.severity)
        .collect::<Vec<_>>();
    let hidden = effects
        .iter()
        .map(|effect| effect.effect.hidden)
        .collect::<Vec<_>>();

    if !effects.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO delphi_rule_effects (
                revision,
                detail_id,
                rule_id,
                severity,
                hidden
            )
            SELECT $1, effect.*
            FROM UNNEST(
                $2::BIGINT[],
                $3::BIGINT[],
                $4::delphi_severity[],
                $5::BOOLEAN[]
            ) AS effect(detail_id, rule_id, severity, hidden)
            "#,
            revision,
            &detail_ids,
            &rule_ids,
            &severities as &[Option<DelphiSeverity>],
            &hidden,
        )
        .execute(&mut transaction)
        .await
        .wrap_err("failed to insert delphi rule effects")?;
    }

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

pub(super) fn evaluate_rule(
    program: &cel::Program,
    input: impl Serialize,
) -> Result<Option<DelphiRuleEffect>> {
    let mut context = cel::Context::default();
    context
        .add_variable("input", input)
        .wrap_err("failed to build cel input")?;

    let value = program
        .execute(&context)
        .wrap_err("failed to execute cel expression")?;
    let value = value.json().map_err(|error| {
        eyre!("failed to convert cel result to json: {error}")
    })?;

    match value {
        serde_json::Value::Null => Ok(None),
        value => serde_json::from_value(value)
            .map(Some)
            .wrap_err("cel expression returned an invalid rule effect"),
    }
}

fn to_public_id(id: i64) -> String {
    to_base62(id as u64)
}

fn send_progress(
    sender: &mpsc::UnboundedSender<Bytes>,
    phase: &'static str,
    revision: i64,
    scanned: usize,
    total: usize,
    effects: usize,
) {
    send_event(
        sender,
        "progress",
        &RuleScanEvent {
            phase,
            revision,
            scanned,
            total,
            effects,
        },
    );
}

fn send_event(
    sender: &mpsc::UnboundedSender<Bytes>,
    event: &str,
    data: &impl Serialize,
) {
    let Ok(data) = serde_json::to_string(data) else {
        return;
    };
    let _ =
        sender.send(Bytes::from(format!("event: {event}\ndata: {data}\n\n")));
}
