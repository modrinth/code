use crate::database::redis::RedisPool;
use crate::models::ids::NotificationId;
use crate::models::notifications::Notification;
use crate::models::v2::notifications::LegacyNotification;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::v2_reroute;
use crate::routes::v3;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, web};
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
    let resp = v3::notifications::notifications_get(
        req,
        web::Query(v3::notifications::NotificationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error);
    match v2_reroute::extract_ok_json::<Vec<Notification>>(resp?).await {
        Ok(notifications) => {
            let notifications: Vec<LegacyNotification> = notifications
                .into_iter()
                .map(LegacyNotification::from)
                .collect();
            Ok(HttpResponse::Ok().json(notifications))
        }
        Err(response) => Ok(response),
    }
}

#[get("{id}")]
pub async fn notification_get(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let response = v3::notifications::notification_get(
        req,
        info,
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;
    match v2_reroute::extract_ok_json::<Notification>(response).await {
        Ok(notification) => {
            let notification = LegacyNotification::from(notification);
            Ok(HttpResponse::Ok().json(notification))
        }
        Err(response) => Ok(response),
    }
}

#[patch("{id}")]
pub async fn notification_read(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::notifications::notification_read(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[delete("{id}")]
pub async fn notification_delete(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::notifications::notification_delete(
        req,
        info,
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[patch("notifications")]
pub async fn notifications_read(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::notifications::notifications_read(
        req,
        web::Query(v3::notifications::NotificationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[delete("notifications")]
pub async fn notifications_delete(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::notifications::notifications_delete(
        req,
        web::Query(v3::notifications::NotificationIds { ids: ids.ids }),
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}
