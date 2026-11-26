use actix_web::{HttpRequest, get, web};
use xredis::RedisPool;

use crate::{
    auth::get_user_from_headers,
    database::{
        ReadOnlyPgPool,
        models::project_flag_item::{DBProjectFlag, PersistedProjectFlag},
    },
    models::{ids::ProjectId, pats::Scopes},
    queue::{
        moderation::project_flag::{FlagContext, ProjectFlagEngine},
        session::AuthQueue,
    },
    routes::ApiError,
    util::error::Context as _,
};

/// Fetch current and retained flags for a project.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    params(("id" = ProjectId, Path)),
    responses((status = OK, body = [PersistedProjectFlag]), (status = NOT_FOUND))
)]
#[get("/project/{id}")]
pub async fn get_project_flags(
    req: HttpRequest,
    path: web::Path<ProjectId>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<PersistedProjectFlag>>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    let project_id = path.into_inner().into();

    let flags = DBProjectFlag::get_project_flags(project_id, &ro_pool)
        .await
        .internal_context("fetching project flags")?
        .not_found_context("project not found")?;
    Ok(web::Json(flags))
}

/// Fetch rule inputs without running file scans or inference.
#[utoipa::path(
    context_path = "/moderation/project-flag",
    tag = "moderation",
    security(("bearer_auth" = [])),
    params(("id" = ProjectId, Path)),
    responses((status = OK, body = FlagContext), (status = NOT_FOUND))
)]
#[get("/project/{id}/context")]
pub async fn get_project_context(
    req: HttpRequest,
    path: web::Path<ProjectId>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    engine: web::Data<ProjectFlagEngine>,
) -> Result<web::Json<FlagContext>, ApiError> {
    get_user_from_headers(
        &req,
        &***ro_pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS | Scopes::MODERATION_READ,
    )
    .await
    .auth_context("authenticating moderator")?;

    let context = engine
        .load_context(path.into_inner().into())
        .await
        .internal_context("loading project flag context")?
        .not_found_context("project not found")?;
    Ok(web::Json(context))
}
