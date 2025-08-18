use std::{collections::HashMap, fmt::Write, io, sync::LazyLock};

use actix_web::{HttpResponse, get, post, put, web};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::info;

use crate::{
    database::{
        models::{
            DBFileId, DelphiReportId, DelphiReportIssueId,
            DelphiReportIssueJavaClassId,
            delphi_report_item::{
                DBDelphiReport, DBDelphiReportIssue,
                DBDelphiReportIssueJavaClass, DecompiledJavaClassSource,
                DelphiReportIssueStatus, DelphiReportIssueType,
                DelphiReportListOrder, InternalJavaClassName,
            },
        },
        redis::RedisPool,
    },
    routes::ApiError,
    util::guards::admin_key_guard,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("delphi")
            .service(ingest_report)
            .service(_run)
            .service(version)
            .service(issues)
            .service(update_issue),
    );
}

#[derive(Deserialize)]
struct DelphiReport {
    pub url: String,
    pub project_id: crate::models::ids::ProjectId,
    #[serde(rename = "version_id")]
    pub _version_id: crate::models::ids::VersionId,
    pub file_id: crate::models::ids::FileId,
    /// A sequential, monotonically increasing version number for the
    /// Delphi version that generated this report.
    pub delphi_version: i32,
    pub issues: HashMap<
        DelphiReportIssueType,
        HashMap<InternalJavaClassName, DecompiledJavaClassSource>,
    >,
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
            for (path, code) in trace {
                write!(
                    &mut message_header,
                    "\n issue {issue} found at file {path}:\n```\n{code}\n```"
                )
                .ok();
            }
        }

        crate::util::webhook::send_slack_webhook(
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
    pub file_id: crate::database::models::ids::DBFileId,
}

#[post("ingest", guard = "admin_key_guard")]
async fn ingest_report(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    web::Json(report): web::Json<DelphiReport>,
) -> Result<HttpResponse, ApiError> {
    if report.issues.is_empty() {
        info!("No issues found for file {}", report.url);
        return Ok(HttpResponse::NoContent().finish());
    }

    report.send_to_slack(&pool, &redis).await.ok();

    let mut transaction = pool.begin().await?;

    let report_id = DBDelphiReport {
        id: DelphiReportId(0), // This will be set by the database
        file_id: Some(DBFileId(report.file_id.0 as i64)),
        delphi_version: report.delphi_version,
        artifact_url: report.url.clone(),
        created: DateTime::<Utc>::MIN_UTC, // This will be set by the database
    }
    .upsert(&mut transaction)
    .await?;

    for (issue_type, issue_java_classes) in report.issues {
        let issue_id = DBDelphiReportIssue {
            id: DelphiReportIssueId(0), // This will be set by the database
            report_id,
            issue_type,
            status: DelphiReportIssueStatus::Pending,
        }
        .upsert(&mut transaction)
        .await?;

        for (internal_class_name, decompiled_source) in issue_java_classes {
            DBDelphiReportIssueJavaClass {
                id: DelphiReportIssueJavaClassId(0), // This will be set by the database
                issue_id,
                internal_class_name,
                decompiled_source,
            }
            .upsert(&mut transaction)
            .await?;
        }
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
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
        run_parameters.file_id.0
    )
    .fetch_one(exec)
    .await?;

    static DELPHI_CLIENT: LazyLock<reqwest::Client> =
        LazyLock::new(reqwest::Client::new);

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
            "project_id": file_data.project_id,
            "version_id": file_data.version_id,
            "file_id": run_parameters.file_id,
        }))
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .map_err(ApiError::Delphi)?;

    Ok(HttpResponse::NoContent().finish())
}

#[post("run", guard = "admin_key_guard")]
async fn _run(
    pool: web::Data<PgPool>,
    run_parameters: web::Query<DelphiRunParameters>,
) -> Result<HttpResponse, ApiError> {
    run(&**pool, run_parameters.into_inner()).await
}

#[get("version", guard = "admin_key_guard")]
async fn version(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(
        sqlx::query_scalar!("SELECT MAX(delphi_version) FROM delphi_reports")
            .fetch_one(&**pool)
            .await?,
    ))
}

#[derive(Deserialize)]
struct DelphiIssuesSearchOptions {
    #[serde(rename = "type")]
    ty: Option<DelphiReportIssueType>,
    status: Option<DelphiReportIssueStatus>,
    order_by: Option<DelphiReportListOrder>,
    count: Option<u16>,
    offset: Option<u64>,
}

#[get("issues", guard = "admin_key_guard")]
async fn issues(
    pool: web::Data<PgPool>,
    search_options: web::Query<DelphiIssuesSearchOptions>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(
        DBDelphiReportIssue::find_all_by(
            search_options.ty,
            search_options.status,
            search_options.order_by,
            search_options.count,
            search_options
                .offset
                .map(|offset| offset.try_into())
                .transpose()
                .map_err(|err| {
                    io::Error::other(format!("Invalid offset: {err}"))
                })?,
            &**pool,
        )
        .await?,
    ))
}

#[put("issue/{issue_id}", guard = "admin_key_guard")]
async fn update_issue(
    pool: web::Data<PgPool>,
    issue_id: web::Path<DelphiReportIssueId>,
    web::Json(update_data): web::Json<DBDelphiReportIssue>,
) -> Result<HttpResponse, ApiError> {
    let new_id = issue_id.into_inner();

    let mut transaction = pool.begin().await?;

    let modified_same_issue = (DBDelphiReportIssue {
        id: new_id, // Doesn't matter, upsert done for values of other fields
        report_id: update_data.report_id,
        issue_type: update_data.issue_type,
        status: update_data.status,
    })
    .upsert(&mut transaction)
    .await?
        == new_id;

    transaction.commit().await?;

    if modified_same_issue {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::Created().finish())
    }
}
