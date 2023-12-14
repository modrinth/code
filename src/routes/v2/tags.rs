use std::collections::HashMap;

use super::ApiError;
use crate::database::models::loader_fields::LoaderFieldEnumValue;
use crate::database::redis::RedisPool;
use crate::models::v2::projects::LegacySideType;
use crate::routes::v2_reroute::capitalize_first;
use crate::routes::v3::tags::{
    LinkPlatformQueryData, LoaderData as LoaderDataV3, LoaderFieldsEnumQuery,
};
use crate::routes::{v2_reroute, v3};
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("tag")
            .service(category_list)
            .service(loader_list)
            .service(game_version_list)
            .service(license_list)
            .service(license_text)
            .service(donation_platform_list)
            .service(report_type_list)
            .service(project_type_list)
            .service(side_type_list),
    );
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CategoryData {
    pub icon: String,
    pub name: String,
    pub project_type: String,
    pub header: String,
}

#[get("category")]
pub async fn category_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    v3::tags::category_list(pool, redis).await
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoaderData {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,
}

#[get("loader")]
pub async fn loader_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::tags::loader_list(pool, redis).await?;

    // Convert to V2 format
    match v2_reroute::extract_ok_json::<Vec<LoaderDataV3>>(response).await {
        Ok(loaders) => {
            let loaders = loaders
                .into_iter()
                .map(|l| LoaderData {
                    icon: l.icon,
                    name: l.name,
                    // Add generic 'project' type to all loaders, which is the v2 representation of
                    // a project type before any versions are set.
                    supported_project_types: l
                        .supported_project_types
                        .into_iter()
                        .chain(std::iter::once("project".to_string()))
                        .collect(),
                })
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(loaders))
        }
        Err(response) => Ok(response),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameVersionQueryData {
    pub version: String,
    pub version_type: String,
    pub date: DateTime<Utc>,
    pub major: bool,
}

#[derive(serde::Deserialize)]
pub struct GameVersionQuery {
    #[serde(rename = "type")]
    type_: Option<String>,
    major: Option<bool>,
}

#[get("game_version")]
pub async fn game_version_list(
    pool: web::Data<PgPool>,
    query: web::Query<GameVersionQuery>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let mut filters = HashMap::new();
    if let Some(type_) = &query.type_ {
        filters.insert("type".to_string(), serde_json::json!(type_));
    }
    if let Some(major) = query.major {
        filters.insert("major".to_string(), serde_json::json!(major));
    }
    let response = v3::tags::loader_fields_list(
        pool,
        web::Query(LoaderFieldsEnumQuery {
            loader_field: "game_versions".to_string(),
            filters: Some(filters),
        }),
        redis,
    )
    .await?;

    // Convert to V2 format
    Ok(
        match v2_reroute::extract_ok_json::<Vec<LoaderFieldEnumValue>>(response).await {
            Ok(fields) => {
                let fields = fields
                    .into_iter()
                    .map(|f| GameVersionQueryData {
                        version: f.value,
                        version_type: f
                            .metadata
                            .get("type")
                            .and_then(|m| m.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        date: f.created,
                        major: f
                            .metadata
                            .get("major")
                            .and_then(|m| m.as_bool())
                            .unwrap_or_default(),
                    })
                    .collect::<Vec<_>>();
                HttpResponse::Ok().json(fields)
            }
            Err(response) => response,
        },
    )
}

#[derive(serde::Serialize)]
pub struct License {
    short: String,
    name: String,
}

#[get("license")]
pub async fn license_list() -> HttpResponse {
    v3::tags::license_list().await
}

#[derive(serde::Serialize)]
pub struct LicenseText {
    title: String,
    body: String,
}

#[get("license/{id}")]
pub async fn license_text(params: web::Path<(String,)>) -> Result<HttpResponse, ApiError> {
    v3::tags::license_text(params)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub struct DonationPlatformQueryData {
    // The difference between name and short is removed in v3.
    // Now, the 'id' becomes the name, and the 'name' is removed (the frontend uses the id as the name)
    // pub short: String,
    pub short: String,
    pub name: String,
}

#[get("donation_platform")]
pub async fn donation_platform_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::tags::link_platform_list(pool, redis).await?;

    // Convert to V2 format
    Ok(
        match v2_reroute::extract_ok_json::<Vec<LinkPlatformQueryData>>(response).await {
            Ok(platforms) => {
                let platforms = platforms
                    .into_iter()
                    .filter_map(|p| {
                        if p.donation {
                            Some(DonationPlatformQueryData {
                                // Short vs name is no longer a recognized difference in v3.
                                // We capitalize to recreate the old behavior, with some special handling.
                                // This may result in different behaviour for platforms added after the v3 migration.
                                name: match p.name.as_str() {
                                    "bmac" => "Buy Me A Coffee".to_string(),
                                    "github" => "GitHub Sponsors".to_string(),
                                    "ko-fi" => "Ko-fi".to_string(),
                                    "paypal" => "PayPal".to_string(),
                                    // Otherwise, capitalize it
                                    _ => capitalize_first(&p.name),
                                },
                                short: p.name,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                HttpResponse::Ok().json(platforms)
            }
            Err(response) => response,
        },
    )
    .or_else(v2_reroute::flatten_404_error)
}

#[get("report_type")]
pub async fn report_type_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    v3::tags::report_type_list(pool, redis)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[get("project_type")]
pub async fn project_type_list(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    v3::tags::project_type_list(pool, redis)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[get("side_type")]
pub async fn side_type_list() -> Result<HttpResponse, ApiError> {
    // Original side types are no longer reflected in the database.
    // Therefore, we hardcode and return all the fields that are supported by our v2 conversion logic.
    let side_types = [
        LegacySideType::Required,
        LegacySideType::Optional,
        LegacySideType::Unsupported,
        LegacySideType::Unknown,
    ];
    let side_types = side_types.iter().map(|s| s.to_string()).collect_vec();
    Ok(HttpResponse::Ok().json(side_types))
}
