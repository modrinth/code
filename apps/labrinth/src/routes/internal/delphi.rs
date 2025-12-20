use std::{collections::HashMap, fmt::Write, sync::LazyLock, time::Instant};

use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        models::{
            DBFileId, DBProjectId, DBThreadId, DelphiReportId,
            DelphiReportIssueDetailsId, DelphiReportIssueId,
            delphi_report_item::{
                DBDelphiReport, DBDelphiReportIssue, DelphiSeverity,
                DelphiStatus, ReportIssueDetail,
            },
            thread_item::ThreadMessageBuilder,
        },
        redis::RedisPool,
    },
    models::{
        ids::{ProjectId, VersionId},
        pats::Scopes,
        threads::MessageBody,
    },
    queue::session::AuthQueue,
    routes::ApiError,
    util::{error::Context, guards::admin_key_guard},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("delphi")
            .service(ingest_report)
            .service(_run)
            .service(version)
            .service(issue_type_schema),
    );
}

static DELPHI_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .default_headers({
            HeaderMap::from_iter([(
                USER_AGENT,
                HeaderValue::from_static(concat!(
                    "Labrinth/",
                    env!("COMPILATION_DATE")
                )),
            )])
        })
        .build()
        .unwrap()
});

#[derive(Deserialize)]
struct DelphiReportIssueDetails {
    pub file: String,
    pub key: String,
    pub data: HashMap<String, serde_json::Value>,
    pub severity: DelphiSeverity,
}

#[derive(Deserialize)]
struct DelphiReport {
    pub url: String,
    pub project_id: crate::models::ids::ProjectId,
    #[serde(rename = "version_id")]
    pub version_id: crate::models::ids::VersionId,
    pub file_id: crate::models::ids::FileId,
    /// A sequential, monotonically increasing version number for the
    /// Delphi version that generated this report.
    pub delphi_version: i32,
    pub issues: HashMap<String, Vec<DelphiReportIssueDetails>>,
    pub severity: DelphiSeverity,
    /// Map of [`DelphiReportIssueDetails::file`] to the decompiled Java source
    /// code.
    pub decompiled_sources: HashMap<String, Option<String>>,
}

impl DelphiReport {
    async fn send_to_slack(
        &self,
        pool: &PgPool,
        redis: &RedisPool,
    ) -> Result<(), ApiError> {
        let webhook_url = dotenvy::var("DELPHI_SLACK_WEBHOOK")?;

        let mut message_header =
            format!("⚠️ Suspicious traces found at {}", self.url);

        for (issue, trace) in &self.issues {
            for DelphiReportIssueDetails { file, .. } in trace {
                let decompiled_source =
                    self.decompiled_sources.get(file).and_then(|o| o.as_ref());

                write!(
                    &mut message_header,
                    "\n issue {issue} found at class `{file}`:\n```\n{}\n```",
                    decompiled_source.as_ref().map_or(
                        "No decompiled source available",
                        |decompiled_source| &**decompiled_source
                    )
                )
                .ok();
            }
        }

        crate::util::webhook::send_slack_project_webhook(
            self.project_id,
            pool,
            redis,
            webhook_url,
            Some(message_header),
        )
        .await
    }
}

#[derive(Deserialize)]
pub struct DelphiRunParameters {
    pub file_id: crate::models::ids::FileId,
}

#[post("ingest", guard = "admin_key_guard")]
async fn ingest_report(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    web::Json(report): web::Json<serde_json::Value>,
) -> Result<(), ApiError> {
    // treat this as an internal error, since it's not a bad request from the
    // client's side - it's *our* fault for handling the Delphi schema wrong
    // this could happen if Delphi updates and Labrinth doesn't
    let report = serde_json::from_value::<DelphiReport>(report.clone())
        .wrap_internal_err_with(|| {
            eyre!(
                "Delphi sent a response which does not match our schema\n\n{}",
                serde_json::to_string_pretty(&report).unwrap()
            )
        })?;

    ingest_report_deserialized(pool, redis, report).await
}

#[tracing::instrument(
    level = "info",
    skip_all,
    fields(
        %report.url,
        %report.file_id,
        %report.project_id,
        %report.version_id,
    )
)]
async fn ingest_report_deserialized(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    report: DelphiReport,
) -> Result<(), ApiError> {
    if report.issues.is_empty() {
        info!("No issues found for file");
        return Ok(());
    }

    report.send_to_slack(&pool, &redis).await.ok();

    let mut transaction = pool.begin().await?;

    let report_id = DBDelphiReport {
        id: DelphiReportId(0), // This will be set by the database
        file_id: Some(DBFileId(report.file_id.0 as i64)),
        delphi_version: report.delphi_version,
        artifact_url: report.url.clone(),
        created: DateTime::<Utc>::MIN_UTC, // This will be set by the database
        severity: report.severity,
    }
    .upsert(&mut transaction)
    .await?;

    info!(
        num_issues = %report.issues.len(),
        "Delphi found issues in file",
    );

    let record = sqlx::query!(
        r#"
        SELECT
            EXISTS(
                SELECT 1 FROM delphi_issue_details_with_statuses didws
                WHERE didws.project_id = $1 AND didws.status = 'pending'
            ) AS "pending_issue_details_exist!",
            t.id AS "thread_id: DBThreadId"
        FROM mods m
        INNER JOIN threads t ON t.mod_id = $1
        "#,
        DBProjectId::from(report.project_id) as _,
    )
    .fetch_one(&mut *transaction)
    .await
    .wrap_internal_err("failed to check if pending issue details exist")?;

    if record.pending_issue_details_exist {
        info!(
            "File's project already has pending issue details, is not entering tech review queue"
        );
    } else {
        info!("File's project is entering tech review queue");

        ThreadMessageBuilder {
            author_id: None,
            body: MessageBody::TechReviewEntered,
            thread_id: record.thread_id,
            hide_identity: false,
        }
        .insert(&mut transaction)
        .await
        .wrap_internal_err("failed to add entering tech review message")?;
    }

    // TODO: Currently, the way we determine if an issue is in tech review or not
    // is if it has any issue details which are pending.
    // If you mark all issue details are safe or not safe - even if you don't
    // submit the final report - the project will be taken out of tech review
    // queue, and into moderation queue.
    //
    // This is undesirable, but we can't rework the database schema to fix it
    // right now. As a hack, we add a dummy report issue which blocks the
    // project from exiting the tech review queue.
    {
        let dummy_issue_id = DBDelphiReportIssue {
            id: DelphiReportIssueId(0), // This will be set by the database
            report_id,
            issue_type: "__dummy".into(),
        }
        .insert(&mut transaction)
        .await?;

        ReportIssueDetail {
            id: DelphiReportIssueDetailsId(0), // This will be set by the database
            issue_id: dummy_issue_id,
            key: "".into(),
            file_path: "".into(),
            decompiled_source: None,
            data: HashMap::new(),
            severity: DelphiSeverity::Low,
            status: DelphiStatus::Pending,
        }
        .insert(&mut transaction)
        .await?;
    }

    for (issue_type, issue_details) in report.issues {
        let issue_id = DBDelphiReportIssue {
            id: DelphiReportIssueId(0), // This will be set by the database
            report_id,
            issue_type,
        }
        .insert(&mut transaction)
        .await?;

        // This is required to handle the case where the same Delphi version is re-run on the same file
        ReportIssueDetail::remove_all_by_issue_id(issue_id, &mut transaction)
            .await?;

        for issue_detail in issue_details {
            let decompiled_source =
                report.decompiled_sources.get(&issue_detail.file);

            ReportIssueDetail {
                id: DelphiReportIssueDetailsId(0), // This will be set by the database
                issue_id,
                key: issue_detail.key,
                file_path: issue_detail.file,
                decompiled_source: decompiled_source.cloned().flatten(),
                data: issue_detail.data,
                severity: issue_detail.severity,
                status: DelphiStatus::Pending,
            }
            .insert(&mut transaction)
            .await?;
        }
    }

    transaction.commit().await?;

    Ok(())
}

pub async fn run(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    run_parameters: DelphiRunParameters,
) -> Result<HttpResponse, ApiError> {
    let file_data = sqlx::query!(
        r#"
        SELECT
            version_id AS "version_id: crate::database::models::DBVersionId",
            versions.mod_id AS "project_id: crate::database::models::DBProjectId",
            files.url AS "url"
        FROM files INNER JOIN versions ON files.version_id = versions.id
        WHERE files.id = $1
        "#,
        run_parameters.file_id.0 as i64
    )
    .fetch_one(exec)
    .await?;

    tracing::debug!(
        "Running Delphi for project {}, version {}, file {}",
        file_data.project_id.0,
        file_data.version_id.0,
        run_parameters.file_id.0
    );

    DELPHI_CLIENT
        .post(dotenvy::var("DELPHI_URL")?)
        .json(&serde_json::json!({
            "url": file_data.url,
            "project_id": ProjectId(file_data.project_id.0 as u64),
            "version_id": VersionId(file_data.version_id.0 as u64),
            "file_id": run_parameters.file_id,
        }))
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .map_err(ApiError::Delphi)?;

    Ok(HttpResponse::NoContent().finish())
}

#[post("run")]
async fn _run(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    run_parameters: web::Query<DelphiRunParameters>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    run(&**pool, run_parameters.into_inner()).await
}

#[get("version")]
async fn version(
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
        Scopes::PROJECT_READ,
    )
    .await?;

    Ok(HttpResponse::Ok().json(
        sqlx::query_scalar!("SELECT MAX(delphi_version) FROM delphi_reports")
            .fetch_one(&**pool)
            .await?,
    ))
}

#[get("issue_type/schema")]
async fn issue_type_schema(
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
        Scopes::PROJECT_READ,
    )
    .await?;

    // This route is expected to be called often by the frontend, and Delphi is not necessarily
    // built to scale beyond malware analysis, so cache the result of its quasi-constant-valued
    // schema route to alleviate the load on it

    static CACHED_ISSUE_TYPE_SCHEMA: Mutex<
        Option<(serde_json::Map<String, serde_json::Value>, Instant)>,
    > = Mutex::const_new(None);

    match &mut *CACHED_ISSUE_TYPE_SCHEMA.lock().await {
        Some((schema, last_fetch)) if last_fetch.elapsed().as_secs() < 60 => {
            Ok(HttpResponse::Ok().json(schema))
        }
        cache_entry => Ok(HttpResponse::Ok().json(
            &cache_entry
                .insert((
                    DELPHI_CLIENT
                        .get(format!("{}/schema", dotenvy::var("DELPHI_URL")?))
                        .send()
                        .await
                        .and_then(|res| res.error_for_status())
                        .map_err(ApiError::Delphi)?
                        .json::<serde_json::Map<String, serde_json::Value>>()
                        .await
                        .map_err(ApiError::Delphi)?,
                    Instant::now(),
                ))
                .0,
        )),
    }
}
