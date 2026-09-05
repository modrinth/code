use crate::database::PgPool;
use crate::util::error::Context as _;
use crate::{
    auth::get_user_from_headers,
    models::{pats::Scopes, v3::user_limits::UserLimits},
    queue::session::AuthQueue,
    routes::ApiError,
};
use actix_web::{HttpRequest, get, web};
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_project_limits)
        .service(get_organization_limits)
        .service(get_collection_limits);
}

#[utoipa::path(tag = "limits", responses((status = OK)))]
#[get("/limits/projects")]
pub async fn get_project_limits(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserLimits>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let limits = UserLimits::get_for_projects(&user, &pool)
        .await
        .wrap_internal_err("fetching user limits from Redis")?;
    Ok(web::Json(limits))
}

#[utoipa::path(tag = "limits", responses((status = OK)))]
#[get("/limits/organizations")]
pub async fn get_organization_limits(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserLimits>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let limits = UserLimits::get_for_organizations(&user, &pool)
        .await
        .wrap_internal_err("fetching user limits from Redis")?;
    Ok(web::Json(limits))
}

#[utoipa::path(tag = "limits", responses((status = OK)))]
#[get("/limits/collections")]
pub async fn get_collection_limits(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<UserLimits>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let limits = UserLimits::get_for_collections(&user, &pool)
        .await
        .wrap_internal_err("fetching user limits from Redis")?;
    Ok(web::Json(limits))
}
