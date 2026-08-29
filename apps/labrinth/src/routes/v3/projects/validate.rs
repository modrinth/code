use actix_web::{HttpRequest, get, web};
use xredis::RedisPool;

use crate::{database::PgPool, routes::ApiError};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(validate);
}

#[derive(Debug)]
pub enum ValidateError {}

/// Validate that a project is ready to be submitted for review.
#[utoipa::path(tag = "projects")]
#[get("/{id}/validate")]
pub async fn validate(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<String, ApiError> {
    Ok(subsecond::call(|| String::from("foobar world")))
}
