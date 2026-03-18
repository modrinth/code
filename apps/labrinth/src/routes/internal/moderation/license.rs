use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::util::error::Context;
use crate::{auth::check_is_moderator_from_headers, routes::ApiError};
use actix_web::{HttpRequest, get, patch, post, web};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(search_licenses)
        .service(get_license)
        .service(get_license_by_sha1)
        .service(update_license);
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchLicenses {
    /// Optional text query that must be present in the title
    #[serde(default)]
    pub query: Option<String>,
    /// The ID to start pagination from (exclusive)
    #[serde(default)]
    pub next_id: Option<i64>,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    #[schema(default = 20)]
    pub limit: u64,
}

fn default_limit() -> u64 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExternalLicense {
    pub id: i64,
    pub title: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exceptions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flame_project_id: Option<i32>,
    pub files: Vec<ModerationExternalFile>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModerationExternalFile {
    /// Hex-encoded SHA1 hash
    pub sha1: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateExternalLicense {
    /// The title of the license
    #[serde(default)]
    pub title: Option<String>,
    /// The status of the license
    #[serde(default)]
    pub status: Option<String>,
    /// A link to the license
    #[serde(default)]
    pub link: Option<String>,
    /// Exceptions to the license
    #[serde(default)]
    pub exceptions: Option<String>,
    /// Proof of the license
    #[serde(default)]
    pub proof: Option<String>,
    /// The Flame project ID associated with the license
    #[serde(default)]
    pub flame_project_id: Option<i32>,
}

/// Search moderation external licenses
#[utoipa::path(
    responses((status = OK, body = Vec<ExternalLicense>))
)]
#[post("/external-licenses/search")]
async fn search_licenses(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(search): web::Json<SearchLicenses>,
) -> Result<web::Json<Vec<ExternalLicense>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let SearchLicenses {
        query,
        next_id,
        limit,
    } = search;
    let query_pattern = query.as_ref().map(|q| format!("%{q}%"));

    let results = sqlx::query!(
        r#"
        SELECT id, title, status, link, exceptions, proof, flame_project_id
        FROM moderation_external_licenses
        WHERE ($1::text IS NULL OR title ILIKE $1)
        AND ($2::bigint IS NULL OR id > $2)
        ORDER BY id ASC
        LIMIT $3
        "#,
        query_pattern,
        next_id,
        limit as i64
    )
    .fetch_all(&**pool)
    .await?;

    let licenses = results
        .into_iter()
        .map(|row| ExternalLicense {
            id: row.id,
            title: row.title,
            status: row.status,
            link: row.link,
            exceptions: row.exceptions,
            proof: row.proof,
            flame_project_id: row.flame_project_id,
            files: vec![],
        })
        .collect();

    Ok(web::Json(licenses))
}

/// Get a single moderation external license by ID
#[utoipa::path(
    params(("id" = i64, Path, description = "License ID")),
    responses((status = OK, body = ExternalLicense))
)]
#[get("/external-licenses/{id}")]
async fn get_license(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(i64,)>,
) -> Result<web::Json<ExternalLicense>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let (id,) = path.into_inner();

    let result = sqlx::query!(
        r#"
        SELECT id, title, status, link, exceptions, proof, flame_project_id
        FROM moderation_external_licenses
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let external_files = sqlx::query!(
        r#"
        SELECT encode(sha1, 'hex') as "sha1!"
        FROM moderation_external_files
        WHERE external_license_id = $1
        "#,
        id
    )
    .fetch_all(&**pool)
    .await?
    .into_iter()
    .map(|row| ModerationExternalFile { sha1: row.sha1 })
    .collect::<Vec<_>>();

    Ok(web::Json(ExternalLicense {
        id: result.id,
        title: result.title,
        status: result.status,
        link: result.link,
        exceptions: result.exceptions,
        proof: result.proof,
        flame_project_id: result.flame_project_id,
        files: external_files,
    }))
}

/// Get the external license associated with a file by its SHA1 hash
#[utoipa::path(
    params(("sha1" = String, Path, description = "SHA1 hash of the file (hex-encoded)")),
    responses((status = OK, body = ExternalLicense))
)]
#[get("/external-licenses/by-sha1/{sha1}")]
async fn get_license_by_sha1(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<ExternalLicense>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let (sha1_hex,) = path.into_inner();

    // Validate and convert hex string to bytea
    let sha1_bytes =
        hex::decode(&sha1_hex).wrap_request_err("invalid SHA1 hex string")?;

    let sha1_bytes = <[u8; 20]>::try_from(sha1_bytes).map_err(|_| {
        ApiError::Request(eyre!(
            "SHA1 hash must be exactly 20 bytes".to_string(),
        ))
    })?;

    let result = sqlx::query!(
        r#"
        SELECT mel.id, mel.title, mel.status, mel.link, mel.exceptions, mel.proof, mel.flame_project_id
        FROM moderation_external_files mef
        INNER JOIN moderation_external_licenses mel ON mef.external_license_id = mel.id
        WHERE mef.sha1 = $1
        "#,
        sha1_bytes.as_slice()
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let files = sqlx::query!(
        r#"
        SELECT encode(sha1, 'hex') as "sha1!"
        FROM moderation_external_files
        WHERE external_license_id = $1
        "#,
        result.id
    )
    .fetch_all(&**pool)
    .await?
    .into_iter()
    .map(|row| ModerationExternalFile { sha1: row.sha1 })
    .collect::<Vec<_>>();

    Ok(web::Json(ExternalLicense {
        id: result.id,
        title: result.title,
        status: result.status,
        link: result.link,
        exceptions: result.exceptions,
        proof: result.proof,
        flame_project_id: result.flame_project_id,
        files,
    }))
}

/// Update a moderation external license by ID
#[utoipa::path]
#[patch("/external-licenses/{id}")]
async fn update_license(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(i64,)>,
    web::Json(update): web::Json<UpdateExternalLicense>,
) -> Result<web::Json<ExternalLicense>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let (id,) = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE moderation_external_licenses
        SET
            title = COALESCE($1, title),
            status = COALESCE($2, status),
            link = COALESCE($3, link),
            exceptions = COALESCE($4, exceptions),
            proof = COALESCE($5, proof),
            flame_project_id = COALESCE($6, flame_project_id)
        WHERE id = $7
        RETURNING id, title, status, link, exceptions, proof, flame_project_id
        "#,
        update.title,
        update.status,
        update.link,
        update.exceptions,
        update.proof,
        update.flame_project_id,
        id,
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(web::Json(ExternalLicense {
        id: result.id,
        title: result.title,
        status: result.status,
        link: result.link,
        exceptions: result.exceptions,
        proof: result.proof,
        flame_project_id: result.flame_project_id,
        files: vec![],
    }))
}
