use crate::routes::{v3, ApiError};
use actix_web::{get, web, HttpResponse};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}

#[get("statistics")]
pub async fn get_stats(pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    v3::statistics::get_stats(pool).await
}
