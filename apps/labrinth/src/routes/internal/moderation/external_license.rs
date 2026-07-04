use std::collections::HashMap;

use actix_web::{HttpRequest, get, patch, post, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::PgPool;
use crate::database::models::ids::DBUserId;
use crate::database::models::moderation_external_item::ExternalLicense;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::queue::moderation::ApprovalType;
use crate::routes::ApiError;
use crate::{auth::check_is_moderator_from_headers, queue::session::AuthQueue};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(search)
        .service(get_by_sha1)
        .service(get_by_sha1_bulk)
        .service(lookup)
        .service(update_license)
        .service(add_file)
        .service(reassign_file);
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExternalProject {
    pub id: i64,
    pub title: Option<String>,
    pub status: ApprovalType,
    pub link: Option<String>,
    pub exceptions: Option<String>,
    pub proof: Option<String>,
    pub flame_project_id: Option<i32>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub inserted_by: Option<i64>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<i64>,
    pub linked_files: Vec<LinkedFile>,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LinkedFile {
    pub name: Option<String>,
    pub sha1: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SearchRequest {
    pub title: Option<String>,
    pub flame_id: Option<i32>,
    pub flame_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct HashLookupRequest {
    pub hashes: Vec<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ExternalLicenseLookupRequest {
    #[serde(default)]
    pub flame_ids: Vec<i32>,
    #[serde(default)]
    pub hashes: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ExternalLicenseLookupResponse {
    pub flame_ids: HashMap<i32, Vec<ExternalProject>>,
    pub hashes: HashMap<String, ExternalProject>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateLicenseRequest {
    pub title: Option<String>,
    pub status: ApprovalType,
    pub link: Option<String>,
    pub exceptions: Option<String>,
    pub proof: Option<String>,
    pub flame_project_id: Option<i32>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct FileLicenseRequest {
    pub hashes: Vec<String>,
    pub license_id: LicenseId,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum LicenseId {
    Number(i64),
    String(String),
}

impl LicenseId {
    fn parse(self) -> Result<i64, ApiError> {
        match self {
            LicenseId::Number(id) => Ok(id),
            LicenseId::String(id) => id.parse().map_err(|_| {
                ApiError::InvalidInput(
                    "license_id must be a valid integer".to_string(),
                )
            }),
        }
    }
}

struct LicenseRow {
    id: i64,
    title: Option<String>,
    status: String,
    link: Option<String>,
    exceptions: Option<String>,
    proof: Option<String>,
    flame_project_id: Option<i32>,
    inserted_at: Option<DateTime<Utc>>,
    inserted_by: Option<i64>,
    updated_at: Option<DateTime<Utc>>,
    updated_by: Option<i64>,
}

struct LicenseHashRow {
    hash: Vec<u8>,
    id: i64,
    title: Option<String>,
    status: String,
    link: Option<String>,
    exceptions: Option<String>,
    proof: Option<String>,
    flame_project_id: Option<i32>,
    inserted_at: Option<DateTime<Utc>>,
    inserted_by: Option<i64>,
    updated_at: Option<DateTime<Utc>>,
    updated_by: Option<i64>,
}

fn normalize_sha1_hashes(hashes: &[String]) -> Result<Vec<String>, ApiError> {
    hashes
        .iter()
        .map(|hash| {
            let hash = hash.trim().to_lowercase();
            if hash.len() != 40 || !hash.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Err(ApiError::InvalidInput(
                    "hash must be a valid SHA1 hex string".to_string(),
                ));
            }

            Ok(hash)
        })
        .collect()
}

impl LicenseRow {
    fn into_external_project(
        self,
        linked_files: Vec<LinkedFile>,
    ) -> ExternalProject {
        ExternalProject {
            id: self.id,
            title: self.title,
            status: ApprovalType::from_string(&self.status)
                .unwrap_or(ApprovalType::Unidentified),
            link: self.link,
            exceptions: self.exceptions,
            proof: self.proof,
            flame_project_id: self.flame_project_id,
            inserted_at: self.inserted_at,
            inserted_by: self.inserted_by,
            updated_at: self.updated_at,
            updated_by: self.updated_by,
            linked_files,
        }
    }
}

async fn fetch_linked_files(
    pool: &PgPool,
    license_ids: &[i64],
) -> Result<HashMap<i64, Vec<LinkedFile>>, ApiError> {
    if license_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let file_rows = sqlx::query!(
        r#"
        SELECT
            mef.external_license_id,
            mef.sha1,
            mef.filename
        FROM moderation_external_files mef
        WHERE mef.external_license_id = ANY($1)
        "#,
        license_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<i64, Vec<LinkedFile>> = HashMap::new();
    for row in file_rows {
        map.entry(row.external_license_id)
            .or_default()
            .push(LinkedFile {
                name: row.filename,
                sha1: String::from_utf8(row.sha1)
                    .unwrap_or_else(|err| hex::encode(err.into_bytes())),
            });
    }
    Ok(map)
}

async fn fetch_by_hashes(
    pool: &PgPool,
    hashes: &[String],
) -> Result<HashMap<String, ExternalProject>, ApiError> {
    if hashes.is_empty() {
        return Ok(HashMap::new());
    }

    let hash_bytes = hashes
        .iter()
        .map(|hash| hash.as_bytes().to_vec())
        .collect::<Vec<_>>();

    let rows = sqlx::query_as!(
        LicenseHashRow,
        r#"
        SELECT
            mef.sha1 hash,
            mel.id,
            mel.title,
            mel.status,
            mel.link,
            mel.exceptions,
            mel.proof,
            mel.flame_project_id,
            mel.inserted_at,
            mel.inserted_by,
            mel.updated_at,
            mel.updated_by
        FROM moderation_external_files mef
        INNER JOIN moderation_external_licenses mel ON mel.id = mef.external_license_id
        WHERE mef.sha1 = ANY($1)
        "#,
        &hash_bytes,
    )
    .fetch_all(pool)
    .await?;

    let license_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let files_map = fetch_linked_files(pool, &license_ids).await?;

    let mut results = HashMap::new();
    for row in rows {
        let hash = String::from_utf8(row.hash)
            .unwrap_or_else(|err| hex::encode(err.into_bytes()));
        let linked_files = files_map.get(&row.id).cloned().unwrap_or_default();
        results.insert(
            hash,
            LicenseRow {
                id: row.id,
                title: row.title,
                status: row.status,
                link: row.link,
                exceptions: row.exceptions,
                proof: row.proof,
                flame_project_id: row.flame_project_id,
                inserted_at: row.inserted_at,
                inserted_by: row.inserted_by,
                updated_at: row.updated_at,
                updated_by: row.updated_by,
            }
            .into_external_project(linked_files),
        );
    }

    Ok(results)
}

async fn fetch_by_flame_ids(
    pool: &PgPool,
    flame_ids: &[i32],
) -> Result<HashMap<i32, Vec<ExternalProject>>, ApiError> {
    if flame_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as!(
        LicenseRow,
        r#"
        SELECT
            mel.id,
            mel.title,
            mel.status,
            mel.link,
            mel.exceptions,
            mel.proof,
            mel.flame_project_id,
            mel.inserted_at,
            mel.inserted_by,
            mel.updated_at,
            mel.updated_by
        FROM moderation_external_licenses mel
        WHERE mel.flame_project_id = ANY($1)
        ORDER BY mel.id
        "#,
        flame_ids,
    )
    .fetch_all(pool)
    .await?;

    let license_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let files_map = fetch_linked_files(pool, &license_ids).await?;

    let mut results: HashMap<i32, Vec<ExternalProject>> = HashMap::new();
    for row in rows {
        if let Some(flame_project_id) = row.flame_project_id {
            let linked_files =
                files_map.get(&row.id).cloned().unwrap_or_default();
            results
                .entry(flame_project_id)
                .or_default()
                .push(row.into_external_project(linked_files));
        }
    }

    Ok(results)
}

/// Search external licenses.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = inline(Vec<ExternalProject>)))
)]
#[post("/search")]
pub async fn search(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<SearchRequest>,
) -> Result<web::Json<Vec<ExternalProject>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let rows = sqlx::query_as!(
        LicenseRow,
        r#"
        SELECT
            mel.id,
            mel.title,
            mel.status,
            mel.link,
            mel.exceptions,
            mel.proof,
            mel.flame_project_id,
            mel.inserted_at,
            mel.inserted_by,
            mel.updated_at,
            mel.updated_by
        FROM moderation_external_licenses mel
        WHERE ($1::text IS NULL OR mel.title ILIKE '%' || $1 || '%')
          AND (
            ($2::integer IS NULL AND $3::integer[] IS NULL)
            OR mel.flame_project_id = $2
            OR mel.flame_project_id = ANY($3)
          )
        ORDER BY mel.id
        "#,
        body.title,
        body.flame_id,
        body.flame_ids.as_deref(),
    )
    .fetch_all(&**pool)
    .await?;

    let license_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let files_map = fetch_linked_files(&pool, &license_ids).await?;

    let results = rows
        .into_iter()
        .map(|row| {
            let linked_files =
                files_map.get(&row.id).cloned().unwrap_or_default();
            row.into_external_project(linked_files)
        })
        .collect();

    Ok(web::Json(results))
}

/// Look up external license metadata.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = ExternalLicenseLookupResponse))
)]
#[post("/lookup")]
pub async fn lookup(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<ExternalLicenseLookupRequest>,
) -> Result<web::Json<ExternalLicenseLookupResponse>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let body = body.into_inner();
    let hashes = normalize_sha1_hashes(&body.hashes)?;
    let flame_ids = fetch_by_flame_ids(&pool, &body.flame_ids).await?;
    let hashes = fetch_by_hashes(&pool, &hashes).await?;

    Ok(web::Json(ExternalLicenseLookupResponse {
        flame_ids,
        hashes,
    }))
}

/// Get external license by SHA-1.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = ExternalProject))
)]
#[get("/by-sha1/{sha1}")]
pub async fn get_by_sha1(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(String,)>,
) -> Result<web::Json<ExternalProject>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let hashes = normalize_sha1_hashes(&[path.into_inner().0])?;
    let hash = hashes.first().ok_or(ApiError::NotFound)?;
    let mut results = fetch_by_hashes(&pool, &hashes).await?;
    let result = results.remove(hash).ok_or(ApiError::NotFound)?;

    Ok(web::Json(result))
}

/// Get external licenses by SHA-1.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = inline(HashMap<String, ExternalProject>)))
)]
#[post("/by-sha1")]
pub async fn get_by_sha1_bulk(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<HashLookupRequest>,
) -> Result<web::Json<HashMap<String, ExternalProject>>, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let hashes = normalize_sha1_hashes(&body.hashes)?;
    let results = fetch_by_hashes(&pool, &hashes).await?;

    Ok(web::Json(results))
}

/// Add an external license file.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = ExternalProject))
)]
#[post("/file")]
pub async fn add_file(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<FileLicenseRequest>,
) -> Result<web::Json<ExternalProject>, ApiError> {
    upsert_file_license(req, pool, redis, session_queue, body).await
}

/// Reassign an external license file.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = ExternalProject))
)]
#[post("/file/reassign")]
pub async fn reassign_file(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<FileLicenseRequest>,
) -> Result<web::Json<ExternalProject>, ApiError> {
    upsert_file_license(req, pool, redis, session_queue, body).await
}

async fn upsert_file_license(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<FileLicenseRequest>,
) -> Result<web::Json<ExternalProject>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let body = body.into_inner();
    let license_id = body.license_id.parse()?;
    if body.hashes.is_empty() {
        return Err(ApiError::InvalidInput(
            "hashes must contain at least one SHA1 hex string".to_string(),
        ));
    }
    let hashes = normalize_sha1_hashes(&body.hashes)?;
    let hash_bytes = hashes
        .iter()
        .map(|hash| hash.as_bytes().to_vec())
        .collect::<Vec<_>>();
    let filenames = vec![None; hashes.len()];
    let license_ids = vec![license_id; hashes.len()];

    let mut transaction = pool.begin().await?;

    let license = sqlx::query!(
        r#"
        SELECT
            id,
            title,
            status,
            link,
            exceptions,
            proof,
            flame_project_id,
            inserted_at,
            inserted_by,
            updated_at,
            updated_by
        FROM moderation_external_licenses
        WHERE id = $1
        "#,
        license_id,
    )
    .fetch_optional(&mut transaction)
    .await?
    .ok_or(ApiError::NotFound)?;

    ExternalLicense::insert_files(
        &mut transaction,
        &hash_bytes,
        &filenames,
        &license_ids,
        DBUserId(user.id.0 as i64),
    )
    .await?;

    transaction.commit().await?;

    let files_map = fetch_linked_files(&pool, &[license_id]).await?;
    let linked_files = files_map.get(&license_id).cloned().unwrap_or_default();

    Ok(web::Json(
        LicenseRow {
            id: license.id,
            title: license.title,
            status: license.status,
            link: license.link,
            exceptions: license.exceptions,
            proof: license.proof,
            flame_project_id: license.flame_project_id,
            inserted_at: license.inserted_at,
            inserted_by: license.inserted_by,
            updated_at: license.updated_at,
            updated_by: license.updated_by,
        }
        .into_external_project(linked_files),
    ))
}

/// Update an external license.  
#[utoipa::path(
	context_path = "/moderation/external-license",
	tag = "moderation",
	responses((status = OK, body = ExternalProject))
)]
#[patch("/{id}")]
pub async fn update_license(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    path: web::Path<(i64,)>,
    body: web::Json<UpdateLicenseRequest>,
) -> Result<web::Json<ExternalProject>, ApiError> {
    let user = check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    let id = path.into_inner().0;

    let result = sqlx::query!(
        r#"
        UPDATE moderation_external_licenses
        SET title = COALESCE($2, title),
            status = $3,
            link = COALESCE($4, link),
            exceptions = COALESCE($5, exceptions),
            proof = COALESCE($6, proof),
            flame_project_id = COALESCE($7, flame_project_id),
            updated_at = $8,
            updated_by = $9
        WHERE id = $1
        RETURNING id, title, status, link, exceptions, proof, flame_project_id,
                  inserted_at, inserted_by, updated_at, updated_by
        "#,
        id,
        body.title,
        body.status.as_str(),
        body.link,
        body.exceptions,
        body.proof,
        body.flame_project_id,
        Utc::now(),
        user.id.0 as i64,
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let files_map = fetch_linked_files(&pool, &[id]).await?;
    let linked_files = files_map.get(&id).cloned().unwrap_or_default();

    Ok(web::Json(
        LicenseRow {
            id: result.id,
            title: result.title,
            status: result.status,
            link: result.link,
            exceptions: result.exceptions,
            proof: result.proof,
            flame_project_id: result.flame_project_id,
            inserted_at: result.inserted_at,
            inserted_by: result.inserted_by,
            updated_at: result.updated_at,
            updated_by: result.updated_by,
        }
        .into_external_project(linked_files),
    ))
}
