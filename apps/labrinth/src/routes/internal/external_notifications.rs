use std::collections::HashMap;

use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::ids::{DBNotificationId, DBUserId};
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::user_item::DBUser;
use crate::database::redis::RedisPool;
use crate::models::notifications::NotificationDeliveryStatus;
use crate::models::users::Role;
use crate::models::v3::notifications::{Notification, NotificationBody};
use crate::models::v3::pats::Scopes;
use crate::queue::email::EmailQueue;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::internal::external_notifications::EmailFailure::{
    FailedToSend, MailboxNotFound, UserNotFound,
};
use crate::routes::internal::statuses::broadcast_friends_message;
use crate::sync::friends::RedisFriendsMessage;
use crate::util::guards::external_notification_key_guard;
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, delete, post};
use ariadne::ids::UserId;
use eyre::eyre;
use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create)
        .service(create_email_sync)
        .service(remove)
        .service(send_custom_email);
}

#[derive(Deserialize, PartialEq, Default, utoipa::ToSchema)]
enum EmailStrategy {
    #[default]
    Async,
    Sync,
    None,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct CreateNotification {
    #[schema(value_type = serde_json::Value)]
    pub body: NotificationBody,
    pub user_ids: Vec<UserId>,
    #[serde(default)]
    pub email: EmailStrategy,
}

#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
enum EmailFailure {
    #[error("user not found")]
    UserNotFound,
    #[error("mailbox not found")]
    MailboxNotFound,
    #[error("failed to send: {0:?}")]
    FailedToSend(NotificationDeliveryStatus),
    #[error("api error: {0}")]
    ApiError(
        #[serde(serialize_with = "serialize_api_error")]
        #[from]
        crate::routes::ApiError,
    ),
}

fn serialize_api_error<S>(
    error: &ApiError,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    error.as_api_error().serialize(serializer)
}

/// Create external notifications.
#[utoipa::path(
	tag = "external notifications",
	responses((status = ACCEPTED))
)]
#[post("/external_notifications", guard = "external_notification_key_guard")]
pub async fn create(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    email_queue: web::Data<EmailQueue>,
    create_notification: web::Json<CreateNotification>,
) -> Result<(web::Json<HashMap<UserId, EmailFailure>>, StatusCode), ApiError> {
    create_impl(pool, redis, email_queue, create_notification.into_inner())
        .await
}

/// Create notifications and send emails.
///
/// Responds with the user IDs that could not be emailed:
/// - `200` if every recipient was emailed (empty list)
/// - `207` if some recipients could not be emailed (list of failed IDs)
/// Create email sync.
#[utoipa::path(
	tag = "external notifications",
	responses(
		(status = OK, body = inline(Vec<UserId>)),
		(status = 207, body = inline(Vec<UserId>)),
	)
)]
#[post(
    "external_notifications/email-sync",
    guard = "external_notification_key_guard"
)]
pub async fn create_email_sync(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    email_queue: web::Data<EmailQueue>,
    data: web::Json<CreateNotification>,
) -> Result<(web::Json<Vec<UserId>>, StatusCode), ApiError> {
    let data = data.into_inner();
    create_impl(
        pool,
        redis,
        email_queue,
        CreateNotification {
            body: data.body,
            user_ids: data.user_ids,
            email: EmailStrategy::Sync,
        },
    )
    .await
    .map(|(res, code)| {
        (web::Json(res.into_inner().into_keys().collect()), code)
    })
}

async fn create_impl(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    email_queue: web::Data<EmailQueue>,
    data: CreateNotification,
) -> Result<(web::Json<HashMap<UserId, EmailFailure>>, StatusCode), ApiError> {
    let CreateNotification {
        body,
        user_ids,
        email: email_strategy,
    } = data;
    let raw_user_ids = user_ids.iter().map(|x| x.0 as i64).collect::<Vec<_>>();
    let user_ids = raw_user_ids
        .iter()
        .map(|x| DBUserId(*x))
        .collect::<Vec<_>>();

    let mut txn = pool.begin().await?;

    if !DBUser::exists_many(&user_ids, &mut txn).await? {
        return Err(ApiError::InvalidInput(
            "One of the specified users do not exist.".to_owned(),
        ));
    }

    // Skip users who already have an identical notification
    let body_value = serde_json::value::to_value(&body)?;
    let already_notified = sqlx::query!(
        "
        SELECT DISTINCT user_id
        FROM notifications
        WHERE user_id = ANY($1::bigint[]) AND body = $2::jsonb
        ",
        &raw_user_ids[..],
        body_value,
    )
    .fetch_all(&mut txn)
    .await?
    .into_iter()
    .map(|row| DBUserId(row.user_id))
    .collect::<std::collections::HashSet<_>>();

    let notification_user_ids = user_ids
        .clone()
        .into_iter()
        .filter(|id| !already_notified.contains(id))
        .collect::<Vec<_>>();

    let notification_builder = NotificationBuilder { body: body.clone() };

    let notification_ids = if email_strategy == EmailStrategy::Async {
        notification_builder
            .insert_many(notification_user_ids, &mut txn, &redis)
            .await?
    } else {
        notification_builder
            .insert_many_without_delivery(
                notification_user_ids,
                &mut txn,
                &redis,
            )
            .await?
    };

    let notifications =
        get_site_exposed_notifications(&notification_ids, &mut txn).await?;

    txn.commit().await?;

    broadcast_notifications(&redis, notifications).await;

    if email_strategy == EmailStrategy::Sync {
        let mut email_txn = pool.begin().await?;

        let mut failed = HashMap::new();
        let users = DBUser::get_many_ids(&user_ids, &mut email_txn, &redis)
            .await?
            .into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>();

        for db_user_id in &user_ids {
            let user_id = UserId(db_user_id.0 as u64);
            let Some(user) = users.get(db_user_id) else {
                failed.insert(user_id, UserNotFound);
                continue;
            };

            let Some(mailbox) = user
                .email
                .as_ref()
                .and_then(|email| email.parse::<Mailbox>().ok())
            else {
                failed.insert(user_id, MailboxNotFound);
                continue;
            };

            match email_queue
                .send_one(&mut email_txn, body.clone(), *db_user_id, mailbox)
                .await
            {
                Ok(status) => {
                    if status != NotificationDeliveryStatus::Delivered {
                        failed.insert(user_id, FailedToSend(status));
                    }
                }
                Err(error) => {
                    if matches!(
                        error,
                        ApiError::SqlxDatabase(_) | ApiError::Database(_)
                    ) {
                        return Err(error);
                    };
                    failed.insert(user_id, error.into());
                }
            };
        }

        email_txn.commit().await?;

        let status = if failed
            .values()
            .any(|x| matches!(x, EmailFailure::ApiError(_)))
        {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::OK
        };

        return Ok((web::Json(failed), status));
    }

    Ok((web::Json(HashMap::new()), StatusCode::ACCEPTED))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct NotificationFilter {
    pub user_ids: Vec<UserId>,
    #[serde(flatten)]
    pub body: serde_json::Map<String, serde_json::Value>,
}

/// Remove external notifications.
#[utoipa::path(
	tag = "external notifications",
	responses((status = NO_CONTENT))
)]
#[delete("/external_notifications", guard = "external_notification_key_guard")]
pub async fn remove(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    notification_filter: web::Json<NotificationFilter>,
) -> Result<HttpResponse, ApiError> {
    let NotificationFilter { user_ids, body } =
        notification_filter.into_inner();

    if user_ids.is_empty() {
        return Err(ApiError::Request(eyre!(
            "at least one user must be provided to remove notifications from"
        )));
    }

    if body.is_empty() {
        return Err(ApiError::Request(eyre!(
            "at least one `body` field must be provided to match notifications"
        )));
    }

    let filters = serde_json::Value::Object(body);

    let user_ids = user_ids
        .into_iter()
        .map(|x| DBUserId(x.0 as i64))
        .collect::<Vec<_>>();

    let mut txn = pool.begin().await?;

    DBNotification::remove_many_matching_body(
        &filters, &user_ids, &mut txn, &redis,
    )
    .await?;

    txn.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, utoipa::ToSchema)]
struct SendEmail {
    pub users: Vec<UserId>,
    pub key: String,
    pub body_md: String,
    pub title: String,
}

/// Send a custom email.
#[utoipa::path(
	tag = "external notifications",
	responses((status = ACCEPTED))
)]
#[post("/external_notifications/send_custom_email")]
pub async fn send_custom_email(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<SendEmail>,
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

    if user.role != Role::Admin {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to send custom emails!".to_string(),
        ));
    }

    let SendEmail {
        users,
        body_md,
        title,
        key,
    } = body.into_inner();

    let users = users
        .into_iter()
        .map(|x| DBUserId(x.0 as i64))
        .collect::<Vec<_>>();

    let mut txn = pool.begin().await?;

    NotificationBuilder {
        body: NotificationBody::Custom {
            title,
            body_md,
            key,
        },
    }
    .insert_many(users, &mut txn, &redis)
    .await?;

    txn.commit().await?;

    Ok(HttpResponse::Accepted().finish())
}

async fn get_site_exposed_notifications(
    notification_ids: &[DBNotificationId],
    txn: &mut crate::database::PgTransaction<'_>,
) -> Result<Vec<Notification>, ApiError> {
    let raw_ids = notification_ids.iter().map(|x| x.0).collect::<Vec<_>>();
    let exposed_ids = sqlx::query_scalar!(
        r#"
        SELECT n.id AS "id!"
        FROM notifications n
        INNER JOIN notifications_types nt ON nt.name = n.body ->> 'type'
        WHERE n.id = ANY($1::BIGINT[])
          AND nt.expose_in_site_notifications = TRUE
        "#,
        &raw_ids[..],
    )
    .fetch_all(&mut *txn)
    .await?
    .into_iter()
    .map(DBNotificationId)
    .collect::<Vec<_>>();

    Ok(DBNotification::get_many(&exposed_ids, txn)
        .await?
        .into_iter()
        .map(Notification::from)
        .collect())
}

async fn broadcast_notifications(
    redis: &RedisPool,
    notifications: Vec<Notification>,
) {
    for notification in notifications {
        let notification_id = notification.id;
        let to_user = notification.user_id;
        if let Err(error) = broadcast_friends_message(
            redis,
            RedisFriendsMessage::Notification {
                to_user,
                notification_id,
            },
        )
        .await
        {
            tracing::warn!(
                ?error,
                ?notification_id,
                ?to_user,
                "failed to broadcast realtime notification"
            );
        }
    }
}
