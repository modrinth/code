use actix_web::{get, HttpResponse};
use serde_json::json;

#[get("/")]
pub async fn index_get() -> HttpResponse {
    let data = json!({
        "name": "modrinth-labrinth",
        "version": env!("CARGO_PKG_VERSION"),
        //TODO: Add the documentation link
        "documentation": "Nowhere yet",
        "about": "Welcome traveler !"
    });

    HttpResponse::Ok().json(data)
}
