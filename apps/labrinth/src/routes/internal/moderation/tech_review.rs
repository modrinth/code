use std::{collections::HashMap, fmt};

use actix_web::{HttpRequest, get, post, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, PgTransaction};
use tokio_stream::StreamExt;

use super::ownership::get_projects_ownership;
use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        DBProject,
        models::{
            DBProjectId, DBThread, DBThreadId, DBUser, DelphiReportId,
            DelphiReportIssueId, ProjectTypeId,
            delphi_report_item::{
                DelphiReportIssueStatus, DelphiSeverity, ReportIssueDetail,
            },
        },
        redis::RedisPool,
    },
    models::{
        ids::{FileId, ProjectId, ThreadId, VersionId},
        pats::Scopes,
        projects::{Project, ProjectStatus},
        threads::{MessageBody, Thread},
        users::User,
    },
    queue::session::AuthQueue,
    routes::{
        ApiError,
        internal::moderation::Ownership,
        v3::threads::{NewThreadMessage, thread_send_message_internal},
    },
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(search_projects)
        .service(get_report)
        .service(get_issue)
        .service(update_report)
        .service(update_issue);
}

/// Arguments for searching project technical reviews.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchProjects {
    #[serde(default = "default_limit")]
    #[schema(default = 20)]
    pub limit: u64,
    #[serde(default)]
    #[schema(default = 0)]
    pub page: u64,
    #[serde(default)]
    pub filter: SearchProjectsFilter,
    #[serde(default = "default_sort_by")]
    pub sort_by: SearchProjectsSort,
}

fn default_limit() -> u64 {
    20
}

fn default_sort_by() -> SearchProjectsSort {
    SearchProjectsSort::CreatedAsc
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchProjectsFilter {
    pub project_type: Vec<ProjectTypeId>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SearchProjectsSort {
    CreatedAsc,
    CreatedDesc,
    SeverityAsc,
    SeverityDesc,
}

impl fmt::Display for SearchProjectsSort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_value(*self).unwrap();
        let s = s.as_str().unwrap();
        write!(f, "{s}")
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileReport {
    /// ID of this report.
    pub id: DelphiReportId,
    /// ID of the file that was scanned.
    pub file_id: FileId,
    /// ID of the project version this report is for.
    pub version_id: VersionId,
    /// ID of the project this report is for.
    pub project_id: ProjectId,
    /// When the report for this file was created.
    pub created: DateTime<Utc>,
    /// Why this project was flagged.
    pub flag_reason: FlagReason,
    /// According to this report, how likely is the project malicious?
    pub severity: DelphiSeverity,
    /// Name of the flagged file.
    pub file_name: String,
    /// Size of the flagged file, in bytes.
    pub file_size: i32,
    /// URL to download the flagged file.
    pub download_url: String,
    /// What issues appeared in the file.
    pub issues: Vec<FileIssue>,
}

/// Issue raised by Delphi in a flagged file.
///
/// The issue is scoped to the JAR, not any specific class, but issues can be
/// raised because they appeared in a class - see [`FileIssueDetails`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileIssue {
    /// ID of the issue.
    pub id: DelphiReportIssueId,
    /// ID of the report this issue is a part of.
    pub report_id: DelphiReportId,
    /// Delphi-determined kind of issue that this is, e.g. `OBFUSCATED_NAMES`.
    ///
    /// Labrinth does not know the full set of kinds of issues, so this is kept
    /// as a string.
    pub issue_type: String,
    /// Is this issue valid (malicious) or a false positive (safe)?
    pub status: DelphiReportIssueStatus,
    /// Details of why this issue might have been raised, such as what file it
    /// was found in.
    pub details: Vec<ReportIssueDetail>,
}

/// Why a project was flagged for technical review.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FlagReason {
    /// Delphi anti-malware scanner flagged a file in the project.
    Delphi,
}

/// Get info on an issue in a Delphi report.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = OK, body = inline(FileIssue)))
)]
#[get("/issue/{issue_id}")]
async fn get_issue(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(DelphiReportIssueId,)>,
) -> Result<web::Json<FileIssue>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let (issue_id,) = path.into_inner();
    let row = sqlx::query!(
        r#"
        SELECT
            to_jsonb(dri)
            || jsonb_build_object(
                -- TODO: replace with `json_array` in Postgres 16
                'details', (
                    SELECT json_agg(to_jsonb(drid))
                    FROM delphi_report_issue_details drid
                    WHERE drid.issue_id = dri.id
                )
            ) AS "data!: sqlx::types::Json<FileIssue>"
        FROM delphi_report_issues dri
        WHERE dri.id = $1
        "#,
        issue_id as DelphiReportIssueId,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to fetch issue from database")?
    .ok_or(ApiError::NotFound)?;

    Ok(web::Json(row.data.0))
}

/// Get info on a specific report for a project.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = OK, body = inline(FileReport)))
)]
#[get("/report/{id}")]
async fn get_report(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(DelphiReportId,)>,
) -> Result<web::Json<FileReport>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let (report_id,) = path.into_inner();

    let row = sqlx::query!(
        r#"
        SELECT DISTINCT ON (dr.id)
            to_jsonb(dr)
            || jsonb_build_object(
                'file_id', to_base62(f.id),
                'version_id', to_base62(v.id),
                'project_id', to_base62(v.mod_id),
                'file_name', f.filename,
                'file_size', f.size,
                'flag_reason', 'delphi',
                'download_url', f.url,
                -- TODO: replace with `json_array` in Postgres 16
                'issues', (
                    SELECT json_agg(
                        to_jsonb(dri)
                        || jsonb_build_object(
                            -- TODO: replace with `json_array` in Postgres 16
                            'details', (
                                SELECT json_agg(to_jsonb(drid))
                                FROM delphi_report_issue_details drid
                                WHERE drid.issue_id = dri.id
                            )
                        )
                    )
                    FROM delphi_report_issues dri
                    WHERE dri.report_id = dr.id
                )
            ) AS "data!: sqlx::types::Json<FileReport>"
        FROM delphi_reports dr
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE dr.id = $1
        "#,
        report_id as DelphiReportId,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to fetch report from database")?
    .ok_or(ApiError::NotFound)?;

    Ok(web::Json(row.data.0))
}

/// See [`search_projects`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchResponse {
    /// List of reports returned.
    pub reports: Vec<FileReport>,
    /// Fetched project information for projects in the returned reports.
    pub projects: HashMap<ProjectId, ProjectModerationInfo>,
    /// Fetched moderation threads for projects in the returned reports.
    pub threads: HashMap<ThreadId, Thread>,
    /// Fetched owner information for projects.
    pub ownership: HashMap<ProjectId, Ownership>,
}

/// Limited set of project information returned by [`search_projects`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectModerationInfo {
    /// Projecet ID.
    pub id: ProjectId,
    /// Project moderation thread ID.
    pub thread_id: ThreadId,
    /// Project name.
    pub name: String,
    /// The aggregated project typos of the versions of this project
    pub project_types: Vec<String>,
    /// The URL of the icon of the project
    pub icon_url: Option<String>,
}

/// Searches all projects which are awaiting technical review.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = OK, body = inline(Vec<SearchResponse>)))
)]
#[post("/search")]
async fn search_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_req: web::Json<SearchProjects>,
) -> Result<web::Json<SearchResponse>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let sort_by = search_req.sort_by.to_string();
    let limit = search_req.limit.max(50);
    let offset = limit.saturating_mul(search_req.page);

    let limit =
        i64::try_from(limit).wrap_request_err("limit cannot fit into `i64`")?;
    let offset = i64::try_from(offset)
        .wrap_request_err("offset cannot fit into `i64`")?;

    let mut reports = Vec::<FileReport>::new();
    let mut project_ids = Vec::<DBProjectId>::new();
    let mut thread_ids = Vec::<DBThreadId>::new();
    let mut rows = sqlx::query!(
        r#"
        SELECT
            project_id AS "project_id: DBProjectId",
            project_thread_id AS "project_thread_id: DBThreadId",
            report AS "report!: sqlx::types::Json<FileReport>"
        FROM (
            SELECT DISTINCT ON (dr.id)
                dr.id       AS report_id,
                dr.created  AS report_created,
                dr.severity AS report_severity,
                m.id        AS project_id,
                t.id        AS project_thread_id,

                to_jsonb(dr)
                || jsonb_build_object(
                    'file_id', to_base62(f.id),
                    'version_id', to_base62(v.id),
                    'project_id', to_base62(v.mod_id),
                    'file_name', f.filename,
                    'file_size', f.size,
                    'flag_reason', 'delphi',
                    'download_url', f.url,
                    -- TODO: replace with `json_array` in Postgres 16
                    'issues', (
                        SELECT json_agg(
                            to_jsonb(dri)
                            || jsonb_build_object(
                                -- TODO: replace with `json_array` in Postgres 16
                                'details', (
                                    SELECT json_agg(
                                        jsonb_build_object(
                                            'id', drid.id,
                                            'issue_id', drid.issue_id,
                                            'key', drid.key,
                                            'file_path', drid.file_path,
                                            -- ignore `decompiled_source`
                                            'data', drid.data,
                                            'severity', drid.severity
                                        )
                                    )
                                    FROM delphi_report_issue_details drid
                                    WHERE drid.issue_id = dri.id
                                )
                            )
                        )
                        FROM delphi_report_issues dri
                        WHERE dri.report_id = dr.id
                    )
                ) AS report
            FROM delphi_reports dr
            INNER JOIN files f ON f.id = dr.file_id
            INNER JOIN versions v ON v.id = f.version_id
            INNER JOIN mods m ON m.id = v.mod_id
            INNER JOIN threads t ON t.mod_id = m.id

            -- filtering
            LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
            LEFT JOIN categories c ON c.id = mc.joining_category_id
            WHERE
                -- project type
                (cardinality($4::int[]) = 0 OR c.project_type = ANY($4::int[]))
                AND m.status NOT IN ('draft', 'rejected')
                AND dr.status = 'pending'
        ) t

        -- sorting
        ORDER BY
            CASE WHEN $3 = 'created_asc' THEN t.report_created ELSE TO_TIMESTAMP(0) END ASC,
            CASE WHEN $3 = 'created_desc' THEN t.report_created ELSE TO_TIMESTAMP(0) END DESC,
            CASE WHEN $3 = 'severity_asc' THEN t.report_severity ELSE 'low'::delphi_severity END ASC,
            CASE WHEN $3 = 'severity_desc' THEN t.report_severity ELSE 'low'::delphi_severity END DESC

        -- pagination
        LIMIT $1
        OFFSET $2
        "#,
        limit,
        offset,
        &sort_by,
        &search_req
            .filter
            .project_type
            .iter()
            .map(|ty| ty.0)
            .collect::<Vec<_>>(),
    )
    .fetch(&**pool);

    while let Some(row) = rows
        .next()
        .await
        .transpose()
        .wrap_internal_err("failed to fetch reports")?
    {
        reports.push(row.report.0);
        project_ids.push(row.project_id);
        thread_ids.push(row.project_thread_id);
    }

    let projects = DBProject::get_many_ids(&project_ids, &**pool, &redis)
        .await
        .wrap_internal_err("failed to fetch projects")?
        .into_iter()
        .map(|project| {
            (ProjectId::from(project.inner.id), Project::from(project))
        })
        .collect::<HashMap<_, _>>();
    let db_threads = DBThread::get_many(&thread_ids, &**pool)
        .await
        .wrap_internal_err("failed to fetch threads")?;
    let thread_author_ids = db_threads
        .iter()
        .flat_map(|thread| thread.members.clone())
        .collect::<Vec<_>>();
    let thread_authors =
        DBUser::get_many_ids(&thread_author_ids, &**pool, &redis)
            .await
            .wrap_internal_err("failed to fetch thread authors")?
            .into_iter()
            .map(From::from)
            .collect::<Vec<_>>();
    let threads = db_threads
        .into_iter()
        .map(|thread| {
            let thread = Thread::from(thread, thread_authors.clone(), &user);
            (thread.id, thread)
        })
        .collect::<HashMap<_, _>>();

    let project_list: Vec<Project> = projects.values().cloned().collect();

    let ownerships = get_projects_ownership(&project_list, &pool, &redis)
        .await
        .wrap_internal_err("failed to fetch project ownerships")?;
    let ownership = projects
        .keys()
        .copied()
        .zip(ownerships)
        .collect::<HashMap<_, _>>();

    Ok(web::Json(SearchResponse {
        reports,
        projects: projects
            .into_iter()
            .map(|(id, project)| {
                (
                    id,
                    ProjectModerationInfo {
                        id,
                        thread_id: project.thread_id,
                        name: project.name,
                        project_types: project.project_types,
                        icon_url: project.icon_url,
                    },
                )
            })
            .collect(),
        threads,
        ownership,
    }))
}

/// See [`update_report`] and [`update_issue`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateStatus {
    /// Status to set the issue to.
    pub status: DelphiReportIssueStatus,
    /// If `status` is [`DelphiReportIssueStatus::Unsafe`], provides a message
    /// to reject the project.
    pub message: Option<String>,
}

async fn maybe_reject_project(
    report_id: DelphiReportId,
    txn: &mut PgTransaction<'_>,
    update_req: &UpdateStatus,
    user: &User,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<(), ApiError> {
    if update_req.status != DelphiReportIssueStatus::Unsafe {
        return Ok(());
    };

    let record = sqlx::query!(
        r#"
        UPDATE mods
        SET status = $1
        FROM delphi_reports dr
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m ON v.mod_id = m.id
        INNER JOIN threads t ON t.mod_id = m.id
        WHERE dr.id = $2
        RETURNING
            m.id AS "project_id: DBProjectId",
            t.id AS "thread_id: DBThreadId"
        "#,
        ProjectStatus::Rejected.as_str(),
        report_id as DelphiReportId,
    )
    .fetch_one(&mut **txn)
    .await
    .wrap_internal_err("failed to mark project as rejected")?;

    if let Some(body) = &update_req.message {
        thread_send_message_internal(
            user,
            record.thread_id.into(),
            pool,
            NewThreadMessage {
                body: MessageBody::Text {
                    body: body.clone(),
                    private: true,
                    replying_to: None,
                    associated_images: Vec::new(),
                },
            },
            redis,
        )
        .await
        .wrap_internal_err("failed to add moderation thread message")?;
    }

    DBProject::clear_cache(record.project_id, None, None, redis)
        .await
        .wrap_internal_err("failed to clear project cache")?;

    Ok(())
}

/// Updates the state of a project based on a technical review report.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = NO_CONTENT))
)]
#[post("/report/{id}")]
async fn update_report(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    update_req: web::Json<UpdateStatus>,
    path: web::Path<(DelphiReportId,)>,
) -> Result<(), ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (report_id,) = path.into_inner();

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    sqlx::query!(
        "
        UPDATE delphi_reports dr
        SET status = $1
        WHERE dr.id = $2
        ",
        update_req.status as _,
        report_id as DelphiReportId,
    )
    .execute(&mut *txn)
    .await
    .wrap_internal_err("failed to update report")?;

    maybe_reject_project(
        report_id,
        &mut txn,
        &update_req,
        &user,
        &pool,
        &redis,
    )
    .await?;

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}

/// Updates the state of a technical review issue.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = NO_CONTENT))
)]
#[post("/issue/{id}")]
async fn update_issue(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    update_req: web::Json<UpdateStatus>,
    path: web::Path<(DelphiReportIssueId,)>,
) -> Result<(), ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (issue_id,) = path.into_inner();

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to start transaction")?;

    let record = sqlx::query!(
        r#"
        UPDATE delphi_report_issues dri
        SET status = $1
        FROM delphi_reports dr
        WHERE dri.id = $2 AND dr.id = dri.report_id
        RETURNING dr.id AS "report_id: DelphiReportId"
        "#,
        update_req.status as DelphiReportIssueStatus,
        issue_id as DelphiReportIssueId,
    )
    .fetch_one(&mut *txn)
    .await
    .wrap_internal_err("failed to update issue")?;

    maybe_reject_project(
        record.report_id,
        &mut txn,
        &update_req,
        &user,
        &pool,
        &redis,
    )
    .await?;

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}
