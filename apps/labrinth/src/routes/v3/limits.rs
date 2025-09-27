use crate::{
    auth::get_user_from_headers,
    database::{models::user_limits::UserLimits, redis::RedisPool},
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::ApiError,
};
use actix_web::{HttpRequest, web};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("limits").route("", web::get().to(get_limits)));
}

async fn get_limits(
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

    tracing::info!("FASDVNSOVSNRV");

    let limits = UserLimits::get(&user, &pool).await?;
    Ok(web::Json(limits))
}
