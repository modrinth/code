use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::projects::Project;
use crate::models::v2::projects::LegacyProject;
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::routes::{v2_reroute, v3, ApiError};
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(organizations_get).service(organization_create);
    cfg.service(
        web::scope("organization")
            .service(organization_get)
            .service(organizations_edit)
            .service(organization_delete)
            .service(organization_projects_get)
            .service(organization_projects_add)
            .service(organization_projects_remove)
            .service(organization_icon_edit)
            .service(delete_organization_icon)
            .service(super::teams::team_members_get_organization),
    );
}

#[derive(Deserialize, Validate)]
pub struct NewOrganization {
    #[validate(
        length(min = 3, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    // Title of the organization, also used as slug
    pub title: String,
    #[validate(length(min = 3, max = 256))]
    pub description: String,
}

#[post("organization")]
pub async fn organization_create(
    req: HttpRequest,
    new_organization: web::Json<NewOrganization>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let new_organization = new_organization.into_inner();
    v3::organizations::organization_create(
        req,
        web::Json(v3::organizations::NewOrganization {
            name: new_organization.title,
            description: new_organization.description,
        }),
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
}

#[get("{id}")]
pub async fn organization_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::organizations::organization_get(req, info, pool.clone(), redis.clone(), session_queue).await
}

#[derive(Deserialize)]
pub struct OrganizationIds {
    pub ids: String,
}
#[get("organizations")]
pub async fn organizations_get(
    req: HttpRequest,
    web::Query(ids): web::Query<OrganizationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::organizations::organizations_get(
        req,
        web::Query(v3::organizations::OrganizationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
}

#[derive(Serialize, Deserialize, Validate)]
pub struct OrganizationEdit {
    #[validate(length(min = 3, max = 256))]
    pub description: Option<String>,
    #[validate(
        length(min = 3, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    // Title of the organization, also used as slug
    pub title: Option<String>,
}

#[patch("{id}")]
pub async fn organizations_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    new_organization: web::Json<OrganizationEdit>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_organization = new_organization.into_inner();
    v3::organizations::organizations_edit(
        req,
        info,
        web::Json(v3::organizations::OrganizationEdit {
            description: new_organization.description,
            name: new_organization.title,
        }),
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
}

#[delete("{id}")]
pub async fn organization_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::organizations::organization_delete(req, info, pool.clone(), redis.clone(), session_queue)
        .await
}

#[get("{id}/projects")]
pub async fn organization_projects_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::organizations::organization_projects_get(
        req,
        info,
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await?;

    // Convert v3 projects to v2
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects = LegacyProject::from_many(project, &**pool, &redis).await?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Deserialize)]
pub struct OrganizationProjectAdd {
    pub project_id: String, // Also allow title/slug
}
#[post("{id}/projects")]
pub async fn organization_projects_add(
    req: HttpRequest,
    info: web::Path<(String,)>,
    project_info: web::Json<OrganizationProjectAdd>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let project_info = project_info.into_inner();
    v3::organizations::organization_projects_add(
        req,
        info,
        web::Json(v3::organizations::OrganizationProjectAdd {
            project_id: project_info.project_id,
        }),
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
}

#[delete("{organization_id}/projects/{project_id}")]
pub async fn organization_projects_remove(
    req: HttpRequest,
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::organizations::organization_projects_remove(
        req,
        info,
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn organization_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::organizations::organization_icon_edit(
        web::Query(v3::organizations::Extension { ext: ext.ext }),
        req,
        info,
        pool.clone(),
        redis.clone(),
        file_host,
        payload,
        session_queue,
    )
    .await
}

#[delete("{id}/icon")]
pub async fn delete_organization_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::organizations::delete_organization_icon(
        req,
        info,
        pool.clone(),
        redis.clone(),
        file_host,
        session_queue,
    )
    .await
}
