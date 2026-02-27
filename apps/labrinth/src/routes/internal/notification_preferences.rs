use crate::auth::get_user_from_headers;
use crate::database::models::notifications_type_item::NotificationTypeItem;
use crate::database::models::users_notifications_preferences_item::UserNotificationPreference;
use crate::database::redis::RedisPool;
use crate::database::PgPool;
use crate::models::pats::Scopes;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::web::{self, Data};
use actix_web::{HttpRequest, HttpResponse, get, patch};
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_email_preferences);
    cfg.service(set_email_preferences);
}

#[derive(Serialize)]
pub struct NotificationPreferenceEntry {
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct EmailPreferencesResponse {
    pub notification_types: Vec<NotificationType>,
    pub preferences: Vec<NotificationPreferenceEntry>,
}

#[get("email_preferences")]
pub async fn get_email_preferences(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let notification_types =
        NotificationTypeItem::list(&**pool, &redis).await?;

    let exposed_types: Vec<NotificationType> = notification_types
        .iter()
        .filter(|t| t.expose_in_user_preferences)
        .map(|t| t.name)
        .collect();

    let user_prefs = UserNotificationPreference::get_user_or_default(
        user.id.into(),
        &**pool,
    )
    .await?;

    let preferences: Vec<NotificationPreferenceEntry> = user_prefs
        .into_iter()
        .filter(|p| {
            exposed_types.contains(&p.notification_type)
        })
        .map(|p| NotificationPreferenceEntry {
            notification_type: p.notification_type,
            channel: p.channel,
            enabled: p.enabled,
        })
        .collect();

    Ok(HttpResponse::Ok().json(EmailPreferencesResponse {
        notification_types: exposed_types,
        preferences,
    }))
}

#[derive(Deserialize)]
pub struct UpdatePreferenceEntry {
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateEmailPreferencesRequest {
    pub preferences: Vec<UpdatePreferenceEntry>,
}

#[patch("email_preferences")]
pub async fn set_email_preferences(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
    body: web::Json<UpdateEmailPreferencesRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let notification_types =
        NotificationTypeItem::list(&**pool, &redis).await?;

    let exposed_types: Vec<NotificationType> = notification_types
        .iter()
        .filter(|t| t.expose_in_user_preferences)
        .map(|t| t.name)
        .collect();

    let mut transaction = pool.begin().await?;

    for pref in &body.preferences {
        if !exposed_types.contains(&pref.notification_type) {
            return Err(ApiError::InvalidInput(format!(
                "Notification type '{}' is not configurable",
                pref.notification_type.as_str()
            )));
        }

        UserNotificationPreference::upsert(
            user.id.into(),
            pref.channel,
            pref.notification_type,
            pref.enabled,
            &mut transaction,
        )
        .await?;
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

