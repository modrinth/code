use super::ApiError;
use crate::database::models;
use crate::database::models::categories::{
    DonationPlatform, ProjectType, ReportType, SideType,
};
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use models::categories::{Category, GameVersion, Loader};
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
    icon: String,
    name: String,
    project_type: String,
    header: String,
}

// TODO: searching / filtering? Could be used to implement a live
// searching category list
#[get("category")]
pub async fn category_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = Category::list(&**pool)
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
    icon: String,
    name: String,
    supported_project_types: Vec<String>,
}

#[get("loader")]
pub async fn loader_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let mut results = Loader::list(&**pool)
        .await?
        .into_iter()
        .map(|x| LoaderData {
            icon: x.icon,
            name: x.loader,
            supported_project_types: x.supported_project_types,
        })
        .collect::<Vec<_>>();

    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Serialize)]
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
) -> Result<HttpResponse, ApiError> {
    let results: Vec<GameVersionQueryData> = if query.type_.is_some()
        || query.major.is_some()
    {
        GameVersion::list_filter(query.type_.as_deref(), query.major, &**pool)
            .await?
    } else {
        GameVersion::list(&**pool).await?
    }
    .into_iter()
    .map(|x| GameVersionQueryData {
        version: x.version,
        version_type: x.type_,
        date: x.created,
        major: x.major,
    })
    .collect();

    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Serialize)]
pub struct License {
    short: String,
    name: String,
}

#[get("license")]
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

#[derive(serde::Serialize)]
pub struct LicenseText {
    title: String,
    body: String,
}

#[get("license/{id}")]
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

#[derive(serde::Serialize)]
pub struct DonationPlatformQueryData {
    short: String,
    name: String,
}

#[get("donation_platform")]
pub async fn donation_platform_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results: Vec<DonationPlatformQueryData> =
        DonationPlatform::list(&**pool)
            .await?
            .into_iter()
            .map(|x| DonationPlatformQueryData {
                short: x.short,
                name: x.name,
            })
            .collect();
    Ok(HttpResponse::Ok().json(results))
}

#[get("report_type")]
pub async fn report_type_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = ReportType::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[get("project_type")]
pub async fn project_type_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = ProjectType::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[get("side_type")]
pub async fn side_type_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = SideType::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}
