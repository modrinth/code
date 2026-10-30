use crate::database::PgPool;
use crate::models::analytics::DownloadReason;
use crate::models::ids::{ProjectId, VersionId};
use crate::routes::ApiError;
use crate::search::SearchBackend;
use crate::search::incremental::consume::reindex_project_document;
use crate::util::error::Context;
use crate::util::guards::admin_key_guard;
use actix_web::{HttpResponse, post, web};
use eyre::eyre;
use serde::Deserialize;
use xredis::RedisPool;

pub mod user_credentials;
pub mod user_lock;
pub mod user_sessions;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .service(force_reindex)
            .service(force_reindex_project)
            .service(
                web::scope("/user")
                    .configure(user_credentials::config)
                    .configure(user_lock::config)
                    .configure(user_sessions::config),
            ),
    );
}

/// Extra data attached to each download request, transmitted through the
/// `modrinth-download-meta` header.
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadMeta {
    pub reason: Option<DownloadReason>,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub dependent_on: Option<VersionId>,
}

/// Reindex all projects.  
#[utoipa::path(
	context_path = "/admin",
	tag = "v2 admin",
    post,
    operation_id = "forceReindex",
    responses(
        (status = 204, description = "Search index rebuilt successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
#[post("/_force_reindex", guard = "admin_key_guard")]
pub async fn force_reindex(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    search_backend: web::Data<dyn SearchBackend>,
) -> Result<HttpResponse, ApiError> {
    let redis = redis.get_ref();
    search_backend
        .rebuild_index(pool.as_ref().clone(), redis.clone())
        .await
        .wrap_internal_err("failed to index projects")?;
    Ok(HttpResponse::NoContent().finish())
}

/// Reindex a project.  
#[utoipa::path(
	context_path = "/admin",
	tag = "v2 admin",
    post,
    operation_id = "forceReindexProject",
    responses(
        (status = 204, description = "Project search documents rebuilt successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
#[post("/_force_reindex/{project_id}", guard = "admin_key_guard")]
pub async fn force_reindex_project(
    path: web::Path<(ProjectId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    search_backend: web::Data<dyn SearchBackend>,
) -> Result<HttpResponse, ApiError> {
    let (project_id,) = path.into_inner();
    reindex_project_document(
        pool.as_ref(),
        redis.as_ref(),
        search_backend.as_ref(),
        project_id,
    )
    .await
    .wrap_internal_err_with(|| {
        eyre!("failed to reindex project `{project_id}`")
    })?;

    Ok(HttpResponse::NoContent().finish())
}
