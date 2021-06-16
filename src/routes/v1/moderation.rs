use crate::database;
use crate::models::projects::{Project, ProjectStatus};
use crate::routes::moderation::ResultCount;
use crate::routes::ApiError;
use crate::util::auth::check_is_moderator_from_headers;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};
use sqlx::PgPool;

#[get("mods")]
pub async fn get_mods(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    count: web::Query<ResultCount>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(req.headers(), &**pool).await?;

    use futures::stream::TryStreamExt;

    let project_ids = sqlx::query!(
        "
        SELECT id FROM mods
        WHERE status = (
            SELECT id FROM statuses WHERE status = $1
        )
        ORDER BY updated ASC
        LIMIT $2;
        ",
        ProjectStatus::Processing.as_str(),
        count.count as i64
    )
    .fetch_many(&**pool)
    .try_filter_map(|e| async { Ok(e.right().map(|m| database::models::ProjectId(m.id))) })
    .try_collect::<Vec<database::models::ProjectId>>()
    .await?;

    let projects: Vec<Project> = database::Project::get_many_full(project_ids, &**pool)
        .await?
        .into_iter()
        .map(crate::routes::projects::convert_project)
        .collect();

    Ok(HttpResponse::Ok().json(projects))
}
