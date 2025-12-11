use crate::models::error::ApiError;
use actix_web::{HttpResponse, Responder};

pub async fn not_found() -> impl Responder {
    let data = ApiError {
        error: "not_found",
        description: "the requested route does not exist".to_string(),
        details: None,
    };

    HttpResponse::NotFound().json(data)
}
