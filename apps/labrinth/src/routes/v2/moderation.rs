use super::ApiError;
use crate::models::projects::Project;
use crate::models::v2::projects::LegacyProject;
use crate::queue::session::AuthQueue;
use crate::routes::internal;
use crate::{database::redis::RedisPool, routes::v2_reroute};
use actix_web::{HttpRequest, HttpResponse, get, web};
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
    let response = internal::moderation::get_projects(
        req,
        pool.clone(),
        redis.clone(),
        web::Query(internal::moderation::ResultCount { count: count.count }),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // Convert to V2 projects
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects =
                LegacyProject::from_many(project, &**pool, &redis).await?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}
