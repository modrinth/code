use super::ApiError;
use crate::queue::session::AuthQueue;
use crate::routes::v3;
use crate::{database::redis::RedisPool, routes::v2_reroute};
use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("moderation").service(get_projects));
}

#[derive(Deserialize)]
pub struct ResultCount {
    #[serde(default = "default_count")]
    pub count: i16,
}

fn default_count() -> i16 {
    100
}

#[get("projects")]
pub async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    count: web::Query<ResultCount>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::moderation::get_projects(
        req,
        pool,
        redis,
        web::Query(v3::moderation::ResultCount { count: count.count }),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}
