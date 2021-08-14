use actix_web::{get, HttpResponse};
use serde_json::json;

#[get("/")]
pub async fn index_get() -> HttpResponse {
    let data = json!({
        "name": "modrinth-labrinth",
        "version": env!("CARGO_PKG_VERSION"),
        "documentation": "https://docs.modrinth.com",
        "about": "Welcome traveler!"
    });

    HttpResponse::Ok().json(data)
}
