use actix_web::{HttpResponse, get};
use serde_json::json;

#[get("/")]
pub async fn index_get() -> HttpResponse {
    let data = json!({
        "name": "modrinth-labrinth",
        "version": env!("CARGO_PKG_VERSION"),
        "documentation": "https://docs.modrinth.com",
        "about": "Welcome traveler!",

        "build_info": {
            "comp_date": env!("COMPILATION_DATE"),
            "git_hash":  env!("GIT_HASH", "unknown"),
            "profile": env!("COMPILATION_PROFILE"),
        }
    });

    HttpResponse::Ok().json(data)
}
