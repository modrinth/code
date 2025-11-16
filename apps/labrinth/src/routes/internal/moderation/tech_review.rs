use std::{collections::HashMap, fmt};

use actix_web::{HttpRequest, post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio_stream::StreamExt;

use super::ownership::get_projects_ownership;
use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        DBProject,
        models::{
            DBFileId, DBProjectId, DBThread, DBThreadId, DelphiReportId,
            DelphiReportIssueDetailsId, DelphiReportIssueId, ProjectTypeId,
            delphi_report_item::{DelphiReportIssueStatus, DelphiSeverity},
        },
        redis::RedisPool,
    },
    models::{pats::Scopes, projects::Project},
    queue::session::AuthQueue,
    routes::{ApiError, internal::moderation::Ownership},
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(search_projects).service(update_issue);
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
pub struct ProjectReview {
    pub project: Project,
    pub project_owner: Ownership,
    pub thread: DBThread,
    pub reports: Vec<ProjectReport>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectReport {
    /// When this report was created.
    pub created_at: DateTime<Utc>,
    /// Why this project was flagged.
    pub flag_reason: FlagReason,
    /// According to this report, how likely is the project malicious?
    pub severity: DelphiSeverity,
    /// What files were flagged in this review.
    pub files: Vec<FileReview>,
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

/// Details of a JAR file which was flagged for technical review, as part of
/// a [`ProjectReview`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileReview {
    /// Name of the flagged file.
    pub file_name: String,
    /// Size of the flagged file, in bytes.
    pub file_size: i32,
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
    pub issue_id: DelphiReportIssueId,
    /// Delphi-determined kind of issue that this is, e.g. `OBFUSCATED_NAMES`.
    ///
    /// Labrinth does not know the full set of kinds of issues, so this is kept
    /// as a string.
    pub kind: String,
    /// Is this issue valid (malicious) or a false positive (safe)?
    pub status: DelphiReportIssueStatus,
    /// Details of why this issue might have been raised, such as what file it
    /// was found in.
    pub details: Vec<FileIssueDetail>,
}

/// Occurrence of a [`FileIssue`] in a specific class in a scanned JAR file.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileIssueDetail {
    /// Name of the Java class path in which this issue was found.
    pub file_path: String,
    /// Decompiled, pretty-printed source of the Java class.
    pub decompiled_source: String,
    /// How important is this issue, as flagged by Delphi?
    pub severity: DelphiSeverity,
}

/// Searches all projects which are awaiting technical review.
#[utoipa::path(
    security(("bearer_auth" = [])),
    responses((status = OK, body = inline(Vec<ProjectReview>)))
)]
#[post("/search")]
async fn search_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_req: web::Json<SearchProjects>,
) -> Result<web::Json<Vec<ProjectReview>>, ApiError> {
    #[derive(Debug)]
    struct ProjectRecord {
        reports: IndexMap<DelphiReportId, ReportRecord>,
    }

    #[derive(Debug)]
    struct ReportRecord {
        created: DateTime<Utc>,
        severity: DelphiSeverity,
        files: IndexMap<DBFileId, FileRecord>,
    }

    #[derive(Debug)]
    struct FileRecord {
        file_name: String,
        file_size: i32,
        issues: IndexMap<DelphiReportIssueId, IssueRecord>,
    }

    #[derive(Debug)]
    struct IssueRecord {
        issue_type: String,
        status: DelphiReportIssueStatus,
        details: IndexMap<DelphiReportIssueDetailsId, FileIssueDetail>,
    }

    check_is_moderator_from_headers(
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

    let mut project_records = IndexMap::<DBProjectId, ProjectRecord>::new();
    let mut project_ids = Vec::<DBProjectId>::new();
    let mut thread_ids = Vec::<DBThreadId>::new();
    let _file_ids = Vec::<DBFileId>::new();

    let mut rows = sqlx::query!(
        r#"
        SELECT
            dr.id AS "report_id!: DelphiReportId",
            f.id AS "file_id!: DBFileId",
            f.filename AS "file_name!",
            f.size AS "file_size!",
            m.id AS "project_id!: DBProjectId",
            t.id AS "project_thread_id!: DBThreadId",
            dr.created AS "report_created!",
            dr.severity AS "report_severity!: DelphiSeverity",
            dri.id AS "issue_id!: DelphiReportIssueId",
            dri.issue_type AS "issue_type!",
            dri.status AS "issue_status!: DelphiReportIssueStatus",
            -- maybe null
            drid.id AS "issue_detail_id?: DelphiReportIssueDetailsId",
            drid.file_path AS "issue_detail_file_path?",
            drid.decompiled_source AS "issue_detail_decompiled_source?",
            drid.severity AS "issue_detail_severity?: DelphiSeverity"
        FROM delphi_reports dr

        -- fetch the project this report is for, its type, and thread
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m ON m.id = v.mod_id
        LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
        LEFT JOIN categories c ON c.id = mc.joining_category_id
        INNER JOIN threads t ON t.mod_id = m.id
        -- fetch report issues and details
        INNER JOIN delphi_report_issues dri ON dri.report_id = dr.id
        LEFT JOIN delphi_report_issue_details drid ON drid.issue_id = dri.id

        -- filtering
        WHERE
            -- project type
            (cardinality($1::int[]) = 0 OR c.project_type = ANY($1::int[]))

        -- sorting
        ORDER BY
            CASE WHEN $2 = 'created_asc' THEN created ELSE TO_TIMESTAMP(0) END ASC,
            CASE WHEN $2 = 'created_desc' THEN created ELSE TO_TIMESTAMP(0) END DESC,
            CASE WHEN $2 = 'severity_asc' THEN dr.severity ELSE 'low'::delphi_severity END ASC,
            CASE WHEN $2 = 'severity_desc' THEN dr.severity ELSE 'low'::delphi_severity END DESC

        -- pagination
        LIMIT $3
        OFFSET $4
        "#,
        &search_req
            .filter
            .project_type
            .iter()
            .map(|ty| ty.0)
            .collect::<Vec<_>>(),
        &sort_by,
        limit,
        offset,
    )
    .fetch(&**pool);

    while let Some(row) = rows
        .next()
        .await
        .transpose()
        .wrap_internal_err("failed to fetch reports")?
    {
        project_ids.push(row.project_id);
        thread_ids.push(row.project_thread_id);

        let project =
            project_records.entry(row.project_id).or_insert_with(|| {
                ProjectRecord {
                    reports: IndexMap::new(),
                }
            });
        let report =
            project.reports.entry(row.report_id).or_insert_with(|| {
                ReportRecord {
                    created: row.report_created,
                    severity: row.report_severity,
                    files: IndexMap::new(),
                }
            });
        let file =
            report
                .files
                .entry(row.file_id)
                .or_insert_with(|| FileRecord {
                    file_name: row.file_name,
                    file_size: row.file_size,
                    issues: IndexMap::new(),
                });
        let issue =
            file.issues
                .entry(row.issue_id)
                .or_insert_with(|| IssueRecord {
                    issue_type: row.issue_type,
                    status: row.issue_status,
                    details: IndexMap::new(),
                });

        let (
            Some(issue_detail_id),
            Some(file_path),
            Some(decompiled_source),
            Some(severity),
        ) = (
            row.issue_detail_id,
            row.issue_detail_file_path,
            row.issue_detail_decompiled_source,
            row.issue_detail_severity,
        )
        else {
            continue;
        };
        issue.details.entry(issue_detail_id).or_insert_with(|| {
            FileIssueDetail {
                file_path,
                decompiled_source,
                severity,
            }
        });
    }

    let projects = DBProject::get_many_ids(&project_ids, &**pool, &redis)
        .await
        .wrap_internal_err("failed to fetch projects")?
        .into_iter()
        .map(|project| (project.inner.id, Project::from(project)))
        .collect::<HashMap<_, _>>();
    let threads = DBThread::get_many(&thread_ids, &**pool)
        .await
        .wrap_internal_err("failed to fetch threads")?
        .into_iter()
        .map(|thread| (thread.id, thread))
        .collect::<HashMap<_, _>>();

    let project_list: Vec<Project> = projects.values().cloned().collect();

    let ownerships = get_projects_ownership(&project_list, &pool, &redis)
        .await
        .wrap_internal_err("failed to fetch project ownerships")?;

    let ownership_map = projects
        .keys()
        .copied()
        .zip(ownerships)
        .collect::<HashMap<_, _>>();

    let projects = project_records
        .into_iter()
        .map(|(project_id, project_record)| {
            let project =
                projects.get(&project_id).wrap_internal_err_with(|| {
                    eyre!("no fetched project with ID {project_id:?}")
                })?;
            let thread = threads
                .get(&DBThreadId::from(project.thread_id))
                .wrap_internal_err_with(|| {
                    eyre!("no fetched thread with ID {:?}", project.thread_id)
                })?;
            Ok::<_, ApiError>(ProjectReview {
                project: project.clone(),
                project_owner: ownership_map
                    .get(&project_id)
                    .cloned()
                    .wrap_internal_err_with(|| {
                        eyre!("no owner for {project_id:?}")
                    })?,
                thread: thread.clone(),
                reports: project_record
                    .reports
                    .into_iter()
                    .map(|(_, report_record)| ProjectReport {
                        created_at: report_record.created,
                        flag_reason: FlagReason::Delphi,
                        severity: report_record.severity,
                        files: report_record
                            .files
                            .into_iter()
                            .map(|(_, file)| FileReview {
                                file_name: file.file_name,
                                file_size: file.file_size,
                                issues: file
                                    .issues
                                    .into_iter()
                                    .map(|(issue_id, issue)| FileIssue {
                                        issue_id,
                                        kind: issue.issue_type.clone(),
                                        status: issue.status,
                                        details: issue
                                            .details
                                            .into_iter()
                                            .map(|(_, detail)| {
                                                FileIssueDetail {
                                                    file_path: detail.file_path,
                                                    decompiled_source: detail
                                                        .decompiled_source,
                                                    severity: detail.severity,
                                                }
                                            })
                                            .collect(),
                                    })
                                    .collect(),
                            })
                            .collect(),
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(web::Json(projects))
}

/// See [`update_issue`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateIssue {
    /// Status to set the issue to.
    pub status: DelphiReportIssueStatus,
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
    update_req: web::Json<UpdateIssue>,
    path: web::Path<(DelphiReportIssueId,)>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;
    let (issue_id,) = path.into_inner();

    sqlx::query!(
        "
        UPDATE delphi_report_issues
        SET status = $1
        WHERE id = $2
        ",
        update_req.status as DelphiReportIssueStatus,
        issue_id as DelphiReportIssueId,
    )
    .execute(&**pool)
    .await
    .wrap_internal_err("failed to update issue")?;

    Ok(())
}
