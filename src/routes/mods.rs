use crate::models::mods::SearchRequest;
use crate::search::{search_for_mod, SearchError};
use actix_web::{get, web, HttpResponse};

#[get("api/v1/mod")]
pub async fn mod_search(
    web::Query(info): web::Query<SearchRequest>,
) -> Result<HttpResponse, SearchError> {
    let results = search_for_mod(&info).await?;
    Ok(HttpResponse::Ok().json(results))
}
