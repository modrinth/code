use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::redis::RedisPool;
use crate::models::ids::NotificationId;
use crate::models::notifications::Notification;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("notifications", web::get().to(notifications_get));
    cfg.route("notifications", web::patch().to(notifications_read));
    cfg.route("notifications", web::delete().to(notifications_delete));

    cfg.service(
        web::scope("notification")
            .route("{id}", web::get().to(notification_get))
            .route("{id}", web::patch().to(notification_read))
            .route("{id}", web::delete().to(notification_delete)),
    );
}

#[derive(Serialize, Deserialize)]
pub struct NotificationIds {
    pub ids: String,
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
        Some(&[Scopes::NOTIFICATION_READ]),
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
        Some(&[Scopes::NOTIFICATION_READ]),
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
        Some(&[Scopes::NOTIFICATION_WRITE]),
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
        Some(&[Scopes::NOTIFICATION_WRITE]),
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
        Some(&[Scopes::NOTIFICATION_WRITE]),
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
        Some(&[Scopes::NOTIFICATION_WRITE]),
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
