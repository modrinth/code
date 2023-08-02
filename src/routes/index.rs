use actix_web::HttpResponse;
use serde_json::json;

pub async fn index_get() -> HttpResponse {
    let data = json!({
        "name": "modrinth-labrinth",
        "version": env!("CARGO_PKG_VERSION"),
        "documentation": "https://docs.modrinth.com",
        "about": "Welcome traveler!"
    });

    HttpResponse::Ok().json(data)
}
