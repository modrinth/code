use actix_web::{HttpRequest, post, web};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    auth::check_is_moderator_from_headers,
    database::{
        PgPool,
        models::{
            DBFileId, DBProjectId, DBVersionId, DelphiReportIssueDetailsId,
            DelphiReportIssueId,
            delphi_report_item::{DelphiSeverity, DelphiStatus},
        },
        redis::RedisPool,
    },
    models::{
        ids::{FileId, ProjectId, VersionId},
        pats::Scopes,
    },
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context,
};
use eyre::eyre;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(search_global_issue_details)
        .service(get_global_issue_detail);
}

fn default_limit() -> u64 {
    20
}

const LOCAL_TRACE_PREVIEW_LIMIT: i64 = 10;

/// Arguments for searching globally-verdict'ed Delphi detail traces.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchGlobalIssueDetails {
    #[serde(default = "default_limit")]
    #[schema(default = 20)]
    pub limit: u64,
    #[serde(default)]
    #[schema(default = 0)]
    pub page: u64,
    #[serde(default)]
    pub query: Option<String>,
}

/// Response for searching globally-verdict'ed Delphi detail traces.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchGlobalIssueDetailsResponse {
    /// Total number of matching global verdicts.
    pub total: i64,
    /// Globally-verdict'ed detail keys and their local matches.
    pub traces: Vec<GlobalIssueDetail>,
}

/// Arguments for fetching one globally-verdict'ed Delphi detail trace.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GetGlobalIssueDetail {
    /// Key used by Delphi to identify matching detail traces.
    pub detail_key: String,
    #[serde(default = "default_limit")]
    #[schema(default = 20)]
    pub limit: u64,
    /// Return local traces with IDs greater than this detail ID.
    #[serde(default)]
    pub after_detail_id: Option<DelphiReportIssueDetailsId>,
}

/// Response for one globally-verdict'ed Delphi detail trace.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GetGlobalIssueDetailResponse {
    /// Globally-verdict'ed detail key and a page of local matches.
    pub trace: GlobalIssueDetail,
    /// Pass this as `after_detail_id` to fetch the next page.
    pub next_after_detail_id: Option<DelphiReportIssueDetailsId>,
}

/// A globally-verdict'ed Delphi detail key.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GlobalIssueDetail {
    /// Key used by Delphi to identify matching detail traces.
    pub detail_key: String,
    /// Verdict applied to every matching detail trace.
    pub verdict: DelphiStatus,
    /// Number of local detail traces with this key.
    pub local_trace_count: i64,
    /// Local detail traces matching this key.
    pub local_traces: Vec<GlobalIssueDetailTrace>,
}

/// A local Delphi detail trace matching a globally-verdict'ed key.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GlobalIssueDetailTrace {
    /// ID of the local detail trace.
    pub detail_id: DelphiReportIssueDetailsId,
    /// ID of the issue containing this detail trace.
    pub issue_id: DelphiReportIssueId,
    /// Delphi issue kind.
    pub issue_type: String,
    /// ID of the project containing this detail trace.
    pub project_id: ProjectId,
    /// Project slug, when one is set.
    pub project_slug: Option<String>,
    /// Project name.
    pub project_name: String,
    /// ID of the version containing this detail trace.
    pub version_id: VersionId,
    /// Version number.
    pub version_number: String,
    /// ID of the file containing this detail trace.
    pub file_id: FileId,
    /// File name.
    pub file_name: String,
    /// JAR containing the detail trace, when Delphi reported one.
    pub jar: Option<String>,
    /// File path containing the detail trace.
    pub file_path: String,
    /// Delphi severity for this detail trace.
    pub severity: DelphiSeverity,
    /// Project-local verdict for this key.
    pub local_status: DelphiStatus,
    /// Effective verdict after applying global fallback rules.
    pub effective_status: DelphiStatus,
}

/// Search globally-verdict'ed Delphi detail traces.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
    security(("bearer_auth" = [])),
	responses((status = OK, body = SearchGlobalIssueDetailsResponse))
)]
#[post("/global-issue-detail/search")]
pub async fn search_global_issue_details(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_req: web::Json<SearchGlobalIssueDetails>,
) -> Result<web::Json<SearchGlobalIssueDetailsResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let query = search_req
        .query
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty());
    let limit = search_req.limit.clamp(1, 100);
    let offset = limit.saturating_mul(search_req.page);

    let limit =
        i64::try_from(limit).wrap_request_err("limit cannot fit into `i64`")?;
    let offset = i64::try_from(offset)
        .wrap_request_err("offset cannot fit into `i64`")?;

    let total = sqlx::query!(
        r#"
        SELECT COUNT(*) AS "total!"
        FROM delphi_global_detail_verdicts dgdv
        WHERE (
            $1::text IS NULL
            OR dgdv.detail_key ILIKE '%' || $1 || '%'
        )
        "#,
        query,
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("failed to count global issue details")?
    .total;

    let global_rows = sqlx::query!(
        r#"
        SELECT
            dgdv.detail_key,
            dgdv.verdict AS "verdict!: DelphiStatus",
            COUNT(dri.id) AS "local_trace_count!"
        FROM delphi_global_detail_verdicts dgdv
        LEFT JOIN delphi_issue_details_with_statuses didws
            ON didws.key = dgdv.detail_key
            AND NOT didws.hidden
        LEFT JOIN delphi_report_issues dri
            ON dri.id = didws.issue_id
            AND dri.issue_type != '__dummy'
        WHERE (
            $1::text IS NULL
            OR dgdv.detail_key ILIKE '%' || $1 || '%'
        )
        GROUP BY dgdv.detail_key, dgdv.verdict
        ORDER BY dgdv.detail_key
        LIMIT $2 OFFSET $3
        "#,
        query,
        limit,
        offset,
    )
    .fetch_all(&**pool)
    .await
    .wrap_internal_err("failed to fetch global issue details")?;

    let detail_keys = global_rows
        .iter()
        .map(|row| row.detail_key.clone())
        .collect::<Vec<_>>();

    let local_rows = sqlx::query!(
        r#"
        WITH ranked_local_traces AS (
            SELECT
                didws.key AS detail_key,
                didws.id AS detail_id,
                didws.issue_id,
                dri.issue_type,
                m.id AS project_id,
                m.slug AS project_slug,
                m.name AS project_name,
                v.id AS version_id,
                v.version_number,
                f.id AS file_id,
                f.filename AS file_name,
                didws.jar,
                didws.file_path,
                didws.severity,
                COALESCE(didv.verdict, 'pending'::delphi_report_issue_status)
                    AS local_status,
                didws.status AS effective_status,
                ROW_NUMBER() OVER (
                    PARTITION BY didws.key
                    ORDER BY didws.id
                ) AS row_num
            FROM delphi_issue_details_with_statuses didws
            INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
            INNER JOIN delphi_reports dr ON dr.id = dri.report_id
            INNER JOIN files f ON f.id = dr.file_id
            INNER JOIN versions v ON v.id = f.version_id
            INNER JOIN mods m ON m.id = v.mod_id
            LEFT JOIN delphi_issue_detail_verdicts didv
                ON didv.project_id = didws.project_id
                AND didv.detail_key = didws.key
            WHERE
                didws.key = ANY($1::text[])
                AND NOT didws.hidden
                AND dri.issue_type != '__dummy'
        )
        SELECT
            detail_key AS "detail_key!",
            detail_id AS "detail_id!: DelphiReportIssueDetailsId",
            issue_id AS "issue_id!: DelphiReportIssueId",
            issue_type,
            project_id AS "project_id!: DBProjectId",
            project_slug AS "project_slug?",
            project_name AS "project_name!",
            version_id AS "version_id!: DBVersionId",
            v.version_number,
            file_id AS "file_id!: DBFileId",
            file_name AS "file_name!",
            jar AS "jar?",
            file_path AS "file_path!",
            severity AS "severity!: DelphiSeverity",
            local_status AS "local_status!: DelphiStatus",
            effective_status AS "effective_status!: DelphiStatus"
        FROM ranked_local_traces v
        WHERE row_num <= $2
        ORDER BY detail_key, detail_id
        "#,
        &detail_keys,
        LOCAL_TRACE_PREVIEW_LIMIT,
    )
    .fetch_all(&**pool)
    .await
    .wrap_internal_err("failed to fetch local issue detail traces")?;

    let mut traces_by_key = local_rows
        .into_iter()
        .map(|row| {
            (
                row.detail_key,
                GlobalIssueDetailTrace {
                    detail_id: row.detail_id,
                    issue_id: row.issue_id,
                    issue_type: row.issue_type,
                    project_id: ProjectId::from(row.project_id),
                    project_slug: row.project_slug,
                    project_name: row.project_name,
                    version_id: VersionId::from(row.version_id),
                    version_number: row.version_number,
                    file_id: FileId::from(row.file_id),
                    file_name: row.file_name,
                    jar: row.jar,
                    file_path: row.file_path,
                    severity: row.severity,
                    local_status: row.local_status,
                    effective_status: row.effective_status,
                },
            )
        })
        .into_group_map();

    let traces = global_rows
        .into_iter()
        .map(|row| GlobalIssueDetail {
            local_traces: traces_by_key
                .remove(&row.detail_key)
                .unwrap_or_default(),
            detail_key: row.detail_key,
            verdict: row.verdict,
            local_trace_count: row.local_trace_count,
        })
        .collect();

    Ok(web::Json(SearchGlobalIssueDetailsResponse {
        total,
        traces,
    }))
}

/// Fetch one globally-verdict'ed Delphi detail trace with paginated local matches.
#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
    security(("bearer_auth" = [])),
	responses((status = OK, body = GetGlobalIssueDetailResponse))
)]
#[post("/global-issue-detail/local-traces")]
pub async fn get_global_issue_detail(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    get_req: web::Json<GetGlobalIssueDetail>,
) -> Result<web::Json<GetGlobalIssueDetailResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let detail_key = get_req.detail_key.trim();
    if detail_key.is_empty() {
        return Err(ApiError::Request(eyre!("detail key cannot be empty")));
    }

    let limit = get_req.limit.clamp(1, 100);
    let query_limit = i64::try_from(limit + 1)
        .wrap_request_err("limit cannot fit into `i64`")?;
    let limit = usize::try_from(limit)
        .wrap_request_err("limit cannot fit into `usize`")?;
    let after_detail_id = get_req.after_detail_id.map(|id| id.0);

    let global_row = sqlx::query!(
        r#"
        SELECT
            dgdv.detail_key,
            dgdv.verdict AS "verdict!: DelphiStatus",
            COUNT(dri.id) AS "local_trace_count!"
        FROM delphi_global_detail_verdicts dgdv
        LEFT JOIN delphi_issue_details_with_statuses didws
            ON didws.key = dgdv.detail_key
            AND NOT didws.hidden
        LEFT JOIN delphi_report_issues dri
            ON dri.id = didws.issue_id
            AND dri.issue_type != '__dummy'
        WHERE dgdv.detail_key = $1
        GROUP BY dgdv.detail_key, dgdv.verdict
        "#,
        detail_key,
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to fetch global issue detail")?
    .ok_or(ApiError::NotFound)?;

    let local_rows = sqlx::query!(
        r#"
        SELECT
            didws.key AS "detail_key!",
            didws.id AS "detail_id!: DelphiReportIssueDetailsId",
            didws.issue_id AS "issue_id!: DelphiReportIssueId",
            dri.issue_type,
            m.id AS "project_id!: DBProjectId",
            m.slug AS "project_slug?",
            m.name AS "project_name!",
            v.id AS "version_id!: DBVersionId",
            v.version_number,
            f.id AS "file_id!: DBFileId",
            f.filename AS "file_name!",
            didws.jar AS "jar?",
            didws.file_path AS "file_path!",
            didws.severity AS "severity!: DelphiSeverity",
            COALESCE(didv.verdict, 'pending'::delphi_report_issue_status)
                AS "local_status!: DelphiStatus",
            didws.status AS "effective_status!: DelphiStatus"
        FROM delphi_issue_details_with_statuses didws
        INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
        INNER JOIN delphi_reports dr ON dr.id = dri.report_id
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        INNER JOIN mods m ON m.id = v.mod_id
        LEFT JOIN delphi_issue_detail_verdicts didv
            ON didv.project_id = didws.project_id
            AND didv.detail_key = didws.key
        WHERE
            didws.key = $1
            AND ($2::bigint IS NULL OR didws.id > $2)
            AND NOT didws.hidden
            AND dri.issue_type != '__dummy'
        ORDER BY didws.id
        LIMIT $3
        "#,
        detail_key,
        after_detail_id,
        query_limit,
    )
    .fetch_all(&**pool)
    .await
    .wrap_internal_err("failed to fetch local issue detail traces")?;

    let mut local_traces = local_rows
        .into_iter()
        .map(|row| GlobalIssueDetailTrace {
            detail_id: row.detail_id,
            issue_id: row.issue_id,
            issue_type: row.issue_type,
            project_id: ProjectId::from(row.project_id),
            project_slug: row.project_slug,
            project_name: row.project_name,
            version_id: VersionId::from(row.version_id),
            version_number: row.version_number,
            file_id: FileId::from(row.file_id),
            file_name: row.file_name,
            jar: row.jar,
            file_path: row.file_path,
            severity: row.severity,
            local_status: row.local_status,
            effective_status: row.effective_status,
        })
        .collect::<Vec<_>>();

    let has_more = local_traces.len() > limit;
    if has_more {
        local_traces.pop();
    }
    let next_after_detail_id = has_more
        .then(|| local_traces.last().map(|trace| trace.detail_id))
        .flatten();

    Ok(web::Json(GetGlobalIssueDetailResponse {
        trace: GlobalIssueDetail {
            detail_key: global_row.detail_key,
            verdict: global_row.verdict,
            local_trace_count: global_row.local_trace_count,
            local_traces,
        },
        next_after_detail_id,
    }))
}
