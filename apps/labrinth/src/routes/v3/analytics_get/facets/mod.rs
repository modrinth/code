mod dynamic;
mod fixed;

use actix_web::{HttpRequest, post, web};
use serde::{Deserialize, Serialize};

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

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
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
    pub domain: Vec<FacetValue<String>>,
    pub site_path: Vec<FacetValue<String>>,
    pub monetized: Vec<FacetValue<bool>>,
    pub country: Vec<FacetValue<String>>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectDownloadsFacets {
    pub domain: Vec<FacetValue<String>>,
    pub user_agent: Vec<FacetValue<DownloadSource>>,
    pub version_id: Vec<FacetValue<VersionId>>,
    pub monetized: Vec<FacetValue<bool>>,
    pub country: Vec<FacetValue<String>>,
    pub reason: Vec<FacetValue<DownloadReason>>,
    pub game_version: Vec<FacetValue<String>>,
    pub loader: Vec<FacetValue<String>>,
}

#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct ProjectPlaytimeFacets {
    pub version_id: Vec<FacetValue<VersionId>>,
    pub loader: Vec<FacetValue<String>>,
    pub game_version: Vec<FacetValue<String>>,
    pub country: Vec<FacetValue<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, utoipa::ToSchema)]
pub struct FacetValue<T> {
    pub value: T,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
struct FacetsQuery {
    #[serde(default)]
    detailed: bool,
}

#[utoipa::path(
	responses((status = OK, body = inline(FacetsResponse))),
)]
#[post("/facets")]
pub async fn fetch_facets(
    http_req: HttpRequest,
    query: web::Query<FacetsQuery>,
    req: web::Json<super::GetRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    clickhouse: web::Data<clickhouse::Client>,
) -> Result<web::Json<FacetsResponse>, ApiError> {
    let (_, user) = get_user_from_headers(
        &http_req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?;

    let facets = if query.detailed {
        dynamic::fetch(req.into_inner(), &user, &pool, &redis, &clickhouse)
            .await?
    } else {
        fixed::fetch(&req, &user, &pool, &redis).await?
    };

    Ok(web::Json(FacetsResponse { facets }))
}
