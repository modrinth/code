use super::ApiError;
use crate::database::PgPool;
use crate::database::ReadOnlyPgPool;
use crate::database::redis::RedisPool;
use crate::models::projects::{Project, Version, VersionType};
use crate::models::v2::projects::{LegacyProject, LegacyVersion};
use crate::queue::session::AuthQueue;
use crate::routes::v3::version_file::HashQuery;
use crate::routes::{v2_reroute, v3};
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(
        utoipa_actix_web::scope("/version_file")
            .service(delete_file)
            .service(get_version_from_hash)
            .service(download_version)
            .service(get_update_from_hash)
            .service(get_projects_from_hashes),
    );

    cfg.service(
        utoipa_actix_web::scope("/version_files")
            .service(get_versions_from_hashes)
            .service(update_files)
            .service(update_files_many)
            .service(update_individual_files),
    );
}

// under /api/v1/version_file/{hash}
/// Get version metadata by file hash.
#[utoipa::path(
    get,
    operation_id = "versionFromHash",
    params(
        (
            "version_id" = String,
            Path,
            description = "The hexadecimal file hash"
        ),
        (
            "algorithm" = Option<String>,
            Query,
            description = "Hash algorithm to use (sha1 or sha512)"
        ),
        (
            "version_id" = Option<crate::models::ids::VersionId>,
            Query,
            description = "Optional version ID when hash maps to multiple files"
        )
    ),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{version_id}")]
pub async fn get_version_from_hash(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::version_file::get_version_from_hash(
        req,
        info,
        pool,
        redis,
        hash_query,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Version>(response).await {
        Ok(version) => {
            let v2_version = LegacyVersion::from(version);
            Ok(HttpResponse::Ok().json(v2_version))
        }
        Err(response) => Ok(response),
    }
}

// under /api/v1/version_file/{hash}/download
/// Download a file by hash.
#[utoipa::path(
    get,
    operation_id = "downloadVersionFromHash",
    params(
        (
            "version_id" = String,
            Path,
            description = "The hexadecimal file hash"
        ),
        (
            "algorithm" = Option<String>,
            Query,
            description = "Hash algorithm to use (sha1 or sha512)"
        ),
        (
            "version_id" = Option<crate::models::ids::VersionId>,
            Query,
            description = "Optional version ID when hash maps to multiple files"
        )
    ),
    responses(
        (status = 302, description = "Temporary redirect to file URL"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{version_id}/download")]
pub async fn download_version(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns TemporaryRedirect, so no need to convert to V2
    v3::version_file::download_version(
        req,
        info,
        pool,
        redis,
        hash_query,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

// under /api/v1/version_file/{hash}
/// Delete a file by hash.
#[utoipa::path(
    delete,
    operation_id = "deleteFileFromHash",
    params(
        (
            "version_id" = String,
            Path,
            description = "The hexadecimal file hash"
        ),
        (
            "algorithm" = Option<String>,
            Query,
            description = "Hash algorithm to use (sha1 or sha512)"
        ),
        (
            "version_id" = Option<crate::models::ids::VersionId>,
            Query,
            description = "Optional version ID to delete from"
        )
    ),
    responses(
        (status = 204, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["VERSION_WRITE"]))
)]
#[delete("/{version_id}")]
pub async fn delete_file(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert to V2
    v3::version_file::delete_file(
        req,
        info,
        pool,
        redis,
        hash_query,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateData {
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

/// Get the latest compatible version from a file hash.
#[utoipa::path(
    post,
    operation_id = "getLatestVersionFromHash",
    params(
        (
            "version_id" = String,
            Path,
            description = "The hexadecimal file hash"
        ),
        (
            "algorithm" = Option<String>,
            Query,
            description = "Hash algorithm to use (sha1 or sha512)"
        ),
        (
            "version_id" = Option<crate::models::ids::VersionId>,
            Query,
            description = "Optional version ID when hash maps to multiple files"
        )
    ),
    request_body = UpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[post("/{version_id}/update")]
pub async fn get_update_from_hash(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    hash_query: web::Query<HashQuery>,
    update_data: web::Json<UpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let update_data = update_data.into_inner();
    let mut loader_fields = HashMap::new();
    let mut game_versions = vec![];
    for gv in update_data.game_versions.into_iter().flatten() {
        game_versions.push(serde_json::json!(gv.clone()));
    }
    if !game_versions.is_empty() {
        loader_fields.insert("game_versions".to_string(), game_versions);
    }
    let update_data = v3::version_file::UpdateData {
        loaders: update_data.loaders.clone(),
        version_types: update_data.version_types.clone(),
        loader_fields: Some(loader_fields),
    };

    let response = v3::version_file::get_update_from_hash(
        req,
        info,
        pool,
        redis,
        hash_query,
        web::Json(update_data),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Version>(response).await {
        Ok(version) => {
            let v2_version = LegacyVersion::from(version);
            Ok(HttpResponse::Ok().json(v2_version))
        }
        Err(response) => Ok(response),
    }
}

// Requests above with multiple versions below
#[derive(Deserialize, utoipa::ToSchema)]
pub struct FileHashes {
    pub algorithm: Option<String>,
    pub hashes: Vec<String>,
}

// under /api/v2/version_files
/// Get versions from file hashes.
#[utoipa::path(
    post,
    operation_id = "versionsFromHashes",
    request_body = FileHashes,
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[post("")]
pub async fn get_versions_from_hashes(
    req: HttpRequest,
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    file_data: web::Json<FileHashes>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let file_data = file_data.into_inner();
    let file_data = v3::version_file::FileHashes {
        algorithm: file_data.algorithm,
        hashes: file_data.hashes,
    };
    let response = v3::version_file::get_versions_from_hashes(
        req,
        pool,
        redis,
        web::Json(file_data),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert to V2
    match v2_reroute::extract_ok_json::<HashMap<String, Version>>(response)
        .await
    {
        Ok(versions) => {
            let v2_versions = versions
                .into_iter()
                .map(|(hash, version)| {
                    let v2_version = LegacyVersion::from(version);
                    (hash, v2_version)
                })
                .collect::<HashMap<_, _>>();
            Ok(HttpResponse::Ok().json(v2_versions))
        }
        Err(response) => Ok(response),
    }
}

/// Get projects from file hashes.
#[utoipa::path(
    post,
    operation_id = "projectsFromHashes",
    request_body = FileHashes,
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[post("/project")]
pub async fn get_projects_from_hashes(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_data: web::Json<FileHashes>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let file_data = file_data.into_inner();
    let file_data = v3::version_file::FileHashes {
        algorithm: file_data.algorithm,
        hashes: file_data.hashes,
    };
    let response = v3::version_file::get_projects_from_hashes(
        req,
        pool.clone(),
        redis.clone(),
        web::Json(file_data),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert to V2
    match v2_reroute::extract_ok_json::<HashMap<String, Project>>(response)
        .await
    {
        Ok(projects_hashes) => {
            let hash_to_project_id = projects_hashes
                .iter()
                .map(|(hash, project)| {
                    let project_id = project.id;
                    (hash.clone(), project_id)
                })
                .collect::<HashMap<_, _>>();
            let legacy_projects = LegacyProject::from_many(
                projects_hashes.into_values().collect(),
                &**pool,
                &redis,
            )
            .await?;
            let legacy_projects_hashes = hash_to_project_id
                .into_iter()
                .filter_map(|(hash, project_id)| {
                    let legacy_project = legacy_projects
                        .iter()
                        .find(|x| x.id == project_id)?
                        .clone();
                    Some((hash, legacy_project))
                })
                .collect::<HashMap<_, _>>();

            Ok(HttpResponse::Ok().json(legacy_projects_hashes))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ManyUpdateData {
    pub algorithm: Option<String>, // Defaults to calculation based on size of hash
    pub hashes: Vec<String>,
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

/// Get latest compatible versions for multiple hashes.
#[utoipa::path(
    post,
    operation_id = "getLatestVersionsFromHashes",
    request_body = ManyUpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[post("/update")]
pub async fn update_files(
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
) -> Result<HttpResponse, ApiError> {
    let update_data = update_data.into_inner();
    let update_data = v3::version_file::ManyUpdateData {
        loaders: update_data.loaders.clone(),
        version_types: update_data.version_types.clone(),
        game_versions: update_data.game_versions.clone(),
        algorithm: update_data.algorithm,
        hashes: update_data.hashes,
    };

    let returned_versions = match v3::version_file::update_files(
        pool,
        redis,
        web::Json(update_data),
    )
    .await
    {
        Ok(resp) => resp,
        Err(ApiError::NotFound) => return Ok(HttpResponse::NotFound().body("")),
        Err(err) => return Err(err),
    };

    // Convert response to V2 format
    let v3_versions = returned_versions
        .0
        .into_iter()
        .map(|(hash, version)| {
            let v2_version = LegacyVersion::from(version);
            (hash, v2_version)
        })
        .collect::<HashMap<_, _>>();
    Ok(HttpResponse::Ok().json(v3_versions))
}

/// Get all latest compatible versions for multiple hashes.
#[utoipa::path(
    post,
    operation_id = "getLatestVersionsFromHashesMany",
    request_body = ManyUpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[post("/update_many")]
pub async fn update_files_many(
    pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyUpdateData>,
) -> Result<HttpResponse, ApiError> {
    let update_data = update_data.into_inner();
    let update_data = v3::version_file::ManyUpdateData {
        loaders: update_data.loaders.clone(),
        version_types: update_data.version_types.clone(),
        game_versions: update_data.game_versions.clone(),
        algorithm: update_data.algorithm,
        hashes: update_data.hashes,
    };

    let returned_versions = match v3::version_file::update_files_many(
        pool,
        redis,
        web::Json(update_data),
    )
    .await
    {
        Ok(resp) => resp,
        Err(ApiError::NotFound) => return Ok(HttpResponse::NotFound().body("")),
        Err(err) => return Err(err),
    };

    // Convert response to V2 format
    let v3_versions = returned_versions
        .0
        .into_iter()
        .map(|(hash, versions)| {
            let v2_versions = versions
                .into_iter()
                .map(LegacyVersion::from)
                .collect::<Vec<_>>();
            (hash, v2_versions)
        })
        .collect::<HashMap<_, _>>();
    Ok(HttpResponse::Ok().json(v3_versions))
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileUpdateData {
    pub hash: String,
    pub loaders: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub version_types: Option<Vec<VersionType>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ManyFileUpdateData {
    pub algorithm: Option<String>, // Defaults to calculation based on size of hash
    pub hashes: Vec<FileUpdateData>,
}

/// Get latest versions with per-hash filters.
#[utoipa::path(
    post,
    operation_id = "getLatestVersionsFromHashesIndividual",
    request_body = ManyFileUpdateData,
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[post("/update_individual")]
pub async fn update_individual_files(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    update_data: web::Json<ManyFileUpdateData>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let update_data = update_data.into_inner();
    let update_data = v3::version_file::ManyFileUpdateData {
        algorithm: update_data.algorithm,
        hashes: update_data
            .hashes
            .into_iter()
            .map(|x| {
                let mut loader_fields = HashMap::new();
                let mut game_versions = vec![];
                for gv in x.game_versions.into_iter().flatten() {
                    game_versions.push(serde_json::json!(gv.clone()));
                }
                if !game_versions.is_empty() {
                    loader_fields
                        .insert("game_versions".to_string(), game_versions);
                }
                v3::version_file::FileUpdateData {
                    hash: x.hash.clone(),
                    loaders: x.loaders.clone(),
                    loader_fields: Some(loader_fields),
                    version_types: x.version_types,
                }
            })
            .collect(),
    };

    let response = v3::version_file::update_individual_files(
        req,
        pool,
        redis,
        web::Json(update_data),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<HashMap<String, Version>>(response)
        .await
    {
        Ok(returned_versions) => {
            let v3_versions = returned_versions
                .into_iter()
                .map(|(hash, version)| {
                    let v2_version = LegacyVersion::from(version);
                    (hash, v2_version)
                })
                .collect::<HashMap<_, _>>();
            Ok(HttpResponse::Ok().json(v3_versions))
        }
        Err(response) => Ok(response),
    }
}
