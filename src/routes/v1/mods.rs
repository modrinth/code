use crate::models::projects::SearchRequest;
use crate::routes::projects::ProjectIds;
use crate::routes::ApiError;
use crate::search::{search_for_project, SearchConfig, SearchError};
use crate::util::auth::{get_user_from_headers, is_authorized};
use crate::{database, models};
use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultSearchMod {
    pub mod_id: String,
    pub slug: Option<String>,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i32,
    pub follows: i32,
    pub page_url: String,
    pub icon_url: String,
    pub author_url: String,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    pub hits: Vec<ResultSearchMod>,
    pub offset: usize,
    pub limit: usize,
    pub total_hits: usize,
}

#[get("mod")]
pub async fn mod_search(
    web::Query(info): web::Query<SearchRequest>,
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_project(&info, &config).await?;
    Ok(HttpResponse::Ok().json(SearchResults {
        hits: results
            .hits
            .into_iter()
            .map(|x| ResultSearchMod {
                mod_id: format!("local-{}", x.project_id),
                slug: x.slug,
                author: x.author.clone(),
                title: format!("[STOP USING API v1] {}", x.title),
                description: format!("[STOP USING API v1] {}", x.description),
                categories: x.categories,
                versions: x.versions,
                downloads: x.downloads,
                follows: x.follows,
                page_url: format!("https://modrinth.com/mod/{}", x.project_id),
                icon_url: x.icon_url,
                author_url: format!("https://modrinth.com/user/{}", x.author),
                date_created: x.date_created,
                date_modified: x.date_modified,
                latest_version: x.latest_version,
                license: x.license,
                client_side: x.client_side,
                server_side: x.server_side,
                host: "modrinth".to_string(),
            })
            .collect(),
        offset: results.offset,
        limit: results.limit,
        total_hits: results.total_hits,
    }))
}

#[get("{id}")]
pub async fn mod_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let project_data =
        database::models::Project::get_full_from_slug_or_project_id(
            &string, &**pool,
        )
        .await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    if let Some(mut data) = project_data {
        if is_authorized(&data.inner, &user_option, &pool).await? {
            data.inner.title =
                format!("[STOP USING API v1] {}", data.inner.title);
            data.inner.description =
                format!("[STOP USING API v1] {}", data.inner.description);
            data.inner.body =
                format!("# STOP USING API v1 - whatever application you're using right now is likely deprecated or abandoned\n{}", data.inner.body);
            return Ok(
                HttpResponse::Ok().json(models::projects::Project::from(data))
            );
        }
    }
    Ok(HttpResponse::NotFound().body(""))
}

#[get("mods")]
pub async fn mods_get(
    req: HttpRequest,
    ids: web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let project_ids: Vec<database::models::ids::ProjectId> =
        serde_json::from_str::<Vec<models::ids::ProjectId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect();

    let projects_data =
        database::models::Project::get_many_full(&project_ids, &**pool).await?;

    let user_option = get_user_from_headers(req.headers(), &**pool).await.ok();

    let mut projects = Vec::with_capacity(projects_data.len());

    // can't use `map` and `collect` here since `is_authorized` must be async
    for mut proj in projects_data {
        if is_authorized(&proj.inner, &user_option, &pool).await? {
            proj.inner.title =
                format!("[STOP USING API v1] {}", proj.inner.title);
            proj.inner.description =
                format!("[STOP USING API v1] {}", proj.inner.description);
            proj.inner.body =
                format!("# STOP USING API v1 - whatever application you're using right now is likely deprecated or abandoned\n{}", proj.inner.body);
            projects.push(crate::models::projects::Project::from(proj))
        }
    }
    Ok(HttpResponse::Ok().json(projects))
}
