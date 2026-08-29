use actix_web::{HttpRequest, web};
use xredis::RedisPool;

use crate::database::PgPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(validate)
}

#[derive(Debug)]
pub enum ValidateError {}

pub async fn validate(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<(), ApiError> {
}
