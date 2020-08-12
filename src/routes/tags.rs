use super::ApiError;
use crate::database::models;
use actix_web::{delete, get, put, web, HttpResponse};
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
            .service(game_version_delete),
    );
}

// TODO: searching / filtering? Could be used to implement a live
// searching category list
#[get("category")]
pub async fn category_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results = Category::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

// At some point this may take more info, but it should be able to
// remain idempotent
// TODO: don't fail if category already exists
#[put("category/{name}")]
pub async fn category_create(
    pool: web::Data<PgPool>,
    category: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    let name = category.into_inner().0;

    let _id = Category::builder().name(&name)?.insert(&**pool).await?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("category/{name}")]
pub async fn category_delete(
    pool: web::Data<PgPool>,
    category: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
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

// At some point this may take more info, but it should be able to
// remain idempotent
// TODO: don't fail if loader already exists
#[put("loader/{name}")]
pub async fn loader_create(
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    let name = loader.into_inner().0;

    let _id = Loader::builder().name(&name)?.insert(&**pool).await?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("loader/{name}")]
pub async fn loader_delete(
    pool: web::Data<PgPool>,
    loader: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
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

#[get("game_version")]
pub async fn game_version_list(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let results = GameVersion::list(&**pool).await?;
    Ok(HttpResponse::Ok().json(results))
}

// At some point this may take more info, but it should be able to
// remain idempotent
#[put("game_version/{name}")]
pub async fn game_version_create(
    pool: web::Data<PgPool>,
    game_version: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
    let name = game_version.into_inner().0;

    let _id = GameVersion::builder()
        .version(&name)?
        .insert(&**pool)
        .await?;

    Ok(HttpResponse::Ok().body(""))
}

#[delete("game_version/{name}")]
pub async fn game_version_delete(
    pool: web::Data<PgPool>,
    game_version: web::Path<(String,)>,
) -> Result<HttpResponse, ApiError> {
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
