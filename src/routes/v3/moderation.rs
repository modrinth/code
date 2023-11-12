use super::ApiError;
use crate::database;
use crate::database::redis::RedisPool;
use crate::models::projects::ProjectStatus;
use crate::queue::session::AuthQueue;
use crate::{auth::check_is_moderator_from_headers, models::pats::Scopes};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("moderation/projects", web::get().to(get_projects));
}

#[derive(Deserialize)]
pub struct ResultCount {
    #[serde(default = "default_count")]
    pub count: i16,
}

fn default_count() -> i16 {
    100
}

pub async fn get_projects(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    count: web::Query<ResultCount>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ]),
    )
    .await?;

    use futures::stream::TryStreamExt;

    let project_ids = sqlx::query!(
        "
        SELECT id FROM mods
        WHERE status = $1
        ORDER BY queued ASC
        LIMIT $2;
        ",
        ProjectStatus::Processing.as_str(),
        count.count as i64
    )
    .fetch_many(&**pool)
    .try_filter_map(|e| async { Ok(e.right().map(|m| database::models::ProjectId(m.id))) })
    .try_collect::<Vec<database::models::ProjectId>>()
    .await?;

    let projects: Vec<_> = database::Project::get_many_ids(&project_ids, &**pool, &redis)
        .await?
        .into_iter()
        .map(crate::models::projects::Project::from)
        .collect();

    Ok(HttpResponse::Ok().json(projects))
}
