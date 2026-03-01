use actix_web::{HttpRequest, post, web};
use serde::{Deserialize, Serialize};

use crate::{
    auth::get_user_from_headers,
    database::{PgPool, redis::RedisPool},
    models::pats::Scopes,
    queue::{server_ping, session::AuthQueue},
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(ping_minecraft_java);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PingRequest {
    pub address: String,
    pub port: u16,
}

#[utoipa::path]
#[post("/minecraft-java")]
pub async fn ping_minecraft_java(
    req: HttpRequest,
    web::Json(request): web::Json<PingRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let (_, _user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    server_ping::ping_server(&request.address, request.port)
        .await
        .wrap_request_err("failed to ping server")?;

    Ok(())
}
