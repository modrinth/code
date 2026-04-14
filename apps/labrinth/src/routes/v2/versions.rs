use std::collections::HashMap;

use super::ApiError;
use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::models;
use crate::models::ids::VersionId;
use crate::models::projects::{
    Dependency, FileType, Version, VersionStatus, VersionType,
};
use crate::models::v2::projects::LegacyVersion;
use crate::queue::session::AuthQueue;
use crate::routes::{v2_reroute, v3};
use crate::search::SearchBackend;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, web};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(versions_get);
    cfg.service(super::version_creation::version_create);
    cfg.service(
        utoipa_actix_web::scope("/version")
            .service(version_get)
            .service(version_delete)
            .service(version_edit)
            .service(super::version_creation::upload_file_to_version),
    );
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct VersionListFilters {
    pub game_versions: Option<String>,
    pub loaders: Option<String>,
    pub featured: Option<bool>,
    pub version_type: Option<VersionType>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(default = "default_true")]
    pub include_changelog: bool,
}

fn default_true() -> bool {
    true
}

/// List versions for a project.
#[utoipa::path(
    get,
    operation_id = "getProjectVersions",
    params(
        (
            "project_id" = String,
            Path,
            description = "The ID or slug of the project"
        ),
        (
            "loaders" = Option<String>,
            Query,
            description = "JSON array of loaders to filter by"
        ),
        (
            "game_versions" = Option<String>,
            Query,
            description = "JSON array of game versions to filter by"
        ),
        (
            "featured" = Option<bool>,
            Query,
            description = "Filter by featured status"
        ),
        (
            "include_changelog" = Option<bool>,
            Query,
            description = "Whether to include changelog fields"
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
#[get("/version")]
pub async fn version_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    web::Query(filters): web::Query<VersionListFilters>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let loaders = if let Some(loaders) = filters.loaders {
        if let Ok(mut loaders) = serde_json::from_str::<Vec<String>>(&loaders) {
            loaders.push("mrpack".to_string());
            Some(loaders)
        } else {
            None
        }
    } else {
        None
    };

    let loader_fields = if let Some(game_versions) = filters.game_versions {
        // TODO: extract this logic which is similar to the other v2->v3 version_file functions
        let mut loader_fields = HashMap::new();
        serde_json::from_str::<Vec<String>>(&game_versions)
            .ok()
            .and_then(|versions| {
                let mut game_versions: Vec<serde_json::Value> = vec![];
                for gv in versions {
                    game_versions.push(serde_json::json!(gv.clone()));
                }
                loader_fields
                    .insert("game_versions".to_string(), game_versions);

                if let Some(ref loaders) = loaders {
                    loader_fields.insert(
                        "loaders".to_string(),
                        loaders
                            .iter()
                            .map(|x| serde_json::json!(x.clone()))
                            .collect(),
                    );
                }

                serde_json::to_string(&loader_fields).ok()
            })
    } else {
        None
    };

    let filters = v3::versions::VersionListFilters {
        loader_fields,
        loaders: loaders.and_then(|x| serde_json::to_string(&x).ok()),
        featured: filters.featured,
        version_type: filters.version_type,
        limit: filters.limit,
        offset: filters.offset,
        include_changelog: filters.include_changelog,
    };

    let response = v3::versions::version_list_internal(
        req,
        info,
        web::Query(filters),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Version>>(response).await {
        Ok(versions) => {
            let v2_versions = versions
                .into_iter()
                .map(LegacyVersion::from)
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(v2_versions))
        }
        Err(response) => Ok(response),
    }
}

// Given a project ID/slug and a version slug
/// Get a project version by ID or version number.
#[utoipa::path(
    get,
    operation_id = "getVersionFromIdOrNumber",
    params(
        (
            "project_id" = String,
            Path,
            description = "The ID or slug of the project"
        ),
        (
            "slug" = String,
            Path,
            description = "The version ID or version number"
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
#[get("/version/{slug}")]
pub async fn version_project_get(
    req: HttpRequest,
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner();
    let response = v3::versions::version_project_get_helper(
        req,
        id,
        pool,
        redis,
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

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct VersionIds {
    pub ids: String,
    #[serde(default = "default_true")]
    pub include_changelog: bool,
}

/// Get multiple versions by ID.
#[utoipa::path(
    get,
    operation_id = "getVersions",
    params(("ids" = String, Query, description = "The JSON array of version IDs")),
    responses((status = 200, description = "Expected response to a valid request"))
)]
#[get("/versions")]
pub async fn versions_get(
    req: HttpRequest,
    web::Query(ids): web::Query<VersionIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = v3::versions::VersionIds {
        ids: ids.ids,
        include_changelog: ids.include_changelog,
    };
    let response = v3::versions::versions_get(
        req,
        web::Query(ids),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Version>>(response).await {
        Ok(versions) => {
            let v2_versions = versions
                .into_iter()
                .map(LegacyVersion::from)
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(v2_versions))
        }
        Err(response) => Ok(response),
    }
}

/// Get a version by ID.
#[utoipa::path(
    get,
    operation_id = "getVersion",
    params(("version_id" = models::ids::VersionId, Path, description = "The ID of the version")),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{version_id}")]
pub async fn version_get(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let response =
        v3::versions::version_get_helper(req, id, pool, redis, session_queue)
            .await
            .map(|b| HttpResponse::Ok().json(b))
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

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EditVersion {
    #[validate(
        length(min = 1, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub name: Option<String>,
    #[validate(
        length(min = 1, max = 32),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    pub version_number: Option<String>,
    #[validate(length(max = 65536))]
    pub changelog: Option<String>,
    pub version_type: Option<models::projects::VersionType>,
    #[validate(
        length(min = 0, max = 4096),
        custom(function = "crate::util::validate::validate_deps")
    )]
    pub dependencies: Option<Vec<Dependency>>,
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<models::projects::Loader>>,
    pub featured: Option<bool>,
    pub downloads: Option<u32>,
    pub status: Option<VersionStatus>,
    pub file_types: Option<Vec<EditVersionFileType>>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct EditVersionFileType {
    pub algorithm: String,
    pub hash: String,
    pub file_type: Option<FileType>,
}

/// Modify an existing version.
#[utoipa::path(
    patch,
    operation_id = "modifyVersion",
    params(("id" = VersionId, Path, description = "The ID of the version")),
    request_body = EditVersion,
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
#[patch("/{id}")]
pub async fn version_edit(
    req: HttpRequest,
    info: web::Path<(VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    new_version: web::Json<EditVersion>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_version = new_version.into_inner();

    let mut fields = HashMap::new();
    if new_version.game_versions.is_some() {
        fields.insert(
            "game_versions".to_string(),
            serde_json::json!(new_version.game_versions),
        );
    }

    // Get the older version to get info from
    let old_version = match v3::versions::version_get_helper(
        req.clone(),
        (*info).0,
        pool.clone(),
        redis.clone(),
        session_queue.clone(),
    )
    .await
    {
        Ok(resp) => resp,
        Err(ApiError::NotFound) => return Ok(HttpResponse::NotFound().body("")),
        Err(err) => return Err(err),
    };
    let old_version = match v2_reroute::extract_ok_json::<Version>(
        HttpResponse::Ok().json(old_version.0),
    )
    .await
    {
        Ok(version) => version,
        Err(response) => return Ok(response),
    };

    // If this has 'mrpack_loaders' as a loader field previously, this is a modpack.
    // Therefore, if we are modifying the 'loader' field in this case,
    // we are actually modifying the 'mrpack_loaders' loader field
    let mut loaders = new_version.loaders.clone();
    if old_version.fields.contains_key("mrpack_loaders")
        && new_version.loaders.is_some()
    {
        fields.insert(
            "mrpack_loaders".to_string(),
            serde_json::json!(new_version.loaders),
        );
        loaders = None;
    }

    let new_version = v3::versions::EditVersion {
        name: new_version.name,
        version_number: new_version.version_number,
        changelog: new_version.changelog,
        version_type: new_version.version_type,
        dependencies: new_version.dependencies,
        loaders,
        featured: new_version.featured,
        downloads: new_version.downloads,
        status: new_version.status,
        file_types: new_version.file_types.map(|v| {
            v.into_iter()
                .map(|evft| v3::versions::EditVersionFileType {
                    algorithm: evft.algorithm,
                    hash: evft.hash,
                    file_type: evft.file_type,
                })
                .collect::<Vec<_>>()
        }),
        ordering: None,
        fields,
    };

    let response = v3::versions::version_edit(
        req,
        info,
        pool,
        redis,
        web::Json(serde_json::to_value(new_version)?),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;
    Ok(response)
}

/// Delete a version by ID.
#[utoipa::path(
    delete,
    operation_id = "deleteVersion",
    params(("version_id" = VersionId, Path, description = "The ID of the version")),
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
    security(("bearer_auth" = ["VERSION_DELETE"]))
)]
#[delete("/{version_id}")]
pub async fn version_delete(
    req: HttpRequest,
    info: web::Path<(VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_backend: web::Data<dyn SearchBackend>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::versions::version_delete(
        req,
        info,
        pool,
        redis,
        session_queue,
        search_backend,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}
