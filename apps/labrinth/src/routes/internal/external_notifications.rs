use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::ids::DBUserId;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::user_item::DBUser;
use crate::database::redis::RedisPool;
use crate::models::users::Role;
use crate::models::v3::notifications::{
    NotificationBody, NotificationDeliveryStatus,
};
use crate::models::v3::pats::Scopes;
use crate::queue::email::EmailQueue;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::guards::external_notification_key_guard;
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{
    CustomizeResponder, HttpRequest, HttpResponse, Responder, delete, post,
};
use ariadne::ids::UserId;
use eyre::eyre;
use lettre::message::Mailbox;
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(create_email_sync)
        .service(remove)
        .service(send_custom_email);
}

#[derive(Deserialize)]
struct CreateNotification {
    pub body: NotificationBody,
    pub user_ids: Vec<UserId>,
}

#[post("external_notifications", guard = "external_notification_key_guard")]
pub async fn create(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    create_notification: web::Json<CreateNotification>,
) -> Result<HttpResponse, ApiError> {
    let CreateNotification { body, user_ids } =
        create_notification.into_inner();
    let user_ids = user_ids
        .into_iter()
        .map(|x| DBUserId(x.0 as i64))
        .collect::<Vec<_>>();

    let mut txn = pool.begin().await?;

    if !DBUser::exists_many(&user_ids, &mut txn).await? {
        return Err(ApiError::InvalidInput(
            "One of the specified users do not exist.".to_owned(),
        ));
    }

    NotificationBuilder { body }
        .insert_many(user_ids, &mut txn, &redis)
        .await?;

    txn.commit().await?;

    Ok(HttpResponse::Accepted().finish())
}

/// Inserts notifications for all users and tries to send emails immediately.
///
/// Responds with the user IDs that could not be emailed:
/// - `200` if every recipient was emailed (empty list)
/// - `207` if some recipients could not be emailed (list of failed IDs)
#[post(
    "external_notifications/email-sync",
    guard = "external_notification_key_guard"
)]
pub async fn create_email_sync(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    email_queue: web::Data<EmailQueue>,
    create_notification: web::Json<CreateNotification>,
) -> Result<CustomizeResponder<web::Json<Vec<UserId>>>, ApiError> {
    let CreateNotification { body, user_ids } =
        create_notification.into_inner();
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

    NotificationBuilder { body: body.clone() }
        .insert_many_without_delivery(notification_user_ids, &mut txn, &redis)
        .await?;

    txn.commit().await?;

    let mut email_txn = pool.begin().await?;

    let mut failed = Vec::new();
    for user_id in &user_ids {
        let Some(user) =
            DBUser::get_id(*user_id, &mut email_txn, &redis).await?
        else {
            failed.push(UserId(user_id.0 as u64));
            continue;
        };

        let delivered = match user
            .email
            .and_then(|email| email.parse::<Mailbox>().ok())
        {
            Some(mailbox) => {
                email_queue
                    .send_one(&mut email_txn, body.clone(), *user_id, mailbox)
                    .await?
                    == NotificationDeliveryStatus::Delivered
            }
            None => false,
        };

        if !delivered {
            failed.push(UserId(user_id.0 as u64));
        }
    }

    let status = if failed.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::MULTI_STATUS
    };

    Ok(web::Json(failed).customize().with_status(status))
}

#[derive(Deserialize)]
struct NotificationFilter {
    pub user_ids: Vec<UserId>,
    #[serde(flatten)]
    pub body: serde_json::Map<String, serde_json::Value>,
}

#[delete("external_notifications", guard = "external_notification_key_guard")]
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

#[derive(Deserialize)]
struct SendEmail {
    pub users: Vec<UserId>,
    pub key: String,
    pub body_md: String,
    pub title: String,
}

#[post("external_notifications/send_custom_email")]
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
