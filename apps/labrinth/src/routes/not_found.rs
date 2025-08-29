use actix_web::{HttpRequest, HttpResponse};
use crate::routes::ApiError;

pub async fn not_found(req: HttpRequest) -> HttpResponse {
    ApiError::NotFound.localized_error_response(&req)
}
