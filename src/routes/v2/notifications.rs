use crate::database::redis::RedisPool;
use crate::models::ids::NotificationId;
use crate::queue::session::AuthQueue;
use crate::routes::v3;
use crate::routes::ApiError;
use actix_web::{delete, get, patch, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(notifications_get);
    cfg.service(notifications_delete);
    cfg.service(notifications_read);

    cfg.service(
        web::scope("notification")
            .service(notification_get)
            .service(notification_read)
            .service(notification_delete),
    );
}

#[derive(Serialize, Deserialize)]
pub struct NotificationIds {
    pub ids: String,
}

#[get("notifications")]
pub async fn notifications_get(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::notifications::notifications_get(
        req,
        web::Query(v3::notifications::NotificationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
}

#[get("{id}")]
pub async fn notification_get(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::notifications::notification_get(req, info, pool, redis, session_queue).await
}

#[patch("{id}")]
pub async fn notification_read(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::notifications::notification_read(req, info, pool, redis, session_queue).await
}

#[delete("{id}")]
pub async fn notification_delete(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::notifications::notification_delete(req, info, pool, redis, session_queue).await
}

#[patch("notifications")]
pub async fn notifications_read(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::notifications::notifications_read(
        req,
        web::Query(v3::notifications::NotificationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
}

#[delete("notifications")]
pub async fn notifications_delete(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    v3::notifications::notifications_delete(
        req,
        web::Query(v3::notifications::NotificationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
}
