use std::time::Duration;

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

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(ping_minecraft_java);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PingRequest {
    pub address: String,
    pub timeout_ms: Option<u64>,
}

/// Ping Minecraft server.  
#[utoipa::path(
	tag = "server ping",
	responses((status = NO_CONTENT))
)]
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

    let timeout = request.timeout_ms.map(Duration::from_millis);
    server_ping::ping_server(&request.address, timeout)
        .await
        .wrap_request_err("failed to ping server")?;

    Ok(())
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(ping_minecraft_java,))]
#[allow(dead_code)]
pub(crate) struct RouteDoc;
