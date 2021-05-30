use crate::auth::get_user_from_headers;
use crate::database;
use crate::models::ids::NotificationId;
use crate::models::notifications::{Notification, NotificationAction};
use crate::routes::ApiError;
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

    let notification_ids = serde_json::from_str::<Vec<NotificationId>>(&*ids.ids)?
        .into_iter()
        .map(|x| x.into())
        .collect();

    let notifications_data =
        database::models::notification_item::Notification::get_many(notification_ids, &**pool)
            .await?;

    let mut notifications: Vec<Notification> = Vec::new();

    for notification in notifications_data {
        if notification.user_id == user.id.into() || user.role.is_mod() {
            notifications.push(convert_notification(notification));
        }
    }

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
        database::models::notification_item::Notification::get(id.into(), &**pool).await?;

    if let Some(data) = notification_data {
        if user.id == data.user_id.into() || user.role.is_mod() {
            Ok(HttpResponse::Ok().json(convert_notification(data)))
        } else {
            Ok(HttpResponse::NotFound().body(""))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

pub fn convert_notification(
    notif: database::models::notification_item::Notification,
) -> Notification {
    Notification {
        id: notif.id.into(),
        user_id: notif.user_id.into(),
        title: notif.title,
        text: notif.text,
        link: notif.link,
        read: notif.read,
        created: notif.created,
        actions: notif
            .actions
            .into_iter()
            .map(|x| NotificationAction {
                title: x.title,
                action_route: (x.action_route_method, x.action_route),
            })
            .collect(),
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
        database::models::notification_item::Notification::get(id.into(), &**pool).await?;

    if let Some(data) = notification_data {
        if data.user_id == user.id.into() || user.role.is_mod() {
            database::models::notification_item::Notification::remove(id.into(), &**pool).await?;

            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthenticationError(
                "You are not authorized to delete this notification!".to_string(),
            ))
        }
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}
