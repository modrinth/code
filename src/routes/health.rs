use crate::health::status::test_database;
use crate::health::SEARCH_READY;
use actix_web::web::Data;
use actix_web::{get, HttpResponse};
use serde_json::json;
use sqlx::PgPool;
use std::sync::atomic::Ordering;

#[get("/health")]
pub async fn health_get(client: Data<PgPool>) -> HttpResponse {
    // Check database connection:
    let result = test_database(client).await;
    if result.is_err() {
        let data = json!({
            "ready": false,
            "reason": "Database connection error"
        });
        return HttpResponse::InternalServerError().json(data);
    }
    if !SEARCH_READY.load(Ordering::Acquire) {
        let data = json!({
            "ready": false,
            "reason": "Indexing is not finished"
        });
        return HttpResponse::InternalServerError().json(data);
    }
    HttpResponse::Ok().json(json!({
        "ready": true,
        "reason": "Everything is OK"
    }))
}
