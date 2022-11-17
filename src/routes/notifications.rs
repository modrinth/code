use crate::database;
use crate::models::ids::NotificationId;
use crate::models::notifications::Notification;
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use actix_web::{delete, get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct NotificationIds {
    pub ids: String,
}

#[get("notifications")]
pub async fn notifications_get(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    // TODO: this is really confusingly named.
    use database::models::notification_item::Notification as DBNotification;
    use database::models::NotificationId as DBNotificationId;

    let notification_ids: Vec<DBNotificationId> =
        serde_json::from_str::<Vec<NotificationId>>(ids.ids.as_str())?
            .into_iter()
            .map(DBNotificationId::from)
            .collect();

    let notifications_data: Vec<DBNotification> =
        database::models::notification_item::Notification::get_many(
            notification_ids,
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

#[get("{id}")]
pub async fn notification_get(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let id = info.into_inner().0;

    let notification_data =
        database::models::notification_item::Notification::get(
            id.into(),
            &**pool,
        )
        .await?;

    if let Some(data) = notification_data {
        if user.id == data.user_id.into() || user.role.is_admin() {
            Ok(HttpResponse::Ok().json(Notification::from(data)))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[delete("{id}")]
pub async fn notification_delete(
    req: HttpRequest,
    info: web::Path<(NotificationId,)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let id = info.into_inner().0;

    let notification_data =
        database::models::notification_item::Notification::get(
            id.into(),
            &**pool,
        )
        .await?;

    if let Some(data) = notification_data {
        if data.user_id == user.id.into() || user.role.is_admin() {
            let mut transaction = pool.begin().await?;

            database::models::notification_item::Notification::remove(
                id.into(),
                &mut transaction,
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
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[delete("notifications")]
pub async fn notifications_delete(
    req: HttpRequest,
    web::Query(ids): web::Query<NotificationIds>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(req.headers(), &**pool).await?;

    let notification_ids =
        serde_json::from_str::<Vec<NotificationId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect();

    let mut transaction = pool.begin().await?;

    let notifications_data =
        database::models::notification_item::Notification::get_many(
            notification_ids,
            &**pool,
        )
        .await?;

    let mut notifications: Vec<database::models::ids::NotificationId> =
        Vec::new();

    for notification in notifications_data {
        if notification.user_id == user.id.into() || user.role.is_admin() {
            notifications.push(notification.id);
        }
    }

    database::models::notification_item::Notification::remove_many(
        notifications,
        &mut transaction,
    )
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}
