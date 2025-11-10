use std::collections::HashMap;

use actix_web::{HttpRequest, get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio_stream::StreamExt;

use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        models::{
            DelphiReportId, DelphiReportIssueDetailsId, DelphiReportIssueId,
            ProjectTypeId, categories::ProjectType,
            delphi_report_item::DelphiSeverity,
        },
        redis::RedisPool,
    },
    models::{pats::Scopes, projects::Project, threads::Thread},
    queue::session::AuthQueue,
    routes::{ApiError, internal::moderation::ProjectsRequestOptions},
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(search_projects);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchProjects {
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
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
    SearchProjectsSort::Oldest
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
pub enum SearchProjectsSort {
    Oldest,
    Newest,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectReview {
    pub project: Project,
    pub project_owner: (),
    pub thread: Thread,
    /// Why this project was flagged.
    pub flag_reason: FlagReason,
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
    pub file_size: u64,
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
    /// How important is this issue, as flagged by Delphi?
    pub severity: DelphiSeverity,
    /// Details of why this issue might have been raised, such as what file it
    /// was found in.
    pub details: Vec<FileIssueDetails>,
}

/// Occurrence of a [`FileIssue`] in a specific class in a scanned JAR file.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileIssueDetails {
    /// Name of the Java class in which this issue was found.
    pub class_name: String,
    /// Decompiled, pretty-printed source of the Java class.
    pub decompiled_source: String,
}

/// Searches all projects which are awaiting technical review.
#[utoipa::path]
#[post("/search")]
async fn search_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_req: web::Json<SearchProjects>,
) -> Result<web::Json<Vec<ProjectReview>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let sort_by = match search_req.sort_by {
        SearchProjectsSort::Oldest => 0,
        SearchProjectsSort::Newest => 1,
    };
    let limit = search_req.limit.max(50);
    let offset = limit * search_req.page;

    let limit =
        i64::try_from(limit).wrap_request_err("limit cannot fit into `i64`")?;
    let offset = i64::try_from(offset)
        .wrap_request_err("offset cannot fit into `i64`")?;

    let mut reports = Vec::new();
    let mut project_ids = Vec::new();

    let mut rows = sqlx::query!(
        r#"
        SELECT
            dr.id AS report_id,
            m.id AS project_id,
            dr.created AS report_created,
            dri.issue_type AS issue_type,
            drid.internal_class_name AS issue_detail_class_name,
            drid.decompiled_source AS issue_detail_decompiled_source,
            drid.severity AS "issue_detail_severity: DelphiSeverity"
        FROM delphi_reports dr

        -- fetch the project this report is for, and its type
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m ON m.id = v.mod_id
        LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
        INNER JOIN categories c ON c.id = mc.joining_category_id

        -- fetch report issues and details
        LEFT JOIN delphi_report_issues dri ON dri.report_id = dr.id
        LEFT JOIN delphi_report_issue_details drid ON drid.issue_id = dri.id

        -- filtering
        WHERE
            -- project type
            (cardinality($1::int[]) = 0 OR c.project_type = ANY($1::int[]))

        -- sorting
        ORDER BY
            CASE WHEN $2 = '
                -- when sorting on TIMESTAMPTZ columns, we extract the int value of the time
                -- so that we can sort by an integer, which we can negate
                -- (we can't negate a TIMESTAMPTZ)

                -- oldest
                WHEN $2 = 0 THEN EXTRACT(EPOCH FROM created)
                -- newest
                WHEN $2 = 1 THEN -EXTRACT(EPOCH FROM created)
            END

        -- pagination
        LIMIT $3
        OFFSET $4
        "#,
        &search_req.filter.project_type.iter().map(|ty| ty.0).collect::<Vec<_>>(),
        sort_by,
        limit,
        offset,
    )
    .fetch(&**pool);
    while let Some(row) = rows
        .next()
        .await
        .transpose()
        .wrap_internal_err("failed to fetch reports")
    {
        project_ids.push(row.project_id);
        reports.push(ProjectReview {
            project: (),
            project_owner: (),
        });
    }

    Ok(())
}
