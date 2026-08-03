use actix_web::{HttpRequest, get, patch, put, web};
use xredis::RedisPool;

use crate::{
    auth::get_user_from_headers,
    database::{PgPool, ReadOnlyPgPool, models::DBUserPreferences},
    models::{
        pats::Scopes,
        v3::user_preferences::{UserPreferences, UserPreferencesPatch},
    },
    queue::session::AuthQueue,
    util::error::Context,
};

use super::ApiError;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_user_preferences)
        .service(put_user_preferences)
        .service(patch_user_preferences);
}

#[utoipa::path(
	tag = "users",
	responses((status = OK, body = UserPreferences))
)]
#[get("/user/preferences")]
pub async fn get_user_preferences(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserPreferences>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_READ,
    )
    .await?
    .1;

    let preferences = DBUserPreferences::get(user.id.into(), &***ro_pool)
        .await
        .wrap_internal_err("failed to fetch user preferences")?
        .unwrap_or_default();

    Ok(web::Json(preferences))
}

#[utoipa::path(
	tag = "users",
	request_body = UserPreferences,
	responses((status = OK, body = UserPreferences))
)]
#[put("/user/preferences")]
pub async fn put_user_preferences(
    req: HttpRequest,
    web::Json(preferences): web::Json<UserPreferences>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserPreferences>, ApiError> {
    preferences.validate().map_err(ApiError::Request)?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await?
    .1;

    DBUserPreferences::upsert(user.id.into(), &preferences, &**pool)
        .await
        .wrap_internal_err("failed to store user preferences")?;

    Ok(web::Json(preferences))
}

#[utoipa::path(
	tag = "users",
	request_body = UserPreferencesPatch,
	responses((status = OK, body = UserPreferences))
)]
#[patch("/user/preferences")]
pub async fn patch_user_preferences(
    req: HttpRequest,
    web::Json(patch): web::Json<UserPreferencesPatch>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserPreferences>, ApiError> {
    patch.validate().map_err(ApiError::Request)?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await?
    .1;
    let user_id = user.id.into();

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin user preferences transaction")?;

    DBUserPreferences::insert_if_absent(
        user_id,
        &UserPreferences::default(),
        &mut transaction,
    )
    .await
    .wrap_internal_err("failed to initialize user preferences")?;

    let mut preferences =
        DBUserPreferences::get_for_update(user_id, &mut transaction)
            .await
            .wrap_internal_err("failed to fetch user preferences for update")?;
    preferences.apply_patch(patch);

    DBUserPreferences::upsert(user_id, &preferences, &mut transaction)
        .await
        .wrap_internal_err("failed to store user preferences")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit user preferences transaction")?;

    Ok(web::Json(preferences))
}
