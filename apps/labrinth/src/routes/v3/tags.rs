use std::collections::HashMap;

use super::ApiError;
use crate::database::models::categories::{
    Category, LinkPlatform, ProjectType, ReportType,
};
use crate::database::models::loader_fields::{
    Game, Loader, LoaderField, LoaderFieldEnumValue, LoaderFieldType,
};
use crate::database::redis::RedisPool;
use actix_web::{HttpResponse, get, web};

use crate::database::PgPool;
use itertools::Itertools;
use serde_json::Value;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(category_list_route)
        .service(loader_list_route)
        .service(games_list_route)
        .service(loader_fields_list_route)
        .service(license_list_route)
        .service(license_text_route)
        .service(link_platform_list_route)
        .service(report_type_list_route)
        .service(project_type_list_route);
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameData {
    pub slug: String,
    pub name: String,
    pub icon: Option<String>,
    pub banner: Option<String>,
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/games")]
pub async fn games_list_route(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    games_list(pool, redis).await
}

pub async fn games_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let results = Game::list(&**pool, &redis)
        .await?
        .into_iter()
        .map(|x| GameData {
            slug: x.slug,
            name: x.name,
            icon: x.icon_url,
            banner: x.banner_url,
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CategoryData {
    pub icon: String,
    pub name: String,
    pub project_type: String,
    pub header: String,
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/tag/category")]
pub async fn category_list_route(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    category_list(pool, redis).await
}

pub async fn category_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let results = Category::list(&**pool, &redis)
        .await?
        .into_iter()
        .map(|x| CategoryData {
            icon: x.icon,
            name: x.category,
            project_type: x.project_type,
            header: x.header,
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoaderData {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,
    pub supported_games: Vec<String>,
    pub supported_fields: Vec<String>, // Available loader fields for this loader
    pub metadata: Value,
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/tag/loader")]
pub async fn loader_list_route(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    loader_list(pool, redis).await
}

pub async fn loader_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let loaders = Loader::list(&**pool, &redis).await?;

    let loader_fields = LoaderField::get_fields_per_loader(
        &loaders.iter().map(|x| x.id).collect_vec(),
        &**pool,
        &redis,
    )
    .await?;

    let mut results = loaders
        .into_iter()
        .map(|x| LoaderData {
            icon: x.icon,
            name: x.loader,
            supported_project_types: x.supported_project_types,
            supported_games: x.supported_games,
            supported_fields: loader_fields
                .get(&x.id)
                .map(|x| x.iter().map(|x| x.field.clone()).collect_vec())
                .unwrap_or_default(),
            metadata: x.metadata,
        })
        .collect::<Vec<_>>();

    results.sort_by_key(|a| a.name.to_lowercase());

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LoaderFieldsEnumQuery {
    pub loader_field: String,
    pub filters: Option<HashMap<String, Value>>, // For metadata
}

// Provides the variants for any enumerable loader field.
#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/loader_field")]
pub async fn loader_fields_list_route(
    pool: web::Data<PgPool>,
    web::Query(query): web::Query<LoaderFieldsEnumQuery>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    loader_fields_list(pool, web::Query(query), redis).await
}

pub async fn loader_fields_list(
    pool: web::Data<PgPool>,
    query: web::Query<LoaderFieldsEnumQuery>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();
    let loader_field = LoaderField::get_fields_all(&**pool, &redis)
        .await?
        .into_iter()
        .find(|x| x.field == query.loader_field)
        .ok_or_else(|| {
            ApiError::InvalidInput(format!(
                "'{}' was not a valid loader field.",
                query.loader_field
            ))
        })?;

    let (LoaderFieldType::Enum(loader_field_enum_id)
    | LoaderFieldType::ArrayEnum(loader_field_enum_id)) =
        loader_field.field_type
    else {
        return Err(ApiError::InvalidInput(format!(
            "'{}' is not an enumerable field, but an '{}' field.",
            query.loader_field,
            loader_field.field_type.to_str()
        )));
    };

    let results: Vec<_> = if let Some(filters) = query.filters {
        LoaderFieldEnumValue::list_filter(
            loader_field_enum_id,
            filters,
            &**pool,
            &redis,
        )
        .await?
    } else {
        LoaderFieldEnumValue::list(loader_field_enum_id, &**pool, &redis)
            .await?
    };

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct License {
    pub short: String,
    pub name: String,
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/license")]
pub async fn license_list_route() -> HttpResponse {
    license_list().await
}

pub async fn license_list() -> HttpResponse {
    let licenses = spdx::identifiers::LICENSES;
    let mut results: Vec<License> = Vec::with_capacity(licenses.len());

    for (short, name, _) in licenses {
        results.push(License {
            short: short.to_string(),
            name: name.to_string(),
        });
    }

    HttpResponse::Ok().json(results)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LicenseText {
    pub title: String,
    pub body: String,
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/license/{id}")]
pub async fn license_text_route(
    params: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    license_text(params).await
}

pub async fn license_text(
    params: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    let license_id = params.into_inner().0;

    if license_id == *crate::models::projects::DEFAULT_LICENSE_ID {
        return Ok(HttpResponse::Ok().json(LicenseText {
            title: "All Rights Reserved".to_string(),
            body: "All rights reserved unless explicitly stated.".to_string(),
        }));
    }

    if let Some(license) = spdx::license_id(&license_id) {
        return Ok(HttpResponse::Ok().json(LicenseText {
            title: license.full_name.to_string(),
            body: license.text().to_string(),
        }));
    }

    Err(ApiError::InvalidInput(
        "Invalid SPDX identifier specified".to_string(),
    ))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LinkPlatformQueryData {
    pub name: String,
    pub donation: bool,
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/link_platform")]
pub async fn link_platform_list_route(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    link_platform_list(pool, redis).await
}

pub async fn link_platform_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let results: Vec<LinkPlatformQueryData> =
        LinkPlatform::list(&**pool, &redis)
            .await?
            .into_iter()
            .map(|x| LinkPlatformQueryData {
                name: x.name,
                donation: x.donation,
            })
            .collect();
    Ok(HttpResponse::Ok().json(results))
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/report_type")]
pub async fn report_type_list_route(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    report_type_list(pool, redis).await
}

pub async fn report_type_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let results = ReportType::list(&**pool, &redis).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[utoipa::path(tag = "tags", responses((status = OK)))]
#[get("/project_type")]
pub async fn project_type_list_route(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    project_type_list(pool, redis).await
}

pub async fn project_type_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let results = ProjectType::list(&**pool, &redis).await?;
    Ok(HttpResponse::Ok().json(results))
}
