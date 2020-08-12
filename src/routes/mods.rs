use super::ApiError;
use crate::database;
use crate::models;
use crate::models::mods::SearchRequest;
use crate::search::{search_for_mod, SearchError};
use actix_web::{delete, get, web, HttpResponse};
use sqlx::PgPool;

#[get("mod")]
pub async fn mod_search(
    web::Query(info): web::Query<SearchRequest>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_mod(&info).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[get("{id}")]
pub async fn mod_get(
    info: web::Path<(models::ids::ModId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.0;
    let mod_data = database::models::Mod::get_full(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if let Some(data) = mod_data {
        let m = data.inner;
        let response = models::mods::Mod {
            id: m.id.into(),
            team: m.team_id.into(),
            title: m.title,
            description: m.description,
            body_url: m.body_url,
            published: m.published,

            downloads: m.downloads as u32,
            categories: data.categories,
            versions: data.versions.into_iter().map(|v| v.into()).collect(),
            icon_url: m.icon_url,
            issues_url: m.issues_url,
            source_url: m.source_url,
            wiki_url: m.wiki_url,
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

// TODO: This really needs auth
// TODO: The mod remains in meilisearch's index until the index is deleted
#[delete("{id}")]
pub async fn mod_delete(
    info: web::Path<(models::ids::ModId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.0;
    let result = database::models::Mod::remove_full(id.into(), &**pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.into()))?;

    if result.is_some() {
        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
