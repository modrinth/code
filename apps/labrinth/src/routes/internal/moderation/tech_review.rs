use actix_web::{HttpRequest, get, web};
use sqlx::PgPool;

use crate::{
    auth::check_is_moderator_from_headers,
    database::redis::RedisPool,
    models::pats::Scopes,
    queue::session::AuthQueue,
    routes::{ApiError, internal::moderation::ProjectsRequestOptions},
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(get_projects);
}

/// Gets all projects which are awaiting technical review.
#[utoipa::path]
#[get("")]
async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    request_opts: web::Query<ProjectsRequestOptions>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await?;

    sqlx::query!(
        "
        SELECT id FROM delphi_reports
        ORDER BY created

        "
    )
    .fetch(&**pool)
}
