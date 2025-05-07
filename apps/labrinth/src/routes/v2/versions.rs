use std::collections::HashMap;

use super::ApiError;
use crate::database::redis::RedisPool;
use crate::models;
use crate::models::ids::VersionId;
use crate::models::projects::{
    Dependency, FileType, Version, VersionStatus, VersionType,
};
use crate::models::v2::projects::LegacyVersion;
use crate::queue::session::AuthQueue;
use crate::routes::{v2_reroute, v3};
use crate::search::SearchConfig;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(versions_get);
    cfg.service(super::version_creation::version_create);

    cfg.service(
        web::scope("version")
            .service(version_get)
            .service(version_delete)
            .service(version_edit)
            .service(super::version_creation::upload_file_to_version),
    );
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VersionListFilters {
    pub game_versions: Option<String>,
    pub loaders: Option<String>,
    pub featured: Option<bool>,
    pub version_type: Option<VersionType>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[get("version")]
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
    };

    let response = v3::versions::version_list(
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
#[get("version/{slug}")]
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

#[derive(Serialize, Deserialize)]
pub struct VersionIds {
    pub ids: String,
}

#[get("versions")]
pub async fn versions_get(
    req: HttpRequest,
    web::Query(ids): web::Query<VersionIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = v3::versions::VersionIds { ids: ids.ids };
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

#[get("{version_id}")]
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

#[derive(Serialize, Deserialize, Validate)]
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

#[derive(Serialize, Deserialize)]
pub struct EditVersionFileType {
    pub algorithm: String,
    pub hash: String,
    pub file_type: Option<FileType>,
}

#[patch("{id}")]
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
    let old_version = v3::versions::version_get_helper(
        req.clone(),
        (*info).0,
        pool.clone(),
        redis.clone(),
        session_queue.clone(),
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;
    let old_version =
        match v2_reroute::extract_ok_json::<Version>(old_version).await {
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

#[delete("{version_id}")]
pub async fn version_delete(
    req: HttpRequest,
    info: web::Path<(VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_config: web::Data<SearchConfig>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so we don't need to convert the response
    v3::versions::version_delete(
        req,
        info,
        pool,
        redis,
        session_queue,
        search_config,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}
