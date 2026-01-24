use std::{collections::HashMap, fmt};

use actix_web::{HttpRequest, get, patch, post, put, web};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::ownership::get_projects_ownership;
use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        DBProject,
        models::{
            DBFileId, DBProjectId, DBThread, DBThreadId, DBUser, DBVersion,
            DBVersionId, DelphiReportId, DelphiReportIssueDetailsId,
            DelphiReportIssueId, ProjectTypeId,
            delphi_report_item::{
                DBDelphiReport, DelphiSeverity, DelphiStatus, DelphiVerdict,
                ReportIssueDetail,
            },
            thread_item::ThreadMessageBuilder,
            version_item::VersionQueryResult,
        },
        redis::RedisPool,
    },
    models::{
        ids::{FileId, ProjectId, ThreadId, VersionId},
        pats::Scopes,
        projects::{Project, ProjectStatus},
        threads::{MessageBody, Thread},
    },
    queue::session::AuthQueue,
    routes::{ApiError, internal::moderation::Ownership},
    util::error::Context,
};
use eyre::eyre;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(search_projects)
        .service(get_project_report)
        .service(get_report)
        .service(get_issue)
        .service(submit_report)
        .service(update_issue_detail)
        .service(add_report);
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
    #[serde(default)]
    pub project_type: Vec<ProjectTypeId>,
    #[serde(default)]
    pub replied_to: Option<RepliedTo>,
    #[serde(default)]
    pub project_status: Vec<ProjectStatus>,
    #[serde(default)]
    pub issue_type: Vec<String>,
}

/// Filter by whether a moderator has replied to the last message in the
/// project's moderation thread.
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
pub enum RepliedTo {
    /// Last message in the thread is from a moderator, indicating a moderator
    /// has replied to it.
    Replied,
    /// Last message in the thread is not from a moderator.
    Unreplied,
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
    pub report_id: DelphiReportId,
    /// ID of the file that was scanned.
    pub file_id: FileId,
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
    #[serde(default)]
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
    /// Details of why this issue might have been raised, such as what file it
    /// was found in.
    #[serde(default)]
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
                    SELECT coalesce(jsonb_agg(
                        jsonb_build_object(
                            'id', didws.id,
                            'issue_id', didws.issue_id,
                            'key', didws.key,
                            'file_path', didws.file_path,
                            'decompiled_source', didws.decompiled_source,
                            'data', didws.data,
                            'severity', didws.severity,
                            'status', didws.status
                        )
                    ), '[]'::jsonb)
                    FROM delphi_issue_details_with_statuses didws
                    WHERE didws.issue_id = dri.id
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
                'report_id', dr.id,
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
                                SELECT coalesce(jsonb_agg(
                                    jsonb_build_object(
                                        'id', didws.id,
                                        'issue_id', didws.issue_id,
                                        'key', didws.key,
                                        'file_path', didws.file_path,
                                        'decompiled_source', didws.decompiled_source,
                                        'data', didws.data,
                                        'severity', didws.severity,
                                        'status', didws.status
                                    )
                                ), '[]'::jsonb)
                                FROM delphi_issue_details_with_statuses didws
                                WHERE didws.issue_id = dri.id
                            )
                        )
                    )
                    FROM delphi_report_issues dri
                    WHERE
                        dri.report_id = dr.id
                        -- see delphi.rs todo comment
                        AND dri.issue_type != '__dummy'
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
    /// List of reported projects returned, and their report data.
    pub project_reports: Vec<ProjectReport>,
    /// Fetched project information for projects in the returned reports.
    pub projects: HashMap<ProjectId, ProjectModerationInfo>,
    /// Fetched moderation threads for projects in the returned reports.
    pub threads: HashMap<ThreadId, Thread>,
    /// Fetched owner information for projects.
    pub ownership: HashMap<ProjectId, Ownership>,
}

/// Response for a single project's technical review report.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectReportResponse {
    /// The project's technical review report.
    pub project_report: Option<ProjectReport>,
    /// The moderation thread for this project.
    pub thread: Thread,
}

/// Single project's reports from a search response.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectReport {
    /// ID of the project this report is for.
    pub project_id: ProjectId,
    /// Highest severity of any report of any file of any version under this
    /// project.
    pub max_severity: Option<DelphiSeverity>,
    /// Reports for this project's versions.
    #[serde(default)]
    pub versions: Vec<VersionReport>,
}

/// Single project version's reports from a search response.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VersionReport {
    /// ID of the project version this report is for.
    pub version_id: VersionId,
    /// Reports for this version's files.
    #[serde(default)]
    pub files: Vec<FileReport>,
}

/// Limited set of project information returned by [`search_projects`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectModerationInfo {
    /// Project ID.
    pub id: ProjectId,
    /// Project moderation thread ID.
    pub thread_id: ThreadId,
    /// Project name.
    pub name: String,
    /// The aggregated project typos of the versions of this project
    #[serde(default)]
    pub project_types: Vec<String>,
    /// The URL of the icon of the project
    pub icon_url: Option<String>,
}

async fn fetch_project_reports(
    project_ids: &[DBProjectId],
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<ProjectReport>, ApiError> {
    struct FileRow {
        file_id: DBFileId,
        version_id: DBVersionId,
        url: String,
        filename: String,
        size: i32,
    }

    struct DelphiReportRow {
        report_id: DelphiReportId,
        file_id: DBFileId,
        created: DateTime<Utc>,
        severity: DelphiSeverity,
    }

    struct DelphiReportIssueRow {
        id: DelphiReportIssueId,
        report_id: DelphiReportId,
        issue_type: String,
    }

    let version_id_rows = sqlx::query!(
        "SELECT id FROM versions WHERE mod_id = ANY($1::bigint[])",
        &project_ids.iter().map(|id| id.0).collect::<Vec<_>>()
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch version ids")?;

    let version_ids: Vec<DBVersionId> = version_id_rows
        .into_iter()
        .map(|r| DBVersionId(r.id))
        .collect();

    let versions = DBVersion::get_many(&version_ids, pool, redis)
        .await
        .wrap_internal_err("failed to fetch versions")?;

    let file_rows = sqlx::query!(
        r#"
        SELECT
            id AS "file_id: DBFileId",
            version_id AS "version_id: DBVersionId",
            url,
            filename,
            size
        FROM files
        WHERE version_id = ANY($1::bigint[])
        "#,
        &version_ids.iter().map(|id| id.0).collect::<Vec<_>>()
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch files")?;

    let report_rows = sqlx::query!(
        r#"
        SELECT
            id AS "report_id!: DelphiReportId",
            file_id AS "file_id!: DBFileId",
            created,
            severity AS "severity!: DelphiSeverity"
        FROM delphi_reports
        WHERE file_id = ANY($1::bigint[])
        "#,
        &file_rows.iter().map(|f| f.file_id.0).collect::<Vec<_>>()
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch delphi reports")?;

    let issue_rows = sqlx::query!(
        r#"
        SELECT
            id AS "id: DelphiReportIssueId",
            report_id AS "report_id: DelphiReportId",
            issue_type
        FROM delphi_report_issues
        WHERE report_id = ANY($1::bigint[])
        "#,
        &report_rows
            .iter()
            .map(|r| r.report_id.0)
            .collect::<Vec<_>>()
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch delphi report issues")?;

    let issue_ids: Vec<DelphiReportIssueId> =
        issue_rows.iter().map(|i| i.id).collect();

    let detail_rows = sqlx::query!(
        r#"
        SELECT
            drid.id AS "id!: DelphiReportIssueDetailsId",
            drid.issue_id AS "issue_id!: DelphiReportIssueId",
            drid.key AS "key!: String",
            drid.file_path AS "file_path!: String",
            drid.data AS "data!: sqlx::types::Json<HashMap<String, serde_json::Value>>",
            drid.severity AS "severity!: DelphiSeverity",
            COALESCE(didv.verdict, 'pending'::delphi_report_issue_status) AS "status!: DelphiStatus"
        FROM delphi_report_issue_details drid
        INNER JOIN delphi_report_issues dri ON dri.id = drid.issue_id
        INNER JOIN delphi_reports dr ON dr.id = dri.report_id
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m ON m.id = v.mod_id
        LEFT JOIN delphi_issue_detail_verdicts didv
            ON m.id = didv.project_id AND drid.key = didv.detail_key
        WHERE drid.issue_id = ANY($1::bigint[])
        "#,
        &issue_ids.iter().map(|i| i.0).collect::<Vec<_>>()
    )
    .fetch_all(pool)
    .await
    .wrap_internal_err("failed to fetch delphi issue details")?;

    let versions_by_project: HashMap<DBProjectId, Vec<VersionQueryResult>> =
        versions
            .into_iter()
            .into_group_map_by(|v| v.inner.project_id);

    let files_by_version: HashMap<DBVersionId, Vec<FileRow>> = file_rows
        .into_iter()
        .map(|r| FileRow {
            file_id: r.file_id,
            version_id: r.version_id,
            url: r.url,
            filename: r.filename,
            size: r.size,
        })
        .into_group_map_by(|f| f.version_id);

    let reports_by_file: HashMap<DBFileId, Vec<DelphiReportRow>> = report_rows
        .into_iter()
        .map(|r| DelphiReportRow {
            report_id: r.report_id,
            file_id: r.file_id,
            created: r.created,
            severity: r.severity,
        })
        .into_group_map_by(|r| r.file_id);

    let issues_by_report: HashMap<DelphiReportId, Vec<DelphiReportIssueRow>> =
        issue_rows
            .into_iter()
            .map(|i| DelphiReportIssueRow {
                id: i.id,
                report_id: i.report_id,
                issue_type: i.issue_type,
            })
            .into_group_map_by(|i| i.report_id);

    let details_by_issue: HashMap<DelphiReportIssueId, Vec<ReportIssueDetail>> =
        detail_rows
            .into_iter()
            .map(|d| ReportIssueDetail {
                id: d.id,
                issue_id: d.issue_id,
                key: d.key,
                file_path: d.file_path,
                decompiled_source: None,
                data: d.data.0,
                severity: d.severity,
                status: d.status,
            })
            .into_group_map_by(|d| d.issue_id);

    let empty_versions: Vec<VersionQueryResult> = vec![];
    let empty_files: Vec<FileRow> = vec![];
    let empty_reports: Vec<DelphiReportRow> = vec![];
    let empty_issues: Vec<DelphiReportIssueRow> = vec![];
    let empty_details: Vec<ReportIssueDetail> = vec![];

    let mut project_reports = Vec::<ProjectReport>::new();

    for project_id in project_ids {
        let project_versions = versions_by_project
            .get(project_id)
            .unwrap_or(&empty_versions);

        let mut version_reports = Vec::new();

        for version_query in project_versions {
            let version_files = files_by_version
                .get(&version_query.inner.id)
                .unwrap_or(&empty_files);

            let mut file_reports = Vec::new();

            for file_row in version_files {
                let report_list = reports_by_file
                    .get(&file_row.file_id)
                    .unwrap_or(&empty_reports);

                for report_row in report_list {
                    let report_issues = issues_by_report
                        .get(&report_row.report_id)
                        .unwrap_or(&empty_issues);

                    let mut file_issues = Vec::new();

                    for issue_row in report_issues {
                        if issue_row.issue_type == "__dummy" {
                            continue;
                        }

                        let issue_details = details_by_issue
                            .get(&issue_row.id)
                            .unwrap_or(&empty_details);

                        file_issues.push(FileIssue {
                            id: issue_row.id,
                            report_id: issue_row.report_id,
                            issue_type: issue_row.issue_type.clone(),
                            details: issue_details.clone(),
                        });
                    }

                    file_reports.push(FileReport {
                        report_id: report_row.report_id,
                        file_id: FileId::from(file_row.file_id),
                        created: report_row.created,
                        flag_reason: FlagReason::Delphi,
                        severity: report_row.severity,
                        file_name: file_row.filename.clone(),
                        file_size: file_row.size,
                        download_url: file_row.url.clone(),
                        issues: file_issues,
                    });
                }
            }

            version_reports.push(VersionReport {
                version_id: VersionId::from(version_query.inner.id),
                files: file_reports,
            });
        }

        let max_severity = version_reports
            .iter()
            .flat_map(|vr| vr.files.iter())
            .map(|fr| fr.severity)
            .max();
        let project_report = ProjectReport {
            project_id: ProjectId::from(*project_id),
            max_severity,
            versions: version_reports,
        };

        project_reports.push(project_report);
    }

    Ok(project_reports)
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

    let replied_to_filter = search_req.filter.replied_to.map(|r| match r {
        RepliedTo::Replied => "replied",
        RepliedTo::Unreplied => "unreplied",
    });

    let mut project_ids = Vec::<DBProjectId>::new();
    let mut thread_ids = Vec::<DBThreadId>::new();

    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT ON (m.id)
            m.id AS "project_id: DBProjectId",
            t.id AS "thread_id: DBThreadId"
        FROM mods m
        INNER JOIN threads t ON t.mod_id = m.id
        INNER JOIN versions v ON v.mod_id = m.id
        INNER JOIN files f ON f.version_id = v.id
        INNER JOIN delphi_reports dr ON dr.file_id = f.id
        INNER JOIN delphi_report_issues dri ON dri.report_id = dr.id
        INNER JOIN delphi_report_issue_details drid
            ON drid.issue_id = dri.id
        LEFT JOIN delphi_issue_detail_verdicts didv
            ON m.id = didv.project_id AND drid.key = didv.detail_key
        LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
        LEFT JOIN categories c ON c.id = mc.joining_category_id
        LEFT JOIN threads_messages tm_last
            ON tm_last.thread_id = t.id
            AND tm_last.id = (
                SELECT id FROM threads_messages
                WHERE thread_id = t.id
                ORDER BY created DESC
                LIMIT 1
            )
        LEFT JOIN users u_last
            ON u_last.id = tm_last.author_id
        WHERE
            (cardinality($4::int[]) = 0 OR c.project_type = ANY($4::int[]))
            AND m.status NOT IN ('draft', 'rejected', 'withheld')
            AND (cardinality($6::text[]) = 0 OR m.status = ANY($6::text[]))
            AND (cardinality($7::text[]) = 0 OR dri.issue_type = ANY($7::text[]))
            AND (didv.verdict IS NULL OR didv.verdict = 'pending'::delphi_report_issue_status)
            AND (
                $5::text IS NULL
                OR ($5::text = 'unreplied' AND (tm_last.id IS NULL OR u_last.role IS NULL OR u_last.role NOT IN ('moderator', 'admin')))
                OR ($5::text = 'replied' AND tm_last.id IS NOT NULL AND u_last.role IS NOT NULL AND u_last.role IN ('moderator', 'admin'))
            )
        GROUP BY m.id, t.id
        ORDER BY m.id,
            CASE WHEN $3 = 'created_asc'   THEN MIN(dr.created) ELSE TO_TIMESTAMP(0)        END ASC,
            CASE WHEN $3 = 'created_desc'  THEN MAX(dr.created)   ELSE TO_TIMESTAMP(0)        END DESC,
            CASE WHEN $3 = 'severity_asc'  THEN MAX(dr.severity)  ELSE 'low'::delphi_severity END ASC,
            CASE WHEN $3 = 'severity_desc' THEN MAX(dr.severity)  ELSE 'low'::delphi_severity END DESC
        LIMIT $1 OFFSET $2
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
        replied_to_filter.as_deref(),
        &search_req
            .filter
            .project_status
            .iter()
            .map(|status| status.to_string())
            .collect::<Vec<_>>(),
        &search_req
            .filter
            .issue_type
    )
    .fetch_all(&**pool)
    .await
    .wrap_internal_err("failed to fetch projects")?;

    for row in rows {
        project_ids.push(row.project_id);
        thread_ids.push(row.thread_id);
    }

    let project_reports =
        fetch_project_reports(&project_ids, &pool, &redis).await?;

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
        .flat_map(|thread| {
            thread
                .messages
                .iter()
                .filter_map(|message| message.author_id)
        })
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
        project_reports,
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

/// Gets the technical review report for a specific project.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = OK, body = inline(ProjectReportResponse)))
)]
#[get("/project/{id}")]
async fn get_project_report(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(ProjectId,)>,
) -> Result<web::Json<ProjectReportResponse>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let (project_id,) = path.into_inner();
    let db_project_id = DBProjectId::from(project_id);

    let row = sqlx::query!(
        r#"
        SELECT t.id AS "thread_id: DBThreadId"
        FROM threads t
        WHERE t.mod_id = $1
        "#,
        db_project_id as _,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to fetch thread")?
    .ok_or(ApiError::NotFound)?;

    let project_reports =
        fetch_project_reports(&[db_project_id], &pool, &redis).await?;

    let project_report = project_reports.into_iter().next();

    let db_threads = DBThread::get_many(&[row.thread_id], &**pool)
        .await
        .wrap_internal_err("failed to fetch thread")?;
    let thread_author_ids = db_threads
        .iter()
        .flat_map(|thread| {
            thread
                .messages
                .iter()
                .filter_map(|message| message.author_id)
        })
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

    let thread = threads
        .get(&row.thread_id.into())
        .cloned()
        .ok_or(ApiError::NotFound)?;

    Ok(web::Json(ProjectReportResponse {
        project_report,
        thread,
    }))
}

/// See [`submit_report`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SubmitReport {
    /// Does the moderator think this report shows that the project is safe or
    /// unsafe?
    pub verdict: DelphiVerdict,
    /// Moderator message to send to the thread when rejecting the project.
    pub message: Option<String>,
}

/// Submits a verdict for a project based on its technical reports.
///
/// Before this is called, all issues for this project's reports must have been
/// marked as either safe or unsafe. Otherwise, this will error with
/// [`ApiError::TechReviewIssuesWithNoVerdict`], providing the issue IDs which
/// are still unmarked.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = NO_CONTENT))
)]
#[post("/submit/{project_id}")]
async fn submit_report(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(submit_report): web::Json<SubmitReport>,
    path: web::Path<(ProjectId,)>,
) -> Result<(), ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (project_id,) = path.into_inner();
    let project_id = DBProjectId::from(project_id);

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let pending_issue_details = sqlx::query!(
        r#"
        SELECT
            didws.id AS "issue_detail_id!"
        FROM mods m
        INNER JOIN versions v ON v.mod_id = m.id
        INNER JOIN files f ON f.version_id = v.id
        INNER JOIN delphi_reports dr ON dr.file_id = f.id
        INNER JOIN delphi_report_issues dri ON dri.report_id = dr.id
        INNER JOIN delphi_issue_details_with_statuses didws ON didws.issue_id = dri.id
        WHERE
            m.id = $1
            AND didws.status = 'pending'
            -- see delphi.rs todo comment
            AND dri.issue_type != '__dummy'
        "#,
        project_id as _,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch pending issues")?;

    if !pending_issue_details.is_empty() {
        return Err(ApiError::TechReviewDetailsWithNoVerdict {
            details: pending_issue_details
                .into_iter()
                .map(|record| {
                    DelphiReportIssueDetailsId(record.issue_detail_id)
                })
                .collect(),
        });
    }

    sqlx::query!(
        "
        DELETE FROM delphi_report_issue_details drid
        WHERE issue_id IN (
            SELECT dri.id
            FROM mods m
            INNER JOIN versions v ON v.mod_id = m.id
            INNER JOIN files f ON f.version_id = v.id
            INNER JOIN delphi_reports dr ON dr.file_id = f.id
            INNER JOIN delphi_report_issues dri ON dri.report_id = dr.id
            WHERE m.id = $1 AND dri.issue_type = '__dummy'
        )
        ",
        project_id as _,
    )
    .execute(&mut *txn)
    .await
    .wrap_internal_err("failed to delete dummy issue")?;

    let record = sqlx::query!(
        r#"
        SELECT t.id AS "thread_id: DBThreadId"
        FROM mods m
        INNER JOIN threads t ON t.mod_id = m.id
        WHERE m.id = $1
        "#,
        project_id as _,
    )
    .fetch_one(&mut *txn)
    .await
    .wrap_internal_err("failed to update reports")?;

    if let Some(body) = submit_report.message {
        ThreadMessageBuilder {
            author_id: Some(user.id.into()),
            body: MessageBody::Text {
                body,
                private: true,
                replying_to: None,
                associated_images: Vec::new(),
            },
            thread_id: record.thread_id,
            hide_identity: user.role.is_mod(),
        }
        .insert(&mut txn)
        .await
        .wrap_internal_err("failed to add moderator message")?;
    }

    let verdict = submit_report.verdict;
    ThreadMessageBuilder {
        author_id: Some(user.id.into()),
        body: MessageBody::TechReview { verdict },
        thread_id: record.thread_id,
        hide_identity: user.role.is_mod(),
    }
    .insert(&mut txn)
    .await
    .wrap_internal_err("failed to add tech review message")?;

    if verdict == DelphiVerdict::Unsafe {
        let record = sqlx::query!(
            r#"
            WITH old AS (
              SELECT id, status
              FROM mods
              WHERE id = $2
            ),
            updated AS (
              UPDATE mods
              SET status = $1
              FROM threads t
              WHERE
                mods.id = $2
                AND t.mod_id = mods.id
              RETURNING mods.id, t.id AS thread_id
            )
            SELECT
              updated.thread_id AS "thread_id: DBThreadId",
              old.status AS "old_status!"
            FROM updated
            INNER JOIN old ON old.id = updated.id
            "#,
            ProjectStatus::Rejected.as_str(),
            project_id as _,
        )
        .fetch_one(&mut *txn)
        .await
        .wrap_internal_err("failed to mark project as rejected")?;

        ThreadMessageBuilder {
            author_id: Some(user.id.into()),
            body: MessageBody::StatusChange {
                new_status: ProjectStatus::Rejected,
                old_status: ProjectStatus::from_string(&record.old_status),
            },
            thread_id: record.thread_id,
            hide_identity: user.role.is_mod(),
        }
        .insert(&mut txn)
        .await
        .wrap_internal_err("failed to add tech review message")?;

        DBProject::clear_cache(project_id, None, None, &redis)
            .await
            .wrap_internal_err("failed to clear project cache")?;
    }

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}

/// See [`update_issue`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateIssue {
    /// What the moderator has decided the outcome of this issue is.
    pub verdict: DelphiVerdict,
}

/// Updates the state of a technical review issue detail.
///
/// This will not automatically reject the project for malware, but just flag
/// this issue with a verdict.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = NO_CONTENT))
)]
#[patch("/issue-detail/{id}")]
async fn update_issue_detail(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    update_req: web::Json<UpdateIssue>,
    path: web::Path<(DelphiReportIssueDetailsId,)>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (issue_detail_id,) = path.into_inner();

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to start transaction")?;

    let status = match update_req.verdict {
        DelphiVerdict::Safe => DelphiStatus::Safe,
        DelphiVerdict::Unsafe => DelphiStatus::Unsafe,
    };
    let results = sqlx::query!(
        r#"
        INSERT INTO delphi_issue_detail_verdicts (
            project_id,
            detail_key,
            verdict
        )
        SELECT
            didws.project_id,
            didws.key,
            $1
        FROM delphi_issue_details_with_statuses didws
        INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
        WHERE
            didws.id = $2
            -- see delphi.rs todo comment
            AND dri.issue_type != '__dummy'
        ON CONFLICT (project_id, detail_key)
        DO UPDATE SET verdict = EXCLUDED.verdict
        "#,
        status as _,
        issue_detail_id as _,
    )
    .execute(&mut *txn)
    .await
    .wrap_internal_err("failed to update issue detail")?;
    if results.rows_affected() == 0 {
        return Err(ApiError::Request(eyre!("issue detail does not exist")));
    }

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}

/// See [`add_report`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AddReport {
    pub file_id: FileId,
}

/// Adds a file to the technical review queue by adding an empty report, if one
/// does not already exist for it.
#[utoipa::path]
#[put("/report")]
async fn add_report(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(add_report): web::Json<AddReport>,
) -> Result<web::Json<DelphiReportId>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let file_id = add_report.file_id;

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let record = sqlx::query!(
        r#"
        SELECT
            f.url,
            COUNT(dr.id) AS "report_count!"
        FROM files f
        LEFT JOIN delphi_reports dr ON dr.file_id = f.id
        WHERE f.id = $1
        GROUP BY f.url
        "#,
        DBFileId::from(file_id) as _,
    )
    .fetch_one(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch file")?;

    if record.report_count > 0 {
        return Err(ApiError::Request(eyre!("file already has reports")));
    }

    let report_id = DBDelphiReport {
        id: DelphiReportId(0),
        file_id: Some(file_id.into()),
        delphi_version: -1, // TODO
        artifact_url: record.url,
        created: Utc::now(),
        severity: DelphiSeverity::Low, // TODO
    }
    .upsert(&mut txn)
    .await
    .wrap_internal_err("failed to insert report")?;

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(web::Json(report_id))
}
