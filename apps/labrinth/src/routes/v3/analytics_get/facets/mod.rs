mod fixed;

use actix_web::{HttpRequest, post, web};
use serde::Serialize;

use super::DownloadSource;
use crate::models::{
    ids::VersionId, pats::Scopes, v3::analytics::DownloadReason,
};
use crate::{
    auth::get_user_from_headers,
    database::{PgPool, redis::RedisPool},
    queue::session::AuthQueue,
    routes::ApiError,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(fetch_facets);
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FacetsResponse {
    pub facets: AnalyticsFacets,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct AnalyticsFacets {
    pub project_views: ProjectViewsFacets,
    pub project_downloads: ProjectDownloadsFacets,
    pub project_playtime: ProjectPlaytimeFacets,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectViewsFacets {
    pub domain: Vec<String>,
    pub site_path: Vec<String>,
    pub monetized: Vec<bool>,
    pub country: Vec<String>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectDownloadsFacets {
    pub domain: Vec<String>,
    pub user_agent: Vec<DownloadSource>,
    pub version_id: Vec<VersionId>,
    pub monetized: Vec<bool>,
    pub country: Vec<String>,
    pub reason: Vec<DownloadReason>,
    pub game_version: Vec<String>,
    pub loader: Vec<String>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectPlaytimeFacets {
    pub version_id: Vec<VersionId>,
    pub loader: Vec<String>,
    pub game_version: Vec<String>,
    pub country: Vec<String>,
}

/// Get analytics facets.  
#[utoipa::path(
	tag = "analytics",
	responses((status = OK, body = inline(FacetsResponse))),
)]
#[post("/facets")]
pub async fn fetch_facets(
    http_req: HttpRequest,
    _req: web::Json<super::GetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<FacetsResponse>, ApiError> {
    get_user_from_headers(
        &http_req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?;

    let facets = fixed::fetch(&pool, &redis).await?;

    Ok(web::Json(FacetsResponse { facets }))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(fetch_facets,))]
#[allow(dead_code)]
pub(crate) struct RouteDoc;
