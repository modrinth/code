use crate::routes::ApiError;
use actix_web::{HttpRequest, HttpResponse};

pub async fn not_found(req: HttpRequest) -> HttpResponse {
    ApiError::NotFound.localized_error_response(&req)
}
