use super::ApiError;
use crate::database::models;
use crate::database::models::categories::{
    DonationPlatform, ProjectType, ReportType,
};
use crate::util::auth::check_is_admin_from_headers;
use actix_web::{delete, get, put, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use models::categories::{Category, GameVersion, Loader};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("tag")
            .service(category_list)
            .service(category_create)
            .service(category_delete)
            .service(loader_list)
            .service(loader_create)
            .service(loader_delete)
            .service(game_version_list)
            .service(game_version_create)
            .service(game_version_delete)
            .service(license_list)
            .service(license_text)
            .service(donation_platform_create)
            .service(donation_platform_list)
            .service(donation_platform_delete)
            .service(report_type_create)
            .service(report_type_delete)
            .service(report_type_list),
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

#[put("category")]
pub async fn category_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    new_category: web::Json<CategoryData>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let project_type = crate::database::models::ProjectTypeId::get_id(
        new_category.project_type.clone(),
        &**pool,
    )
    .await?
    .ok_or_else(|| {
        ApiError::InvalidInput(
            "Specified project type does not exist!".to_string(),
        )
    })?;

    let _id = Category::builder()
        .name(&new_category.name)?
        .project_type(&project_type)?
        .icon(&new_category.icon)?
        .header(&new_category.header)?
        .insert(&**pool)
        .await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("category/{name}")]
pub async fn category_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    category: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = category.into_inner().0;
    let mut transaction =
        pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = Category::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
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

#[put("loader")]
pub async fn loader_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    new_loader: web::Json<LoaderData>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let mut transaction = pool.begin().await?;

    let project_types = ProjectType::get_many_id(
        &new_loader.supported_project_types,
        &mut *transaction,
    )
    .await?;

    let _id = Loader::builder()
        .name(&new_loader.name)?
        .icon(&new_loader.icon)?
        .supported_project_types(
            &project_types.into_iter().map(|x| x.id).collect::<Vec<_>>(),
        )?
        .insert(&mut transaction)
        .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("loader/{name}")]
pub async fn loader_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;
    let mut transaction =
        pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = Loader::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
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

#[derive(serde::Deserialize)]
pub struct GameVersionData {
    #[serde(rename = "type")]
    type_: String,
    date: Option<DateTime<Utc>>,
}

#[put("game_version/{name}")]
pub async fn game_version_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    game_version: web::Path<(String,)>,
    version_data: web::Json<GameVersionData>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = game_version.into_inner().0;

    // The version type currently isn't limited, but it should be one of:
    // "release", "snapshot", "alpha", "beta", "other"

    let mut builder = GameVersion::builder()
        .version(&name)?
        .version_type(&version_data.type_)?;

    if let Some(date) = &version_data.date {
        builder = builder.created(date);
    }

    let _id = builder.insert(&**pool).await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("game_version/{name}")]
pub async fn game_version_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    game_version: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = game_version.into_inner().0;
    let mut transaction =
        pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = GameVersion::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
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
    body: String,
}

#[get("license/{id}")]
pub async fn license_text(
    params: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    let license_id = params.into_inner().0;

    if license_id == *crate::models::projects::DEFAULT_LICENSE_ID {
        return Ok(HttpResponse::Ok().json(LicenseText {
            body: "All rights reserved unless explicitly stated.".to_string(),
        }));
    }

    if let Some(license) = spdx::license_id(&license_id) {
        return Ok(HttpResponse::Ok().json(LicenseText {
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

#[derive(serde::Deserialize)]
pub struct DonationPlatformData {
    name: String,
}

#[put("donation_platform/{name}")]
pub async fn donation_platform_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    license: web::Path<(String,)>,
    license_data: web::Json<DonationPlatformData>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let short = license.into_inner().0;

    let _id = DonationPlatform::builder()
        .short(&short)?
        .name(&license_data.name)?
        .insert(&**pool)
        .await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("donation_platform/{name}")]
pub async fn donation_platform_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;
    let mut transaction =
        pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = DonationPlatform::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("report_type")]
pub async fn report_type_list(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let results = ReportType::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[put("report_type/{name}")]
pub async fn report_type_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;

    let _id = ReportType::builder().name(&name)?.insert(&**pool).await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("report_type/{name}")]
pub async fn report_type_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    report_type: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = report_type.into_inner().0;
    let mut transaction =
        pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = ReportType::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
