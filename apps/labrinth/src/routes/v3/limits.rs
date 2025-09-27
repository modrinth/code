use crate::{
    auth::get_user_from_headers,
    database::redis::RedisPool,
    models::{pats::Scopes, v3::user_limits::UserLimits},
    queue::session::AuthQueue,
    routes::ApiError,
};
use actix_web::{HttpRequest, web};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("limits")
            .route("projects", web::get().to(get_project_limits))
            .route("organizations", web::get().to(get_organization_limits))
            .route("collections", web::get().to(get_collection_limits)),
    );
}

async fn get_project_limits(
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
    .await?;

    let limits = UserLimits::get_for_projects(&user, &pool).await?;
    Ok(web::Json(limits))
}

async fn get_organization_limits(
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
    .await?;

    let limits = UserLimits::get_for_organizations(&user, &pool).await?;
    Ok(web::Json(limits))
}

async fn get_collection_limits(
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
    .await?;

    let limits = UserLimits::get_for_collections(&user, &pool).await?;
    Ok(web::Json(limits))
}
