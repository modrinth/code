use crate::models::mods::SearchRequest;
use crate::search::{search_for_mod, SearchError};
use actix_web::{get, web, HttpResponse};

#[get("api/v1/mods")]
pub async fn mod_search(
    web::Query(info): web::Query<SearchRequest>,
) -> Result<HttpResponse, SearchError> {
    Ok(HttpResponse::Ok().json(search_for_mod(&info)?))
}
