use super::ApiError;
use crate::auth::check_is_admin_from_headers;
use crate::database::models;
use crate::database::models::categories::{DonationPlatform, License, ReportType};
use actix_web::{delete, get, put, web, HttpRequest, HttpResponse};
use models::categories::{Category, GameVersion, Loader};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tag/")
            .service(category_list)
            .service(category_create)
            .service(category_delete)
            .service(loader_list)
            .service(loader_create)
            .service(loader_delete)
            .service(game_version_list)
            .service(game_version_create)
            .service(game_version_delete)
            .service(license_create)
            .service(license_delete)
            .service(license_list)
            .service(donation_platform_create)
            .service(donation_platform_list)
            .service(donation_platform_delete),
    );
}

// TODO: searching / filtering? Could be used to implement a live
// searching category list
#[get("category")]
pub async fn category_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results = Category::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[put("category/{name}")]
pub async fn category_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    category: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = category.into_inner().0;

    let _id = Category::builder().name(&name)?.insert(&**pool).await?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("category/{name}")]
pub async fn category_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    category: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = category.into_inner().0;
    let mut transaction = pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = Category::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("loader")]
pub async fn loader_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results = Loader::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[put("loader/{name}")]
pub async fn loader_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;

    let _id = Loader::builder().name(&name)?.insert(&**pool).await?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("loader/{name}")]
pub async fn loader_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;
    let mut transaction = pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = Loader::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(serde::Deserialize)]
pub struct GameVersionQueryData {
    #[serde(rename = "type")]
    type_: Option<String>,
    major: Option<bool>,
}

#[get("game_version")]
pub async fn game_version_list(
    pool: web::Data<PgPool>,
    query: web::Query<GameVersionQueryData>,
) -> Result<HttpResponse, ApiError> {
    if query.type_.is_some() || query.major.is_some() {
        let results =
            GameVersion::list_filter(query.type_.as_deref(), query.major, &**pool).await?;
        Ok(HttpResponse::Ok().json(results))
    } else {
        let results = GameVersion::list(&**pool).await?;
        Ok(HttpResponse::Ok().json(results))
    }
}

#[derive(serde::Deserialize)]
pub struct GameVersionData {
    #[serde(rename = "type")]
    type_: String,
    date: Option<chrono::DateTime<chrono::Utc>>,
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

    Ok(HttpResponse::Ok().body(""))
}

#[delete("game_version/{name}")]
pub async fn game_version_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    game_version: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = game_version.into_inner().0;
    let mut transaction = pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = GameVersion::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(serde::Serialize)]
pub struct LicenseQueryData {
    short: String,
    name: String,
}

#[get("license")]
pub async fn license_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results: Vec<LicenseQueryData> = License::list(&**pool)
        .await?
        .into_iter()
        .map(|x| LicenseQueryData {
            short: x.short,
            name: x.name,
        })
        .collect();
    Ok(HttpResponse::Ok().json(results))
}

#[derive(serde::Deserialize)]
pub struct LicenseData {
    name: String,
}

#[put("license/{name}")]
pub async fn license_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    license: web::Path<(String,)>,
    license_data: web::Json<LicenseData>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let short = license.into_inner().0;

    let _id = License::builder()
        .short(&short)?
        .name(&license_data.name)?
        .insert(&**pool)
        .await?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("license/{name}")]
pub async fn license_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    license: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = license.into_inner().0;
    let mut transaction = pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = License::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(serde::Serialize)]
pub struct DonationPlatformQueryData {
    short: String,
    name: String,
}

#[get("donation_platform")]
pub async fn donation_platform_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results: Vec<DonationPlatformQueryData> = DonationPlatform::list(&**pool)
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

    Ok(HttpResponse::Ok().body(""))
}

#[delete("donation_platform/{name}")]
pub async fn donation_platform_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = loader.into_inner().0;
    let mut transaction = pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = DonationPlatform::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("report_type")]
pub async fn report_type_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
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

    Ok(HttpResponse::Ok().body(""))
}

#[delete("report_type/{name}")]
pub async fn report_type_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    report_type: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let name = report_type.into_inner().0;
    let mut transaction = pool.begin().await.map_err(models::DatabaseError::from)?;

    let result = ReportType::remove(&name, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(models::DatabaseError::from)?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
