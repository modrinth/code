use super::ApiError;
use crate::database::PgPool;
use crate::models::projects::Project;
use crate::models::v2::projects::LegacyProject;
use crate::queue::session::AuthQueue;
use crate::routes::internal;
use crate::{database::redis::RedisPool, routes::v2_reroute};
use actix_web::{HttpRequest, HttpResponse, get, web};
use serde::Deserialize;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(utoipa_actix_web::scope("/moderation").service(get_projects));
}

#[derive(Deserialize)]
pub struct ResultCount {
    #[serde(default = "default_count")]
    pub count: u16,
}

fn default_count() -> u16 {
    100
}

/// Get projects in the moderation queue.
#[utoipa::path(
    get,
    operation_id = "getModerationProjects",
    params(
        (
            "count" = Option<u16>,
            Query,
            description = "Maximum number of projects to return"
        )
    ),
    responses(
        (status = 200, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_READ"]))
)]
#[get("/projects")]
pub async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    count: web::Query<ResultCount>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = internal::moderation::get_projects_internal(
        req,
        pool.clone(),
        redis.clone(),
        web::Query(internal::moderation::ProjectsRequestOptions {
            count: count.count,
            offset: 0,
        }),
        session_queue,
    )
    .await
    .map(|resp| HttpResponse::Ok().json(resp))
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
