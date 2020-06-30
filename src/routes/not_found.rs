use crate::models::error::ApiError;
use actix_web::{HttpResponse, Responder};

pub async fn not_found() -> impl Responder {
    let data = ApiError {
        error: "not_found",
        description: "the route you called is not (yet) implemented",
    };

    HttpResponse::NotFound().json(data)
}
