use actix_web::{HttpRequest, get, patch, post, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::queue::moderation::ApprovalType;
use crate::routes::ApiError;
use crate::{auth::check_is_moderator_from_headers, queue::session::AuthQueue};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(search)
        .service(get_by_sha1)
        .service(update_license);
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

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct LinkedFile {
    pub name: Option<String>,
    pub sha1: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SearchRequest {
    pub title: Option<String>,
    pub flame_id: Option<i32>,
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

#[utoipa::path]
#[post("/search")]
async fn search(
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

    let rows = sqlx::query!(
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
            mel.updated_by,
            COALESCE(
                jsonb_agg(
                    jsonb_build_object(
                        'name', f.filename,
                        'sha1', encode(mef.sha1, 'escape')
                    )
                ) FILTER (WHERE f.id IS NOT NULL),
                '[]'::jsonb
            ) AS linked_files
        FROM moderation_external_licenses mel
        LEFT JOIN moderation_external_files mef ON mef.external_license_id = mel.id
        LEFT JOIN hashes h ON h.algorithm = 'sha1' AND h.hash = mef.sha1
        LEFT JOIN files f ON f.id = h.file_id
        WHERE ($1::text IS NULL OR mel.title ILIKE '%' || $1 || '%')
          AND ($2::integer IS NULL OR mel.flame_project_id = $2)
        GROUP BY mel.id
        ORDER BY mel.id
        "#,
        body.title,
        body.flame_id,
    )
    .fetch_all(&**pool)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| {
            let linked_files: Vec<LinkedFile> =
                serde_json::from_value(row.linked_files.unwrap_or_default())
                    .unwrap_or_default();

            ExternalProject {
                id: row.id,
                title: row.title,
                status: ApprovalType::from_string(&row.status)
                    .unwrap_or(ApprovalType::Unidentified),
                link: row.link,
                exceptions: row.exceptions,
                proof: row.proof,
                flame_project_id: row.flame_project_id,
                inserted_at: row.inserted_at,
                inserted_by: row.inserted_by,
                updated_at: row.updated_at,
                updated_by: row.updated_by,
                linked_files,
            }
        })
        .collect();

    Ok(web::Json(results))
}

#[utoipa::path]
#[get("/by-sha1/{sha1}")]
async fn get_by_sha1(
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

    let sha1 = path.into_inner().0;

    let row = sqlx::query!(
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
            mel.updated_by,
            COALESCE(
                jsonb_agg(
                    jsonb_build_object(
                        'name', f.filename,
                        'sha1', encode(mef_all.sha1, 'escape')
                    )
                ) FILTER (WHERE f.id IS NOT NULL),
                '[]'::jsonb
            ) AS linked_files
        FROM moderation_external_files mef
        INNER JOIN moderation_external_licenses mel ON mel.id = mef.external_license_id
        LEFT JOIN moderation_external_files mef_all ON mef_all.external_license_id = mel.id
        LEFT JOIN hashes h ON h.algorithm = 'sha1' AND h.hash = mef_all.sha1
        LEFT JOIN files f ON f.id = h.file_id
        WHERE mef.sha1 = $1
        GROUP BY mel.id
        "#,
        sha1.as_bytes().to_vec(),
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let linked_files: Vec<LinkedFile> =
        serde_json::from_value(row.linked_files.unwrap_or_default())
            .unwrap_or_default();

    Ok(web::Json(ExternalProject {
        id: row.id,
        title: row.title,
        status: ApprovalType::from_string(&row.status)
            .unwrap_or(ApprovalType::Unidentified),
        link: row.link,
        exceptions: row.exceptions,
        proof: row.proof,
        flame_project_id: row.flame_project_id,
        inserted_at: row.inserted_at,
        inserted_by: row.inserted_by,
        updated_at: row.updated_at,
        updated_by: row.updated_by,
        linked_files,
    }))
}

#[utoipa::path]
#[patch("/{id}")]
async fn update_license(
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

    let files = sqlx::query!(
        r#"
        SELECT
            f.filename AS name,
            encode(mef.sha1, 'escape') AS sha1
        FROM moderation_external_files mef
        LEFT JOIN hashes h ON h.algorithm = 'sha1' AND h.hash = mef.sha1
        LEFT JOIN files f ON f.id = h.file_id
        WHERE mef.external_license_id = $1
        "#,
        id,
    )
    .fetch_all(&**pool)
    .await?;

    let linked_files = files
        .into_iter()
        .map(|f| LinkedFile {
            name: Some(f.name),
            sha1: f.sha1.unwrap_or_default(),
        })
        .collect();

    Ok(web::Json(ExternalProject {
        id: result.id,
        title: result.title,
        status: ApprovalType::from_string(&result.status)
            .unwrap_or(ApprovalType::Unidentified),
        link: result.link,
        exceptions: result.exceptions,
        proof: result.proof,
        flame_project_id: result.flame_project_id,
        inserted_at: result.inserted_at,
        inserted_by: result.inserted_by,
        updated_at: result.updated_at,
        updated_by: result.updated_by,
        linked_files,
    }))
}
