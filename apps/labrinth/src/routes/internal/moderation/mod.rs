use super::ApiError;
use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::PgPool;
use crate::database::models::DBModerationLock;
use crate::database::models::moderation_external_item;
use crate::database::redis::RedisPool;
use crate::models::ids::{OrganizationId, ProjectId};
use crate::models::projects::{ProjectStatus, VersionStatus};
use crate::queue::moderation::{ApprovalType, IdentifiedFile, MissingMetadata};
use crate::queue::session::AuthQueue;
use crate::util::error::Context;
use crate::{
    auth::{check_is_moderator_from_headers, get_user_from_bearer_token},
    models::pats::Scopes,
};
use actix_web::{HttpRequest, delete, get, post, web};
use ariadne::ids::{UserId, random_base62};
use chrono::{DateTime, Utc};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod external_license;
mod ownership;
pub mod tech_review;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_projects)
        .service(get_project_ids)
        .service(get_project_meta)
        .service(set_project_meta)
        .service(acquire_lock)
        .service(override_lock)
        .service(get_lock_status)
        .service(release_lock)
        .service(release_lock_beacon)
        .service(delete_all_locks)
        .service(web::scope("/tech-review").configure(tech_review::config))
        .service(
            web::scope("/external-license").configure(external_license::config),
        );
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProjectsRequestOptions {
    /// How many projects to fetch.
    #[serde(default = "default_count")]
    pub count: u16,
    /// How many projects to skip.
    #[serde(default)]
    pub offset: u32,
    /// Whether to filter by modpacks that have external dependencies.
    #[serde(default)]
    pub has_external_dependencies: Option<bool>,
    /// Text query to search against project and owner fields.
    #[serde(default)]
    pub query: Option<String>,
    /// Project type to filter by. Use `none` for projects without a type.
    #[serde(default)]
    pub project_type: Option<String>,
    /// Sort order for the moderation queue.
    #[serde(default)]
    pub sort: Option<ModerationProjectsSort>,
}

fn default_count() -> u16 {
    100
}

const MAX_PROJECTS_PER_PAGE: u16 = 200;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ModerationProjectsSort {
    #[default]
    Oldest,
    Newest,
    MostExternalDeps,
    LeastExternalDeps,
}

impl ModerationProjectsSort {
    fn as_str(self) -> &'static str {
        match self {
            Self::Oldest => "oldest",
            Self::Newest => "newest",
            Self::MostExternalDeps => "most_external_deps",
            Self::LeastExternalDeps => "least_external_deps",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ModerationProjectsResponse {
    pub total: i64,
    pub projects: Vec<ModerationQueueProject>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ModerationProjectIdsResponse {
    pub ids: Vec<ProjectId>,
}

/// Lightweight project information for the moderation queue index.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ModerationQueueProject {
    pub id: ProjectId,
    pub slug: Option<String>,
    pub name: String,
    pub summary: String,
    pub icon_url: Option<String>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub queued: Option<DateTime<Utc>>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub project_types: Vec<String>,
    /// Who owns the project.
    pub ownership: Ownership,
    /// How many external file dependencies the project has.
    pub external_dependencies_count: i64,
}

/// Fetched information on who owns a project.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Ownership {
    /// Project is owned by a team, and this is the team owner.
    User {
        /// ID of the team owner.
        id: UserId,
        /// Name of the team owner.
        name: String,
        /// URL of the team owner's icon.
        icon_url: Option<String>,
    },
    /// Project is owned by an organization.
    Organization {
        /// ID of the organization.
        id: OrganizationId,
        /// Name of the organization.
        name: String,
        /// URL of the organization's icon.
        icon_url: Option<String>,
    },
}

/// Response for lock status check
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LockStatusResponse {
    /// Whether the project is currently locked
    pub locked: bool,
    /// Whether the requesting user holds the lock
    pub is_own_lock: bool,
    /// Information about who holds the lock (if locked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<LockedByUser>,
    /// When the lock was acquired
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    /// When the lock expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the lock has expired
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired: Option<bool>,
}

/// Information about the moderator holding the lock
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LockedByUser {
    /// User ID (base62 encoded)
    pub id: String,
    /// Username
    pub username: String,
    /// Avatar URL
    pub avatar_url: Option<String>,
}

/// Response for successful lock acquisition
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LockAcquireResponse {
    /// Whether lock was successfully acquired
    pub success: bool,
    /// Whether the requesting user holds the lock (true when success is true)
    pub is_own_lock: bool,
    /// If blocked, info about who holds the lock
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<LockedByUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    /// When the lock expires (present whether acquired or blocked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired: Option<bool>,
}

/// Response for lock release
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LockReleaseResponse {
    pub success: bool,
}

/// Response for deleting all locks
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeleteAllLocksResponse {
    pub deleted_count: u64,
}

/// List projects in the moderation queue.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    params(
		("count" = Option<u16>, Query),
		("offset" = Option<u32>, Query),
		("has_external_dependencies" = Option<bool>, Query),
		("query" = Option<String>, Query),
		("project_type" = Option<String>, Query),
		("sort" = Option<ModerationProjectsSort>, Query)
	),
    responses((status = OK, body = ModerationProjectsResponse))
)]
#[get("/projects")]
pub async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ModerationProjectsResponse>, ApiError> {
    get_projects_internal(req, pool, redis, request_opts, session_queue).await
}

pub async fn get_projects_internal(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ModerationProjectsResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let request_opts = request_opts.into_inner();
    let query = normalize_optional_string(request_opts.query.as_deref());
    let project_type =
        normalize_optional_string(request_opts.project_type.as_deref());
    let sort = request_opts.sort.unwrap_or_default().as_str();
    let listed_version_statuses = listed_version_statuses();
    let count = request_opts.count.min(MAX_PROJECTS_PER_PAGE) as i64;
    let offset = request_opts.offset as i64;
    let needs_filtered_queue = query.is_some()
        || project_type.is_some()
        || request_opts.has_external_dependencies.is_some()
        || matches!(sort, "most_external_deps" | "least_external_deps");

    let (total, projects) = if needs_filtered_queue {
        let project_rows = sqlx::query!(
            r#"
            WITH moderation_projects AS (
                SELECT
                    m.id,
                    m.slug,
                    m.name,
                    m.summary,
                    m.description,
                    m.queued,
                    m.published,
                    m.organization_id,
                    m.team_id,
                    m.components
                FROM mods m
                WHERE m.status = $1
            ),
            external_dependencies AS (
                SELECT
                    v.mod_id,
                    COUNT(*) AS external_dependencies_count
                FROM versions v
                INNER JOIN moderation_projects mp ON mp.id = v.mod_id
                INNER JOIN dependencies d ON d.dependent_id = v.id
                WHERE d.dependency_file_name IS NOT NULL
                GROUP BY v.mod_id
            ),
            version_project_types AS (
                SELECT
                    v.mod_id,
                    ARRAY_AGG(DISTINCT pt.name::text) FILTER (WHERE pt.name IS NOT NULL) AS project_types
                FROM versions v
                INNER JOIN moderation_projects mp ON mp.id = v.mod_id
                INNER JOIN loaders_versions lv ON v.id = lv.version_id
                INNER JOIN loaders l ON lv.loader_id = l.id
                INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                WHERE v.status = ANY($2)
                GROUP BY v.mod_id
            ),
            queue_projects AS (
                SELECT
                    mp.id,
                    mp.slug,
                    mp.name,
                    mp.summary,
                    mp.description,
                    mp.queued,
                    mp.published,
                    search_organization.name AS organization_name,
                    search_owner.username AS owner_name,
                    CASE
                        WHEN mp.components ? 'minecraft_server'
                            THEN ARRAY_APPEND(
                                ARRAY_REMOVE(
                                    COALESCE(vpt.project_types::text[], ARRAY[]::text[]),
                                    'modpack'
                                ),
                                'minecraft_java_server'
                            )
                        ELSE COALESCE(vpt.project_types::text[], ARRAY[]::text[])
                    END AS project_types,
                    COALESCE(ed.external_dependencies_count, 0) AS external_dependencies_count
                FROM moderation_projects mp
                LEFT JOIN organizations search_organization
                    ON search_organization.id = mp.organization_id
                    AND $3::text IS NOT NULL
                LEFT JOIN LATERAL (
                    SELECT
                        u.username
                    FROM team_members tm
                    INNER JOIN users u ON u.id = tm.user_id
                    WHERE tm.team_id = mp.team_id
                        AND tm.is_owner
                    ORDER BY tm.ordering ASC
                    LIMIT 1
                ) search_owner ON $3::text IS NOT NULL
                    AND mp.organization_id IS NULL
                LEFT JOIN external_dependencies ed ON ed.mod_id = mp.id
                LEFT JOIN version_project_types vpt ON vpt.mod_id = mp.id
            ),
            filtered_projects AS (
                SELECT
                    id,
                    queued,
                    published,
                    project_types,
                    external_dependencies_count
                FROM queue_projects
                WHERE
                    (
                        $3::text IS NULL
                        OR name ILIKE '%' || $3 || '%'
                        OR slug ILIKE '%' || $3 || '%'
                        OR summary ILIKE '%' || $3 || '%'
                        OR description ILIKE '%' || $3 || '%'
                        OR owner_name ILIKE '%' || $3 || '%'
                        OR organization_name ILIKE '%' || $3 || '%'
                        OR EXISTS (
                            SELECT 1
                            FROM UNNEST(project_types) AS searched_project_type(project_type)
                            WHERE searched_project_type.project_type ILIKE '%' || $3 || '%'
                        )
                    )
                    AND (
                        $4::text IS NULL
                        OR ($4 = 'none' AND CARDINALITY(project_types) = 0)
                        OR ($4 = 'minecraft_java_server' AND project_types @> ARRAY['minecraft_java_server']::text[])
                        OR ($4 <> 'none' AND $4 <> 'minecraft_java_server' AND
                            project_types[1] = $4
                        )
                    )
                    AND ($5::boolean IS NULL OR (external_dependencies_count > 0) = $5)
            ),
            total AS (
                SELECT COUNT(*) AS total_count FROM filtered_projects
            ),
            page_ids AS (
                SELECT
                    id,
                    queued,
                    published,
                    project_types,
                    external_dependencies_count
                FROM filtered_projects
                ORDER BY
                    CASE WHEN $8 = 'most_external_deps' THEN external_dependencies_count END DESC,
                    CASE WHEN $8 = 'least_external_deps' THEN external_dependencies_count END ASC,
                    CASE WHEN $8 = 'newest' THEN COALESCE(queued, published) END DESC NULLS LAST,
                    CASE WHEN $8 IN ('oldest', 'most_external_deps', 'least_external_deps') THEN COALESCE(queued, published) END ASC NULLS LAST,
                    id ASC
                OFFSET $7
                LIMIT $6
            ),
            page_projects AS (
                SELECT
                    m.id,
                    m.slug,
                    m.name,
                    m.summary,
                    m.icon_url,
                    m.status,
                    m.requested_status,
                    m.queued,
                    m.published,
                    m.updated,
                    m.organization_id,
                    m.team_id,
                    o.name AS organization_name,
                    o.icon_url AS organization_icon_url,
                    owner.user_id AS owner_id,
                    owner.username AS owner_name,
                    owner.avatar_url AS owner_icon_url,
                    page_ids.project_types,
                    page_ids.external_dependencies_count
                FROM page_ids
                INNER JOIN mods m ON m.id = page_ids.id
                LEFT JOIN organizations o ON o.id = m.organization_id
                LEFT JOIN LATERAL (
                    SELECT
                        tm.user_id,
                        u.username,
                        u.avatar_url
                    FROM team_members tm
                    INNER JOIN users u ON u.id = tm.user_id
                    WHERE tm.team_id = m.team_id
                        AND tm.is_owner
                    ORDER BY tm.ordering ASC
                    LIMIT 1
                ) owner ON m.organization_id IS NULL
            )
            SELECT
                total.total_count AS "total_count!",
                page_projects.id AS "id?",
                page_projects.slug AS "slug?",
                page_projects.name AS "name?",
                page_projects.summary AS "summary?",
                page_projects.icon_url AS "icon_url?",
                page_projects.status AS "status?",
                page_projects.requested_status AS "requested_status?",
                page_projects.queued AS "queued?",
                page_projects.published AS "published?",
                page_projects.updated AS "updated?",
                page_projects.organization_id AS "organization_id?",
                page_projects.organization_name AS "organization_name?",
                page_projects.organization_icon_url AS "organization_icon_url?",
                page_projects.owner_id AS "owner_id?",
                page_projects.owner_name AS "owner_name?",
                page_projects.owner_icon_url AS "owner_icon_url?",
                page_projects.project_types AS "project_types?: Vec<String>",
                page_projects.external_dependencies_count AS "external_dependencies_count?"
            FROM total
            LEFT JOIN page_projects ON true
            ORDER BY
                CASE WHEN $8 = 'most_external_deps' THEN page_projects.external_dependencies_count END DESC,
                CASE WHEN $8 = 'least_external_deps' THEN page_projects.external_dependencies_count END ASC,
                CASE WHEN $8 = 'newest' THEN COALESCE(page_projects.queued, page_projects.published) END DESC NULLS LAST,
                CASE WHEN $8 IN ('oldest', 'most_external_deps', 'least_external_deps') THEN COALESCE(page_projects.queued, page_projects.published) END ASC NULLS LAST,
                page_projects.id ASC
            "#,
            ProjectStatus::Processing.as_str(),
            &listed_version_statuses,
            query,
            project_type,
            request_opts.has_external_dependencies,
            count,
            offset,
            sort,
        )
        .fetch_all(&**pool)
        .await
        .wrap_internal_err("failed to fetch filtered projects awaiting review")?;

        let total =
            project_rows.first().map(|row| row.total_count).unwrap_or(0);
        let mut projects = Vec::new();
        for row in project_rows {
            if let Some(project) = row_to_queue_project(
                row.id,
                row.slug,
                row.name,
                row.summary,
                row.icon_url,
                row.status,
                row.requested_status,
                row.queued,
                row.published,
                row.updated,
                row.organization_id,
                row.organization_name,
                row.organization_icon_url,
                row.owner_id,
                row.owner_name,
                row.owner_icon_url,
                row.project_types,
                row.external_dependencies_count,
            )? {
                projects.push(project);
            }
        }

        (total, projects)
    } else {
        let project_rows = sqlx::query!(
            r#"
            WITH filtered_projects AS (
                SELECT
                    m.id,
                    m.queued,
                    m.published
                FROM mods m
                WHERE m.status = $1
            ),
            total AS (
                SELECT COUNT(*) AS total_count FROM filtered_projects
            ),
            page_ids AS (
                SELECT
                    id,
                    queued,
                    published
                FROM filtered_projects
                ORDER BY
                    CASE WHEN $5 = 'newest' THEN COALESCE(queued, published) END DESC NULLS LAST,
                    CASE WHEN $5 = 'oldest' THEN COALESCE(queued, published) END ASC NULLS LAST,
                    id ASC
                OFFSET $4
                LIMIT $3
            ),
            page_project_types AS (
                SELECT
                    v.mod_id,
                    ARRAY_AGG(DISTINCT pt.name::text) FILTER (WHERE pt.name IS NOT NULL) AS project_types
                FROM versions v
                INNER JOIN page_ids page ON page.id = v.mod_id
                INNER JOIN loaders_versions lv ON v.id = lv.version_id
                INNER JOIN loaders l ON lv.loader_id = l.id
                INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                WHERE v.status = ANY($2)
                GROUP BY v.mod_id
            ),
            page_external_dependencies AS (
                SELECT
                    v.mod_id,
                    COUNT(*) AS external_dependencies_count
                FROM versions v
                INNER JOIN page_ids page ON page.id = v.mod_id
                INNER JOIN dependencies d ON d.dependent_id = v.id
                WHERE d.dependency_file_name IS NOT NULL
                GROUP BY v.mod_id
            ),
            page_projects AS (
                SELECT
                    m.id,
                    m.slug,
                    m.name,
                    m.summary,
                    m.icon_url,
                    m.status,
                    m.requested_status,
                    m.queued,
                    m.published,
                    m.updated,
                    m.organization_id,
                    o.name AS organization_name,
                    o.icon_url AS organization_icon_url,
                    owner.user_id AS owner_id,
                    owner.username AS owner_name,
                    owner.avatar_url AS owner_icon_url,
                    CASE
                        WHEN m.components ? 'minecraft_server'
                            THEN ARRAY_APPEND(
                                ARRAY_REMOVE(
                                    COALESCE(ppt.project_types::text[], ARRAY[]::text[]),
                                    'modpack'
                                ),
                                'minecraft_java_server'
                            )
                        ELSE COALESCE(ppt.project_types::text[], ARRAY[]::text[])
                    END AS project_types,
                    COALESCE(ped.external_dependencies_count, 0) AS external_dependencies_count
                FROM page_ids page
                INNER JOIN mods m ON m.id = page.id
                LEFT JOIN organizations o ON o.id = m.organization_id
                LEFT JOIN LATERAL (
                    SELECT
                        tm.user_id,
                        u.username,
                        u.avatar_url
                    FROM team_members tm
                    INNER JOIN users u ON u.id = tm.user_id
                    WHERE tm.team_id = m.team_id
                        AND tm.is_owner
                    ORDER BY tm.ordering ASC
                    LIMIT 1
                ) owner ON m.organization_id IS NULL
                LEFT JOIN page_project_types ppt ON ppt.mod_id = m.id
                LEFT JOIN page_external_dependencies ped ON ped.mod_id = m.id
            )
            SELECT
                total.total_count AS "total_count!",
                page_projects.id AS "id?",
                page_projects.slug AS "slug?",
                page_projects.name AS "name?",
                page_projects.summary AS "summary?",
                page_projects.icon_url AS "icon_url?",
                page_projects.status AS "status?",
                page_projects.requested_status AS "requested_status?",
                page_projects.queued AS "queued?",
                page_projects.published AS "published?",
                page_projects.updated AS "updated?",
                page_projects.organization_id AS "organization_id?",
                page_projects.organization_name AS "organization_name?",
                page_projects.organization_icon_url AS "organization_icon_url?",
                page_projects.owner_id AS "owner_id?",
                page_projects.owner_name AS "owner_name?",
                page_projects.owner_icon_url AS "owner_icon_url?",
                page_projects.project_types AS "project_types?: Vec<String>",
                page_projects.external_dependencies_count AS "external_dependencies_count?"
            FROM total
            LEFT JOIN page_projects ON true
            ORDER BY
                CASE WHEN $5 = 'newest' THEN COALESCE(page_projects.queued, page_projects.published) END DESC NULLS LAST,
                CASE WHEN $5 = 'oldest' THEN COALESCE(page_projects.queued, page_projects.published) END ASC NULLS LAST,
                page_projects.id ASC
            "#,
            ProjectStatus::Processing.as_str(),
            &listed_version_statuses,
            count,
            offset,
            sort,
        )
        .fetch_all(&**pool)
        .await
        .wrap_internal_err("failed to fetch projects awaiting review")?;

        let total =
            project_rows.first().map(|row| row.total_count).unwrap_or(0);
        let mut projects = Vec::new();
        for row in project_rows {
            if let Some(project) = row_to_queue_project(
                row.id,
                row.slug,
                row.name,
                row.summary,
                row.icon_url,
                row.status,
                row.requested_status,
                row.queued,
                row.published,
                row.updated,
                row.organization_id,
                row.organization_name,
                row.organization_icon_url,
                row.owner_id,
                row.owner_name,
                row.owner_icon_url,
                row.project_types,
                row.external_dependencies_count,
            )? {
                projects.push(project);
            }
        }

        (total, projects)
    };

    Ok(web::Json(ModerationProjectsResponse { total, projects }))
}

#[utoipa::path(
    context_path = "/moderation",
    tag = "moderation",
    params(
		("has_external_dependencies" = Option<bool>, Query),
		("query" = Option<String>, Query),
		("project_type" = Option<String>, Query),
		("sort" = Option<ModerationProjectsSort>, Query)
	),
    responses((status = OK, body = ModerationProjectIdsResponse))
)]
#[get("/projects/ids")]
pub async fn get_project_ids(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ModerationProjectIdsResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let request_opts = request_opts.into_inner();
    let query = normalize_optional_string(request_opts.query.as_deref());
    let project_type =
        normalize_optional_string(request_opts.project_type.as_deref());
    let sort = request_opts.sort.unwrap_or_default().as_str();
    let listed_version_statuses = listed_version_statuses();
    let needs_filtered_queue = query.is_some()
        || project_type.is_some()
        || request_opts.has_external_dependencies.is_some()
        || matches!(sort, "most_external_deps" | "least_external_deps");

    let ids = if needs_filtered_queue {
        let project_rows = sqlx::query!(
            r#"
            WITH moderation_projects AS (
                SELECT
                    m.id,
                    m.slug,
                    m.name,
                    m.summary,
                    m.description,
                    m.queued,
                    m.published,
                    m.organization_id,
                    m.team_id,
                    m.components
                FROM mods m
                WHERE
                    m.status = $1
            ),
            external_dependencies AS (
                SELECT
                    v.mod_id,
                    COUNT(*) AS external_dependencies_count
                FROM versions v
                INNER JOIN moderation_projects mp ON mp.id = v.mod_id
                INNER JOIN dependencies d ON d.dependent_id = v.id
                WHERE d.dependency_file_name IS NOT NULL
                GROUP BY v.mod_id
            ),
            version_project_types AS (
                SELECT
                    v.mod_id,
                    ARRAY_AGG(DISTINCT pt.name::text) FILTER (WHERE pt.name IS NOT NULL) AS project_types
                FROM versions v
                INNER JOIN moderation_projects mp ON mp.id = v.mod_id
                INNER JOIN loaders_versions lv ON v.id = lv.version_id
                INNER JOIN loaders l ON lv.loader_id = l.id
                INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                WHERE v.status = ANY($2)
                GROUP BY v.mod_id
            ),
            queue_projects AS (
                SELECT
                    mp.id,
                    mp.slug,
                    mp.name,
                    mp.summary,
                    mp.description,
                    mp.queued,
                    mp.published,
                    search_organization.name AS organization_name,
                    search_owner.username AS owner_name,
                    CASE
                        WHEN mp.components ? 'minecraft_server'
                            THEN ARRAY_APPEND(
                                ARRAY_REMOVE(
                                    COALESCE(vpt.project_types::text[], ARRAY[]::text[]),
                                    'modpack'
                                ),
                                'minecraft_java_server'
                            )
                        ELSE COALESCE(vpt.project_types::text[], ARRAY[]::text[])
                    END AS project_types,
                    COALESCE(ed.external_dependencies_count, 0) AS external_dependencies_count
                FROM moderation_projects mp
                LEFT JOIN organizations search_organization
                    ON search_organization.id = mp.organization_id
                    AND $3::text IS NOT NULL
                LEFT JOIN LATERAL (
                    SELECT
                        u.username
                    FROM team_members tm
                    INNER JOIN users u ON u.id = tm.user_id
                    WHERE tm.team_id = mp.team_id
                        AND tm.is_owner
                    ORDER BY tm.ordering ASC
                    LIMIT 1
                ) search_owner ON $3::text IS NOT NULL
                    AND mp.organization_id IS NULL
                LEFT JOIN external_dependencies ed ON ed.mod_id = mp.id
                LEFT JOIN version_project_types vpt ON vpt.mod_id = mp.id
            )
            SELECT id
            FROM queue_projects
            WHERE
                (
                    $3::text IS NULL
                    OR name ILIKE '%' || $3 || '%'
                    OR slug ILIKE '%' || $3 || '%'
                    OR summary ILIKE '%' || $3 || '%'
                    OR description ILIKE '%' || $3 || '%'
                    OR owner_name ILIKE '%' || $3 || '%'
                    OR organization_name ILIKE '%' || $3 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM UNNEST(project_types) AS searched_project_type(project_type)
                        WHERE searched_project_type.project_type ILIKE '%' || $3 || '%'
                    )
                )
                AND (
                    $4::text IS NULL
                    OR ($4 = 'none' AND CARDINALITY(project_types) = 0)
                    OR ($4 = 'minecraft_java_server' AND project_types @> ARRAY['minecraft_java_server']::text[])
                    OR ($4 <> 'none' AND $4 <> 'minecraft_java_server' AND project_types[1] = $4)
                )
                AND ($5::boolean IS NULL OR (external_dependencies_count > 0) = $5)
            ORDER BY
                CASE WHEN $6 = 'most_external_deps' THEN external_dependencies_count END DESC,
                CASE WHEN $6 = 'least_external_deps' THEN external_dependencies_count END ASC,
                CASE WHEN $6 = 'newest' THEN COALESCE(queued, published) END DESC NULLS LAST,
                CASE WHEN $6 IN ('oldest', 'most_external_deps', 'least_external_deps') THEN COALESCE(queued, published) END ASC NULLS LAST,
                id ASC
            "#,
            ProjectStatus::Processing.as_str(),
            &listed_version_statuses,
            query,
            project_type,
            request_opts.has_external_dependencies,
            sort,
        )
        .fetch_all(&**pool)
        .await
        .wrap_internal_err("failed to fetch filtered project ids awaiting review")?;

        project_rows
            .into_iter()
            .map(|row| ProjectId::from(database::models::DBProjectId(row.id)))
            .collect()
    } else {
        let project_rows = sqlx::query!(
            r#"
            SELECT id
            FROM mods
            WHERE status = $1
            ORDER BY
                CASE WHEN $2 = 'newest' THEN COALESCE(queued, published) END DESC NULLS LAST,
                CASE WHEN $2 = 'oldest' THEN COALESCE(queued, published) END ASC NULLS LAST,
                id ASC
            "#,
            ProjectStatus::Processing.as_str(),
            sort,
        )
        .fetch_all(&**pool)
        .await
        .wrap_internal_err("failed to fetch project ids awaiting review")?;

        project_rows
            .into_iter()
            .map(|row| ProjectId::from(database::models::DBProjectId(row.id)))
            .collect()
    };

    Ok(web::Json(ModerationProjectIdsResponse { ids }))
}

fn row_to_queue_project(
    id: Option<i64>,
    slug: Option<String>,
    name: Option<String>,
    summary: Option<String>,
    icon_url: Option<String>,
    status: Option<String>,
    requested_status: Option<String>,
    queued: Option<DateTime<Utc>>,
    published: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
    organization_id: Option<i64>,
    organization_name: Option<String>,
    organization_icon_url: Option<String>,
    owner_id: Option<i64>,
    owner_name: Option<String>,
    owner_icon_url: Option<String>,
    project_types: Option<Vec<String>>,
    external_dependencies_count: Option<i64>,
) -> Result<Option<ModerationQueueProject>, ApiError> {
    let Some(id) = id else {
        return Ok(None);
    };

    let project_id = ProjectId::from(database::models::DBProjectId(id));
    let name = name.wrap_internal_err_with(|| {
        eyre!("project {project_id} is missing `name` in moderation queue row")
    })?;
    let summary = summary.wrap_internal_err_with(|| {
        eyre!(
            "project {project_id} is missing `summary` in moderation queue row"
        )
    })?;
    let status = status.wrap_internal_err_with(|| {
        eyre!(
            "project {project_id} is missing `status` in moderation queue row"
        )
    })?;
    let published = published.wrap_internal_err_with(|| {
        eyre!(
            "project {project_id} is missing `published` in moderation queue row"
        )
    })?;
    let updated = updated.wrap_internal_err_with(|| {
        eyre!(
            "project {project_id} is missing `updated` in moderation queue row"
        )
    })?;
    let ownership = row_to_ownership(
        project_id,
        organization_id,
        organization_name,
        organization_icon_url,
        owner_id,
        owner_name,
        owner_icon_url,
    )?;

    Ok(Some(ModerationQueueProject {
        id: project_id,
        slug,
        name,
        summary,
        icon_url,
        status: ProjectStatus::from_string(&status),
        requested_status: requested_status
            .as_deref()
            .map(ProjectStatus::from_string),
        queued,
        published,
        updated,
        project_types: project_types.unwrap_or_default(),
        ownership,
        external_dependencies_count: external_dependencies_count.unwrap_or(0),
    }))
}

fn normalize_optional_string(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn listed_version_statuses() -> Vec<String> {
    VersionStatus::iterator()
        .filter(|status| status.is_listed())
        .map(|status| status.as_str().to_string())
        .collect()
}

fn row_to_ownership(
    project_id: ProjectId,
    organization_id: Option<i64>,
    organization_name: Option<String>,
    organization_icon_url: Option<String>,
    owner_id: Option<i64>,
    owner_name: Option<String>,
    owner_icon_url: Option<String>,
) -> Result<Ownership, ApiError> {
    if let Some(organization_id) = organization_id {
        let organization_name =
            organization_name.wrap_internal_err_with(|| {
                eyre!(
                    "project {project_id} is owned by organization {} without a valid name",
                    OrganizationId::from(database::models::DBOrganizationId(
                        organization_id
                    ))
                )
            })?;

        return Ok(Ownership::Organization {
            id: OrganizationId::from(database::models::DBOrganizationId(
                organization_id,
            )),
            name: organization_name,
            icon_url: organization_icon_url,
        });
    }

    let owner_id = owner_id.wrap_internal_err_with(|| {
        eyre!("project {project_id} is owned by a team without a valid owner")
    })?;
    let owner_name = owner_name.wrap_internal_err_with(|| {
        eyre!(
            "project {project_id} is owned by a team owner without a valid name"
        )
    })?;

    Ok(Ownership::User {
        id: UserId::from(database::models::DBUserId(owner_id)),
        name: owner_name,
        icon_url: owner_icon_url,
    })
}

/// Get project moderation metadata.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
	responses((status = OK, body = MissingMetadata))
)]
#[get("/project/{id}")]
pub async fn get_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    info: web::Path<(String,)>,
) -> Result<web::Json<MissingMetadata>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let project_id = info.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id, &**pool, &redis).await?;

    if let Some(project) = project {
        let rows = sqlx::query!(
            "
            SELECT
            f.metadata, v.id version_id
            FROM versions v
            INNER JOIN files f ON f.version_id = v.id
            WHERE v.mod_id = $1
            ",
            project.inner.id.0
        )
        .fetch_all(&**pool)
        .await?;

        let mut merged = MissingMetadata {
            identified: HashMap::new(),
            flame_files: HashMap::new(),
            unknown_files: HashMap::new(),
        };

        let mut check_hashes = Vec::new();
        let mut check_flames = Vec::new();

        for row in rows {
            if let Some(metadata) = row
                .metadata
                .and_then(|x| serde_json::from_value::<MissingMetadata>(x).ok())
            {
                merged.identified.extend(metadata.identified);
                merged.flame_files.extend(metadata.flame_files);
                merged.unknown_files.extend(metadata.unknown_files);

                check_hashes.extend(merged.flame_files.keys().cloned());
                check_hashes.extend(merged.unknown_files.keys().cloned());
                check_flames
                    .extend(merged.flame_files.values().map(|x| x.id as i32));
            }
        }

        let rows = sqlx::query!(
            "
            SELECT encode(mef.sha1, 'escape') sha1, mel.status status
            FROM moderation_external_files mef
            INNER JOIN moderation_external_licenses mel ON mef.external_license_id = mel.id
            WHERE mef.sha1 = ANY($1)
            ",
            &check_hashes
                .iter()
                .map(|x| x.as_bytes().to_vec())
                .collect::<Vec<_>>()
        )
        .fetch_all(&**pool)
        .await?;

        for row in rows {
            if let Some(sha1) = row.sha1 {
                if let Some(val) = merged.flame_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val.file_name,
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                } else if let Some(val) = merged.unknown_files.remove(&sha1) {
                    merged.identified.insert(
                        sha1,
                        IdentifiedFile {
                            file_name: val,
                            status: ApprovalType::from_string(&row.status)
                                .unwrap_or(ApprovalType::Unidentified),
                        },
                    );
                }
            }
        }

        let rows = sqlx::query!(
            "
            SELECT mel.id, mel.flame_project_id, mel.status status
            FROM moderation_external_licenses mel
            WHERE mel.flame_project_id = ANY($1)
            ",
            &check_flames,
        )
        .fetch_all(&**pool)
        .await?;

        for row in rows {
            if let Some(sha1) = merged
                .flame_files
                .iter()
                .find(|x| Some(x.1.id as i32) == row.flame_project_id)
                .map(|x| x.0.clone())
                && let Some(val) = merged.flame_files.remove(&sha1)
            {
                merged.identified.insert(
                    sha1,
                    IdentifiedFile {
                        file_name: val.file_name.clone(),
                        status: ApprovalType::from_string(&row.status)
                            .unwrap_or(ApprovalType::Unidentified),
                    },
                );
            }
        }

        Ok(web::Json(merged))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Judgement {
    Flame {
        id: i32,
        status: ApprovalType,
        link: String,
        title: String,
    },
    Unknown {
        status: ApprovalType,
        proof: Option<String>,
        link: Option<String>,
        title: Option<String>,
    },
}

/// Update project moderation judgements.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
	responses((status = NO_CONTENT))
)]
#[post("/project")]
pub async fn set_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    judgements: web::Json<HashMap<String, Judgement>>,
) -> Result<(), ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let mut licenses = Vec::new();
    let mut file_hashes = Vec::new();
    let mut file_filenames = Vec::new();
    let mut file_license_ids = Vec::new();

    for (hash, judgement) in judgements.0 {
        let id = random_base62(8);

        let (title, status, link, proof, flame_id) = match judgement {
            Judgement::Flame {
                id,
                status,
                link,
                title,
            } => (
                Some(title),
                status,
                Some(link),
                Some("See Flame page/license for permission".to_string()),
                Some(id),
            ),
            Judgement::Unknown {
                status,
                proof,
                link,
                title,
            } => (title, status, link, proof, None),
        };

        licenses.push(moderation_external_item::ExternalLicense {
            id: id as i64,
            title,
            status: status.as_str().to_string(),
            link,
            proof,
            flame_project_id: flame_id,
        });
        file_hashes.push(hash);
        file_filenames.push(None);
        file_license_ids.push(id as i64);
    }

    let user_id = database::models::ids::DBUserId::from(user.id);

    moderation_external_item::ExternalLicense::insert_many(
        &mut transaction,
        &licenses,
        user_id,
    )
    .await?;

    moderation_external_item::ExternalLicense::insert_files(
        &mut transaction,
        &file_hashes
            .iter()
            .map(|x| x.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        &file_filenames,
        &file_license_ids,
        user_id,
    )
    .await?;

    transaction.commit().await?;

    Ok(())
}

/// Acquire a moderation lock.  
/// Returns success if acquired, or info about who holds the lock if blocked.
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    responses(
        (status = OK, body = LockAcquireResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[post("/lock/{project_id}")]
pub async fn acquire_lock(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<LockAcquireResponse>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;

    let project_id_str = path.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id_str, &**pool, &redis)
            .await?
            .ok_or(ApiError::NotFound)?;

    let db_project_id = project.inner.id;
    let db_user_id = database::models::DBUserId::from(user.id);

    match DBModerationLock::acquire(db_project_id, db_user_id, &pool).await? {
        Ok(()) => Ok(web::Json(LockAcquireResponse {
            success: true,
            is_own_lock: true,
            locked_by: None,
            locked_at: None,
            expires_at: None,
            expired: None,
        })),
        Err(lock) => Ok(web::Json(LockAcquireResponse {
            success: false,
            is_own_lock: false,
            locked_by: Some(LockedByUser {
                id: UserId::from(lock.moderator_id).to_string(),
                username: lock.moderator_username,
                avatar_url: lock.moderator_avatar_url,
            }),
            locked_at: Some(lock.locked_at),
            expires_at: Some(lock.expires_at),
            expired: Some(lock.expired),
        })),
    }
}

/// Override a moderation lock.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    responses(
        (status = OK, body = LockAcquireResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[post("/lock/{project_id}/override")]
pub async fn override_lock(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<LockAcquireResponse>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;

    let project_id_str = path.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id_str, &**pool, &redis)
            .await?
            .ok_or(ApiError::NotFound)?;

    let db_project_id = project.inner.id;
    let db_user_id = database::models::DBUserId::from(user.id);

    DBModerationLock::force_acquire(db_project_id, db_user_id, &pool).await?;

    Ok(web::Json(LockAcquireResponse {
        success: true,
        is_own_lock: true,
        locked_by: None,
        locked_at: None,
        expires_at: None,
        expired: None,
    }))
}

/// Get moderation lock status.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    responses(
        (status = OK, body = LockStatusResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[get("/lock/{project_id}")]
pub async fn get_lock_status(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<LockStatusResponse>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let project_id_str = path.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id_str, &**pool, &redis)
            .await?
            .ok_or(ApiError::NotFound)?;

    let db_project_id = project.inner.id;
    let db_user_id = database::models::DBUserId::from(user.id);

    match DBModerationLock::get_with_user(db_project_id, &pool).await? {
        Some(lock) => {
            let is_own_lock = lock.moderator_id == db_user_id;
            Ok(web::Json(LockStatusResponse {
                locked: true,
                is_own_lock,
                locked_by: Some(LockedByUser {
                    id: UserId::from(lock.moderator_id).to_string(),
                    username: lock.moderator_username,
                    avatar_url: lock.moderator_avatar_url,
                }),
                locked_at: Some(lock.locked_at),
                expires_at: Some(lock.expires_at),
                expired: Some(lock.expired),
            }))
        }
        None => Ok(web::Json(LockStatusResponse {
            locked: false,
            is_own_lock: false,
            locked_by: None,
            locked_at: None,
            expires_at: None,
            expired: None,
        })),
    }
}

/// Release a moderation lock.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    responses(
        (status = OK, body = LockReleaseResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[delete("/lock/{project_id}")]
pub async fn release_lock(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<LockReleaseResponse>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?;

    let project_id_str = path.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id_str, &**pool, &redis)
            .await?
            .ok_or(ApiError::NotFound)?;

    let db_project_id = project.inner.id;
    let db_user_id = database::models::DBUserId::from(user.id);

    let released =
        DBModerationLock::release(db_project_id, db_user_id, &pool).await?;

    let _ = DBModerationLock::cleanup_expired(&pool).await;

    Ok(web::Json(LockReleaseResponse { success: released }))
}

/// Release a moderation lock by beacon.  
///
/// For use with `navigator.sendBeacon`, which cannot set `Authorization` or send `DELETE`.
/// The body must be `text/plain` containing the same token value as the `Authorization` header
/// (optional `Bearer ` prefix). This avoids a CORS preflight compared to `application/json`.
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    request_body(
        content = String,
        description = "Token value (same as Authorization header)",
        content_type = "text/plain"
    ),
    responses(
        (status = OK, body = LockReleaseResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[post("/lock/{project_id}/release")]
pub async fn release_lock_beacon(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
    body: String,
) -> Result<web::Json<LockReleaseResponse>, ApiError> {
    let token = body.trim();
    if token.is_empty() {
        return Err(ApiError::InvalidInput(
            "missing token in request body".to_string(),
        ));
    }
    let token = token.strip_prefix("Bearer ").unwrap_or(token).trim();

    let (scopes, user) = get_user_from_bearer_token(
        &req,
        Some(token),
        &**pool,
        &redis,
        &session_queue,
        false,
    )
    .await?;

    if !scopes.contains(Scopes::PROJECT_WRITE) {
        return Err(ApiError::CustomAuthentication(
            "token is missing required scopes".to_string(),
        ));
    }
    if !user.role.is_mod() {
        return Err(ApiError::CustomAuthentication(
            "only moderators may release moderation locks".to_string(),
        ));
    }

    let project_id_str = path.into_inner().0;
    let project =
        database::models::DBProject::get(&project_id_str, &**pool, &redis)
            .await?
            .ok_or(ApiError::NotFound)?;

    let db_project_id = project.inner.id;
    let db_user_id = database::models::DBUserId::from(user.id);

    let released =
        DBModerationLock::release(db_project_id, db_user_id, &pool).await?;

    let _ = DBModerationLock::cleanup_expired(&pool).await;

    Ok(web::Json(LockReleaseResponse { success: released }))
}

/// Delete all moderation locks.  
#[utoipa::path(
	context_path = "/moderation",
	tag = "moderation",
    responses(
        (status = OK, body = DeleteAllLocksResponse),
        (status = UNAUTHORIZED, description = "Not an admin")
    )
)]
#[delete("/locks")]
pub async fn delete_all_locks(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<DeleteAllLocksResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await?
    .1;

    if !user.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You must be an admin to delete all locks".to_string(),
        ));
    }

    let deleted_count = DBModerationLock::delete_all(&pool).await?;

    Ok(web::Json(DeleteAllLocksResponse { deleted_count }))
}
