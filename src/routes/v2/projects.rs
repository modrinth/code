use crate::database::models::{project_item, version_item};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::projects::{
    DonationLink, MonetizationStatus, Project, ProjectStatus, SearchRequest, SideType,
};
use crate::models::v2::projects::LegacyProject;
use crate::models::v2::search::LegacySearchResults;
use crate::queue::session::AuthQueue;
use crate::routes::v3::projects::ProjectIds;
use crate::routes::{v2_reroute, v3, ApiError};
use crate::search::{search_for_project, SearchConfig, SearchError};
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(project_search);
    cfg.service(projects_get);
    cfg.service(projects_edit);
    cfg.service(random_projects_get);

    cfg.service(
        web::scope("project")
            .service(project_get)
            .service(project_get_check)
            .service(project_delete)
            .service(project_edit)
            .service(project_icon_edit)
            .service(delete_project_icon)
            .service(add_gallery_item)
            .service(edit_gallery_item)
            .service(delete_gallery_item)
            .service(project_follow)
            .service(project_unfollow)
            .service(project_schedule)
            .service(super::teams::team_members_get_project)
            .service(
                web::scope("{project_id}")
                    .service(super::versions::version_list)
                    .service(super::versions::version_project_get)
                    .service(dependency_list),
            ),
    );
}

#[get("search")]
pub async fn project_search(
    web::Query(info): web::Query<SearchRequest>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, SearchError> {
    // TODO: make this nicer
    // Search now uses loader_fields instead of explicit 'client_side' and 'server_side' fields
    // While the backend for this has changed, it doesnt affect much
    // in the API calls except that 'versions:x' is now 'game_versions:x'
    let facets: Option<Vec<Vec<String>>> = if let Some(facets) = info.facets {
        let facets = serde_json::from_str::<Vec<Vec<&str>>>(&facets)?;
        Some(
            facets
                .into_iter()
                .map(|facet| {
                    facet
                        .into_iter()
                        .map(|facet| {
                            let val = match facet.split(':').nth(1) {
                                Some(val) => val,
                                None => return facet.to_string(),
                            };

                            if facet.starts_with("versions:") {
                                format!("game_versions:{}", val)
                            } else if facet.starts_with("project_type:") {
                                format!("project_types:{}", val)
                            } else {
                                facet.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
        )
    } else {
        None
    };

    let info = SearchRequest {
        facets: facets.and_then(|x| serde_json::to_string(&x).ok()),
        ..info
    };

    let results = search_for_project(&info, &config).await?;

    let results = LegacySearchResults::from(results);

    Ok(HttpResponse::Ok().json(results))
}

#[derive(Deserialize, Validate)]
pub struct RandomProjects {
    #[validate(range(min = 1, max = 100))]
    pub count: u32,
}

#[get("projects_random")]
pub async fn random_projects_get(
    web::Query(count): web::Query<RandomProjects>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let count = v3::projects::RandomProjects { count: count.count };

    let response =
        v3::projects::random_projects_get(web::Query(count), pool.clone(), redis.clone()).await?;
    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects = LegacyProject::from_many(project, &**pool, &redis).await?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

#[get("projects")]
pub async fn projects_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Call V3 project creation
    let response = v3::projects::projects_get(
        req,
        web::Query(ids),
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects = LegacyProject::from_many(project, &**pool, &redis).await?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

#[get("{id}")]
pub async fn project_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Convert V2 data to V3 data

    // Call V3 project creation
    let response =
        v3::projects::project_get(req, info, pool.clone(), redis.clone(), session_queue).await?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Project>(response).await {
        Ok(project) => {
            let version_item = match project.versions.first() {
                Some(vid) => version_item::Version::get((*vid).into(), &**pool, &redis).await?,
                None => None,
            };
            let project = LegacyProject::from(project, version_item);
            Ok(HttpResponse::Ok().json(project))
        }
        Err(response) => Ok(response),
    }
}

//checks the validity of a project id or slug
#[get("{id}/check")]
pub async fn project_get_check(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::project_get_check(info, pool, redis).await
}

#[derive(Serialize)]
struct DependencyInfo {
    pub projects: Vec<Project>,
    pub versions: Vec<models::projects::Version>,
}

#[get("dependencies")]
pub async fn dependency_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // TODO: requires V2 conversion and tests, probably
    v3::projects::dependency_list(req, info, pool, redis, session_queue).await
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EditProject {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub title: Option<String>,
    #[validate(length(min = 3, max = 256))]
    pub description: Option<String>,
    #[validate(length(max = 65536))]
    pub body: Option<String>,
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[validate(length(max = 256))]
    pub additional_categories: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub license_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub discord_url: Option<Option<String>>,
    #[validate]
    pub donation_urls: Option<Vec<DonationLink>>,
    pub license_id: Option<String>,
    pub client_side: Option<SideType>,
    pub server_side: Option<SideType>,
    #[validate(
        length(min = 3, max = 64),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub slug: Option<String>,
    pub status: Option<ProjectStatus>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub requested_status: Option<Option<ProjectStatus>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 2000))]
    pub moderation_message: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 65536))]
    pub moderation_message_body: Option<Option<String>>,
    pub monetization_status: Option<MonetizationStatus>,
}

#[patch("{id}")]
pub async fn project_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    config: web::Data<SearchConfig>,
    new_project: web::Json<EditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let v2_new_project = new_project.into_inner();
    let client_side = v2_new_project.client_side.clone();
    let server_side = v2_new_project.server_side.clone();
    let new_slug = v2_new_project.slug.clone();

    // TODO: Some kind of handling here to ensure project type is fine.
    // We expect the version uploaded to be of loader type modpack, but there might  not be a way to check here for that.
    // After all, theoretically, they could be creating a genuine 'fabric' mod, and modpack no longer carries information on whether its a mod or modpack,
    // as those are out to the versions.

    // Ideally this would, if the project 'should' be a modpack:
    // - change the loaders to mrpack only
    // - add categories to the project for the corresponding loaders

    let new_project = v3::projects::EditProject {
        title: v2_new_project.title,
        description: v2_new_project.description,
        body: v2_new_project.body,
        categories: v2_new_project.categories,
        additional_categories: v2_new_project.additional_categories,
        issues_url: v2_new_project.issues_url,
        source_url: v2_new_project.source_url,
        wiki_url: v2_new_project.wiki_url,
        license_url: v2_new_project.license_url,
        discord_url: v2_new_project.discord_url,
        donation_urls: v2_new_project.donation_urls,
        license_id: v2_new_project.license_id,
        slug: v2_new_project.slug,
        status: v2_new_project.status,
        requested_status: v2_new_project.requested_status,
        moderation_message: v2_new_project.moderation_message,
        moderation_message_body: v2_new_project.moderation_message_body,
        monetization_status: v2_new_project.monetization_status,
    };

    // This returns 204 or failure so we don't need to do anything with it
    let project_id = info.clone().0;
    let mut response = v3::projects::project_edit(
        req.clone(),
        info,
        pool.clone(),
        config,
        web::Json(new_project),
        redis.clone(),
        session_queue.clone(),
    )
    .await?;

    // If client and server side were set, we will call
    // the version setting route for each version to set the side types for each of them.
    if response.status().is_success() && (client_side.is_some() || server_side.is_some()) {
        let project_item =
            project_item::Project::get(&new_slug.unwrap_or(project_id), &**pool, &redis).await?;
        let version_ids = project_item.map(|x| x.versions).unwrap_or_default();
        let versions = version_item::Version::get_many(&version_ids, &**pool, &redis).await?;
        for version in versions {
            let mut fields = HashMap::new();
            fields.insert("client_side".to_string(), json!(client_side));
            fields.insert("server_side".to_string(), json!(server_side));
            response = v3::versions::version_edit_helper(
                req.clone(),
                (version.inner.id.into(),),
                pool.clone(),
                redis.clone(),
                v3::versions::EditVersion {
                    fields,
                    ..Default::default()
                },
                session_queue.clone(),
            )
            .await?;
        }
    }
    Ok(response)
}

#[derive(Deserialize, Validate)]
pub struct BulkEditProject {
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[validate(length(max = 3))]
    pub add_categories: Option<Vec<String>>,
    pub remove_categories: Option<Vec<String>>,

    #[validate(length(max = 256))]
    pub additional_categories: Option<Vec<String>>,
    #[validate(length(max = 3))]
    pub add_additional_categories: Option<Vec<String>>,
    pub remove_additional_categories: Option<Vec<String>>,

    #[validate]
    pub donation_urls: Option<Vec<DonationLink>>,
    #[validate]
    pub add_donation_urls: Option<Vec<DonationLink>>,
    #[validate]
    pub remove_donation_urls: Option<Vec<DonationLink>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub discord_url: Option<Option<String>>,
}

#[patch("projects")]
pub async fn projects_edit(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    bulk_edit_project: web::Json<BulkEditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let bulk_edit_project = bulk_edit_project.into_inner();
    v3::projects::projects_edit(
        req,
        web::Query(ids),
        pool.clone(),
        web::Json(v3::projects::BulkEditProject {
            categories: bulk_edit_project.categories,
            add_categories: bulk_edit_project.add_categories,
            remove_categories: bulk_edit_project.remove_categories,
            additional_categories: bulk_edit_project.additional_categories,
            add_additional_categories: bulk_edit_project.add_additional_categories,
            remove_additional_categories: bulk_edit_project.remove_additional_categories,
            donation_urls: bulk_edit_project.donation_urls,
            add_donation_urls: bulk_edit_project.add_donation_urls,
            remove_donation_urls: bulk_edit_project.remove_donation_urls,
            issues_url: bulk_edit_project.issues_url,
            source_url: bulk_edit_project.source_url,
            wiki_url: bulk_edit_project.wiki_url,
            discord_url: bulk_edit_project.discord_url,
        }),
        redis,
        session_queue,
    )
    .await
}

#[derive(Deserialize)]
pub struct SchedulingData {
    pub time: DateTime<Utc>,
    pub requested_status: ProjectStatus,
}

#[post("{id}/schedule")]
pub async fn project_schedule(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    scheduling_data: web::Json<SchedulingData>,
) -> Result<HttpResponse, ApiError> {
    let scheduling_data = scheduling_data.into_inner();
    v3::projects::project_schedule(
        req,
        info,
        pool,
        redis,
        session_queue,
        web::Json(v3::projects::SchedulingData {
            time: scheduling_data.time,
            requested_status: scheduling_data.requested_status,
        }),
    )
    .await
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn project_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::project_icon_edit(
        web::Query(v3::projects::Extension { ext: ext.ext }),
        req,
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
    )
    .await
}

#[delete("{id}/icon")]
pub async fn delete_project_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::delete_project_icon(req, info, pool, redis, file_host, session_queue).await
}

#[derive(Serialize, Deserialize, Validate)]
pub struct GalleryCreateQuery {
    pub featured: bool,
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<String>,
    pub ordering: Option<i64>,
}

#[post("{id}/gallery")]
#[allow(clippy::too_many_arguments)]
pub async fn add_gallery_item(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    web::Query(item): web::Query<GalleryCreateQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::add_gallery_item(
        web::Query(v3::projects::Extension { ext: ext.ext }),
        req,
        web::Query(v3::projects::GalleryCreateQuery {
            featured: item.featured,
            title: item.title,
            description: item.description,
            ordering: item.ordering,
        }),
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
    )
    .await
}

#[derive(Serialize, Deserialize, Validate)]
pub struct GalleryEditQuery {
    /// The url of the gallery item to edit
    pub url: String,
    pub featured: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 255))]
    pub title: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<Option<String>>,
    pub ordering: Option<i64>,
}

#[patch("{id}/gallery")]
pub async fn edit_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryEditQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::edit_gallery_item(
        req,
        web::Query(v3::projects::GalleryEditQuery {
            url: item.url,
            featured: item.featured,
            title: item.title,
            description: item.description,
            ordering: item.ordering,
        }),
        info,
        pool,
        redis,
        session_queue,
    )
    .await
}

#[derive(Serialize, Deserialize)]
pub struct GalleryDeleteQuery {
    pub url: String,
}

#[delete("{id}/gallery")]
pub async fn delete_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryDeleteQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::delete_gallery_item(
        req,
        web::Query(v3::projects::GalleryDeleteQuery { url: item.url }),
        info,
        pool,
        redis,
        file_host,
        session_queue,
    )
    .await
}

#[delete("{id}")]
pub async fn project_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    config: web::Data<SearchConfig>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::project_delete(req, info, pool, redis, config, session_queue).await
}

#[post("{id}/follow")]
pub async fn project_follow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::project_follow(req, info, pool, redis, session_queue).await
}

#[delete("{id}/follow")]
pub async fn project_unfollow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::projects::project_unfollow(req, info, pool, redis, session_queue).await
}
