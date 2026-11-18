use actix_web::{HttpRequest, get, web};
use xredis::RedisPool;

use crate::{
    auth::get_user_from_headers,
    database::{
        ReadOnlyPgPool,
        models::project_flag_item::{DBProjectFlag, ProjectFlag},
    },
    models::{ids::ProjectId, pats::Scopes},
    queue::session::AuthQueue,
    routes::ApiError,
    util::error::Context as _,
};

/// Fetch current and retained flags for a project.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    params(("id" = ProjectId, Path)),
    responses((status = OK, body = [ProjectFlag]), (status = NOT_FOUND))
)]
#[get("/project/{id}")]
pub async fn get_project_flags(
    req: HttpRequest,
    path: web::Path<ProjectId>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<ProjectFlag>>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .wrap_auth_err("authenticating moderator")?;

    let project_id = path.into_inner().into();

    let flags = DBProjectFlag::get_project_flags(project_id, &ro_pool)
        .await
        .wrap_internal_err("fetching project flags")?
        .wrap_not_found_err("project not found")?;
    Ok(web::Json(flags))
}
