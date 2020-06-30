use crate::models::mods::SearchRequest;
use crate::search::search_for_mod;
use actix_web::{get, web, HttpResponse};

#[get("api/v1/mods")]
pub fn mod_search(web::Query(info): web::Query<SearchRequest>) -> HttpResponse {
    //TODO: Fix this line with anyhow
    let body = serde_json::to_string(&search_for_mod(&info).unwrap()).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
}
