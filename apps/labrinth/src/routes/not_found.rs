use crate::routes::error::ApiError;
use actix_web::{HttpResponse, ResponseError};

pub async fn not_found() -> HttpResponse {
    ApiError::RouteNotFound.error_response()
}
