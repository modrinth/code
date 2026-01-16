use super::ApiError;
use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::models::DBModerationLock;
use crate::database::redis::RedisPool;
use crate::models::ids::OrganizationId;
use crate::models::projects::{Project, ProjectStatus};
use crate::queue::moderation::{ApprovalType, IdentifiedFile, MissingMetadata};
use crate::queue::session::AuthQueue;
use crate::util::error::Context;
use crate::{auth::check_is_moderator_from_headers, models::pats::Scopes};
use actix_web::{HttpRequest, delete, get, post, web};
use ariadne::ids::{UserId, random_base62};
use chrono::{DateTime, Utc};
use ownership::get_projects_ownership;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

mod ownership;
mod tech_review;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(get_projects)
        .service(get_project_meta)
        .service(set_project_meta)
        .service(acquire_lock)
        .service(get_lock_status)
        .service(release_lock)
        .service(delete_all_locks)
        .service(
            utoipa_actix_web::scope("/tech-review")
                .configure(tech_review::config),
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
}

fn default_count() -> u16 {
    100
}

/// Project with extra information fetched from the database, to avoid having
/// clients make more round trips.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FetchedProject {
    /// Project info.
    #[serde(flatten)]
    pub project: Project,
    /// Who owns the project.
    pub ownership: Ownership,
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
    /// Information about who holds the lock (if locked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<LockedByUser>,
    /// When the lock was acquired
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    /// Whether the lock has expired (>15 minutes old)
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
    /// If blocked, info about who holds the lock
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<LockedByUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
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

/// Fetch all projects which are in the moderation queue.
#[utoipa::path(
    responses((status = OK, body = inline(Vec<FetchedProject>)))
)]
#[get("/projects")]
async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<FetchedProject>>, ApiError> {
    get_projects_internal(req, pool, redis, request_opts, session_queue).await
}

pub async fn get_projects_internal(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<FetchedProject>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    use futures::stream::TryStreamExt;

    let project_ids = sqlx::query!(
        "
        SELECT id
        FROM (
            SELECT DISTINCT ON (m.id)
                m.id,
                m.queued
            FROM mods m

            /* -- Temporarily, don't exclude projects in tech rev q

                -- exclude projects in tech review queue
                LEFT JOIN delphi_issue_details_with_statuses didws
                    ON didws.project_id = m.id AND didws.status = 'pending'
            */

            WHERE
                m.status = $1
                /* AND didws.status IS NULL */ -- Temporarily don't exclude

            GROUP BY m.id
        ) t
        ORDER BY queued ASC
        OFFSET $3
        LIMIT $2
        ",
        ProjectStatus::Processing.as_str(),
        request_opts.count as i64,
        request_opts.offset as i64
    )
    .fetch(&**pool)
    .map_ok(|m| database::models::DBProjectId(m.id))
    .try_collect::<Vec<database::models::DBProjectId>>()
    .await
    .wrap_internal_err("failed to fetch projects awaiting review")?;

    let projects =
        database::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await
            .wrap_internal_err("failed to fetch projects")?
            .into_iter()
            .map(crate::models::projects::Project::from)
            .collect::<Vec<_>>();

    let ownerships = get_projects_ownership(&projects, &pool, &redis)
        .await
        .wrap_internal_err("failed to fetch project ownerships")?;

    let map_project =
        |(project, ownership): (Project, Ownership)| -> FetchedProject {
            FetchedProject { ownership, project }
        };

    let projects = projects
        .into_iter()
        .zip(ownerships)
        .map(map_project)
        .collect::<Vec<_>>();

    Ok(web::Json(projects))
}

/// Fetch moderation metadata for a specific project.
#[utoipa::path(
    responses((status = OK, body = inline(Vec<Project>)))
)]
#[get("/project/{id}")]
async fn get_project_meta(
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

/// Update moderation judgements for projects in the review queue.
#[utoipa::path]
#[post("/project")]
async fn set_project_meta(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    judgements: web::Json<HashMap<String, Judgement>>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let mut ids = Vec::new();
    let mut titles = Vec::new();
    let mut statuses = Vec::new();
    let mut links = Vec::new();
    let mut proofs = Vec::new();
    let mut flame_ids = Vec::new();

    let mut file_hashes = Vec::new();

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

        ids.push(id as i64);
        titles.push(title);
        statuses.push(status.as_str());
        links.push(link);
        proofs.push(proof);
        flame_ids.push(flame_id);
        file_hashes.push(hash);
    }

    sqlx::query(
    "
        INSERT INTO moderation_external_licenses (id, title, status, link, proof, flame_project_id)
        SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[], $6::integer[])
        "
    )
        .bind(&ids[..])
        .bind(&titles[..])
        .bind(&statuses[..])
        .bind(&links[..])
        .bind(&proofs[..])
        .bind(&flame_ids[..])
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "
        INSERT INTO moderation_external_files (sha1, external_license_id)
        SELECT * FROM UNNEST ($1::bytea[], $2::bigint[])
        ON CONFLICT (sha1)
        DO NOTHING
        ",
    )
    .bind(&file_hashes[..])
    .bind(&ids[..])
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

/// Acquire or refresh a moderation lock on a project.
/// Returns success if acquired, or info about who holds the lock if blocked.
#[utoipa::path(
    responses(
        (status = OK, body = LockAcquireResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[post("/lock/{project_id}")]
async fn acquire_lock(
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
            locked_by: None,
            locked_at: None,
            expired: None,
        })),
        Err(lock) => Ok(web::Json(LockAcquireResponse {
            success: false,
            locked_by: Some(LockedByUser {
                id: UserId::from(lock.moderator_id).to_string(),
                username: lock.moderator_username,
                avatar_url: lock.moderator_avatar_url,
            }),
            locked_at: Some(lock.locked_at),
            expired: Some(lock.expired),
        })),
    }
}

/// Check the lock status for a project
#[utoipa::path(
    responses(
        (status = OK, body = LockStatusResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[get("/lock/{project_id}")]
async fn get_lock_status(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<LockStatusResponse>, ApiError> {
    check_is_moderator_from_headers(
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

    match DBModerationLock::get_with_user(db_project_id, &pool).await? {
        Some(lock) => Ok(web::Json(LockStatusResponse {
            locked: true,
            locked_by: Some(LockedByUser {
                id: UserId::from(lock.moderator_id).to_string(),
                username: lock.moderator_username,
                avatar_url: lock.moderator_avatar_url,
            }),
            locked_at: Some(lock.locked_at),
            expired: Some(lock.expired),
        })),
        None => Ok(web::Json(LockStatusResponse {
            locked: false,
            locked_by: None,
            locked_at: None,
            expired: None,
        })),
    }
}

/// Release a moderation lock on a project
#[utoipa::path(
    responses(
        (status = OK, body = LockReleaseResponse),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
#[delete("/lock/{project_id}")]
async fn release_lock(
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

/// Delete all moderation locks (admin only)
#[utoipa::path(
    responses(
        (status = OK, body = DeleteAllLocksResponse),
        (status = UNAUTHORIZED, description = "Not an admin")
    )
)]
#[delete("/locks")]
async fn delete_all_locks(
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
