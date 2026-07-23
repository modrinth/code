use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::PgPool;
use crate::models::ids::NotificationId;
use crate::models::notifications::Notification;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, web};
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(notifications_get_route)
        .service(notifications_read_route)
        .service(notifications_delete_route)
        .service(notification_get_route)
        .service(notification_read_route)
        .service(notification_delete_route);
}

#[derive(Serialize, Deserialize)]
pub struct NotificationIds {
    pub ids: String,
}

#[utoipa::path(
	tag = "notifications",
	params(("ids" = String, Query)),
	responses((status = OK))
)]
#[get("/notifications")]
pub async fn notifications_get_route(
    req: HttpRequest,
    ids: web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    notifications_get(req, ids, pool, redis, session_queue).await
}

pub async fn notifications_get(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_READ,
    )
    .await?
    .1;

    use database::models::DBNotificationId;
    use database::models::notification_item::DBNotification;

    let notification_ids: Vec<DBNotificationId> =
        serde_json::from_str::<Vec<NotificationId>>(ids.ids.as_str())?
            .into_iter()
            .map(DBNotificationId::from)
            .collect();

    let notifications_data: Vec<DBNotification> =
        database::models::notification_item::DBNotification::get_many(
            &notification_ids,
            &**pool,
        )
        .await?;

    let notifications: Vec<Notification> = notifications_data
        .into_iter()
        .filter(|n| n.user_id == user.id.into() || user.role.is_admin())
        .map(Notification::from)
        .collect();

    Ok(HttpResponse::Ok().json(notifications))
}

#[utoipa::path(tag = "notifications", responses((status = OK)))]
#[get("/notification/{id}")]
pub async fn notification_get_route(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    notification_get(req, info, pool, redis, session_queue).await
}

pub async fn notification_get(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_READ,
    )
    .await?
    .1;

    let id = info.into_inner().0;

    let notification_data =
        database::models::notification_item::DBNotification::get(
            id.into(),
            &**pool,
        )
        .await?;

    if let Some(data) = notification_data {
        if user.id == data.user_id.into() || user.role.is_admin() {
            Ok(HttpResponse::Ok().json(Notification::from(data)))
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(tag = "notifications", responses((status = NO_CONTENT)))]
#[patch("/notification/{id}")]
pub async fn notification_read_route(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    notification_read(req, info, pool, redis, session_queue).await
}

pub async fn notification_read(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_WRITE,
    )
    .await?
    .1;

    let id = info.into_inner().0;

    let notification_data =
        database::models::notification_item::DBNotification::get(
            id.into(),
            &**pool,
        )
        .await?;

    if let Some(data) = notification_data {
        if data.user_id == user.id.into() || user.role.is_admin() {
            let mut transaction = pool.begin().await?;

            database::models::notification_item::DBNotification::read(
                id.into(),
                &mut transaction,
                &redis,
            )
            .await?;

            transaction.commit().await?;

            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You are not authorized to read this notification!".to_string(),
            ))
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(tag = "notifications", responses((status = NO_CONTENT)))]
#[delete("/notification/{id}")]
pub async fn notification_delete_route(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    notification_delete(req, info, pool, redis, session_queue).await
}

pub async fn notification_delete(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_WRITE,
    )
    .await?
    .1;

    let id = info.into_inner().0;

    let notification_data =
        database::models::notification_item::DBNotification::get(
            id.into(),
            &**pool,
        )
        .await?;

    if let Some(data) = notification_data {
        if data.user_id == user.id.into() || user.role.is_admin() {
            let mut transaction = pool.begin().await?;

            database::models::notification_item::DBNotification::remove(
                id.into(),
                &mut transaction,
                &redis,
            )
            .await?;

            transaction.commit().await?;

            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You are not authorized to delete this notification!"
                    .to_string(),
            ))
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(
	tag = "notifications",
	params(("ids" = String, Query)),
	responses((status = NO_CONTENT))
)]
#[patch("/notifications")]
pub async fn notifications_read_route(
    req: HttpRequest,
    ids: web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    notifications_read(req, ids, pool, redis, session_queue).await
}

pub async fn notifications_read(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_WRITE,
    )
    .await?
    .1;

    let notification_ids =
        serde_json::from_str::<Vec<NotificationId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();

    let mut transaction = pool.begin().await?;

    let notifications_data =
        database::models::notification_item::DBNotification::get_many(
            &notification_ids,
            &**pool,
        )
        .await?;

    let mut notifications: Vec<database::models::ids::DBNotificationId> =
        Vec::new();

    for notification in notifications_data {
        if notification.user_id == user.id.into() || user.role.is_admin() {
            notifications.push(notification.id);
        }
    }

    database::models::notification_item::DBNotification::read_many(
        &notifications,
        &mut transaction,
        &redis,
    )
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[utoipa::path(
	tag = "notifications",
	params(("ids" = String, Query)),
	responses((status = NO_CONTENT))
)]
#[delete("/notifications")]
pub async fn notifications_delete_route(
    req: HttpRequest,
    ids: web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    notifications_delete(req, ids, pool, redis, session_queue).await
}

pub async fn notifications_delete(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::NOTIFICATION_WRITE,
    )
    .await?
    .1;

    let notification_ids =
        serde_json::from_str::<Vec<NotificationId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();

    let mut transaction = pool.begin().await?;

    let notifications_data =
        database::models::notification_item::DBNotification::get_many(
            &notification_ids,
            &**pool,
        )
        .await?;

    let mut notifications: Vec<database::models::ids::DBNotificationId> =
        Vec::new();

    for notification in notifications_data {
        if notification.user_id == user.id.into() || user.role.is_admin() {
            notifications.push(notification.id);
        }
    }

    database::models::notification_item::DBNotification::remove_many(
        &notifications,
        &mut transaction,
        &redis,
    )
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}
