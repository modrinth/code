use crate::{
    auth::get_user_from_headers,
    database::models::{User, user_limits::UserLimits},
    models::pats::Scopes,
    routes::ApiError,
};
use actix_web::{HttpRequest, HttpResponse, web};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_limits);
}

#[get("limits")]
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

    let limits = UserLimits::get(&user, &pool).await?;
    Ok(web::Json(limits))
}
