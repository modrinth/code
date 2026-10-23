use crate::util::error::Context as _;
use crate::util::MinecraftMeta;
use std::collections::HashMap;

use super::ApiError;
use crate::database::models::categories::{Category, ProjectType, ReportType};
use crate::database::models::loader_item::{Loader, LoaderMetadata};
use crate::models::exp::minecraft::{Environment, MrpackLoader};
use crate::models::link_platform::LinkPlatform;
use crate::models::minecraft::{
	legacy_game_version_date, legacy_game_version_name,
};
use actix_web::{HttpResponse, get, web};
use chrono::{DateTime, Utc};
use minecraft_meta_model::upstream::mojang::VersionType;
use strum::IntoEnumIterator;
use xredis::RedisPool;

use crate::database::PgPool;

use serde_json::Value;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(category_list_route)
        .service(loader_list_route)
        .service(loader_fields_list_route)
        .service(license_list_route)
        .service(license_text_route)
        .service(link_platform_list_route)
        .service(report_type_list_route)
        .service(project_type_list_route);
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
        .await
        .wrap_internal_err("fetching category from Redis")?
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
    pub metadata: LoaderMetadata,
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
    let loaders = Loader::list(&**pool, &redis)
        .await
        .wrap_internal_err("fetching loader from Redis")?;

    let mut results = loaders
        .into_iter()
        .map(|loader| {
			let mut supported_fields = vec![
				"game_versions".to_owned(),
				"environment".to_owned(),
			];
			if loader.loader == "mrpack" {
				supported_fields.push("mrpack_loaders".to_owned());
			}
            LoaderData {
                icon: loader.icon,
                name: loader.loader,
                supported_project_types: loader.supported_project_types,
				supported_games: vec!["minecraft-java".to_owned()],
                supported_fields,
                metadata: loader.metadata,
            }
        })
        .collect::<Vec<_>>();

    results.sort_by_key(|a| a.name.to_lowercase());

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MinecraftTagValue {
	pub value: String,
	pub ordering: Option<i32>,
	pub created: DateTime<Utc>,
	#[serde(rename = "type")]
	pub ty: Option<String>,
	pub major: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LoaderFieldsEnumQuery {
    pub loader_field: String,
    pub filters: Option<HashMap<String, Value>>, // For metadata
}

// Provides the variants for any enumerable loader field.
#[utoipa::path(
	tag = "tags",
	params(
		("loader_field" = String, Query),
		("filters" = Option<HashMap<String, Value>>, Query)
	),
	responses((status = OK))
)]
#[get("/loader_field")]
pub async fn loader_fields_list_route(
    minecraft_meta: web::Data<MinecraftMeta>,
    web::Query(query): web::Query<LoaderFieldsEnumQuery>,
) -> Result<HttpResponse, ApiError> {
    loader_fields_list(minecraft_meta, web::Query(query)).await
}

pub async fn loader_fields_list(
    minecraft_meta: web::Data<MinecraftMeta>,
    query: web::Query<LoaderFieldsEnumQuery>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();
    let mut results = match query.loader_field.as_str() {
		"game_versions" => {
			let versions = minecraft_meta
				.minecraft_versions()
				.await
				.wrap_internal_err("fetching minecraft-meta game versions")?;
			versions
				.iter()
				.map(|row| {
					let name = row.name.to_string();
					let entry = &row.catalog_entry.0;
					let ty = match entry.ty {
						VersionType::Release => "release",
						VersionType::Snapshot => "snapshot",
						VersionType::OldAlpha => "alpha",
						VersionType::OldBeta => "beta",
					};
					let created = DateTime::<Utc>::from_timestamp(
						entry.release_time.as_second(),
						entry.release_time.subsec_nanosecond() as u32,
					)
					.ok_or_else(|| ApiError::Internal(eyre::eyre!(
						"minecraft-meta release timestamp is out of range"
					)))?;
					Ok(MinecraftTagValue {
						value: legacy_game_version_name(&name).to_owned(),
						ordering: None,
						created: legacy_game_version_date(&name, created),
						ty: Some(ty.to_owned()),
						major: Some(matches!(entry.ty, VersionType::Release)),
					})
				})
				.collect::<Result<Vec<_>, ApiError>>()?
		}
		"environment" => Environment::iter()
			.map(|value| MinecraftTagValue {
				value: value.as_str().to_owned(),
				ordering: None,
				created: DateTime::<Utc>::UNIX_EPOCH,
				ty: None,
				major: None,
			})
			.collect(),
		"mrpack_loaders" => MrpackLoader::iter()
			.map(|value| MinecraftTagValue {
				value: value.as_str().to_owned(),
				ordering: None,
				created: DateTime::<Utc>::UNIX_EPOCH,
				ty: None,
				major: None,
			})
			.collect(),
        _ => {
            return Err(ApiError::Request(eyre::eyre!(
                "`{}` is not a valid loader field",
                query.loader_field
            )));
        }
    };

    results.sort_by(|a, b| {
        b.created
            .cmp(&a.created)
            .then_with(|| a.value.cmp(&b.value))
    });
    if let Some(filters) = query.filters {
        results.retain(|value| {
            filters.iter().all(|(key, expected)| match key.as_str() {
                "type" => value
                    .ty
                    .as_deref()
                    .is_some_and(|ty| expected.as_str() == Some(ty)),
                "major" => value
                    .major
                    .is_some_and(|major| expected.as_bool() == Some(major)),
                _ => false,
            })
        });
    }

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

    Err(ApiError::Request(eyre::eyre!(
        "Invalid SPDX identifier specified",
    )))
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
    _pool: web::Data<PgPool>,
    _redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let results: Vec<LinkPlatformQueryData> = LinkPlatform::iter()
        .map(|platform| LinkPlatformQueryData {
            name: platform.to_string(),
            donation: platform.is_donation(),
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
    let results = ReportType::list(&**pool, &redis)
        .await
        .wrap_internal_err("reading HTTP response body")?;
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
    let results = ProjectType::list(&**pool, &redis)
        .await
        .wrap_internal_err("fetching project type from Redis")?;
    Ok(HttpResponse::Ok().json(results))
}
