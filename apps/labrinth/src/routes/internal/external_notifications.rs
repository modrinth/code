use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::ids::DBUserId;
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
    CustomizeResponder, HttpRequest, HttpResponse, Responder, post,
};
use ariadne::ids::UserId;
use eyre::eyre;
use lettre::message::Mailbox;
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(create_direct_email)
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

/// Directly sends emails to users and inserts notifications when emails are
/// delivered successfully.
///
/// Responds with the user IDs that could not be emailed:
/// - `200` if every recipient was delivered (empty list)
/// - `207` if some recipients failed (list of failed IDs)
/// - `500` if no recipient was delivered
#[post(
    "external_notifications/direct-email",
    guard = "external_notification_key_guard"
)]
pub async fn create_direct_email(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    email_queue: web::Data<EmailQueue>,
    create_notification: web::Json<CreateNotification>,
) -> Result<CustomizeResponder<web::Json<Vec<UserId>>>, ApiError> {
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

    let mut results: Vec<Result<DBUserId, DBUserId>> =
        Vec::with_capacity(user_ids.len());

    for user_id in &user_ids {
        let user = DBUser::get_id(*user_id, &mut txn, &redis).await?.ok_or(
            ApiError::Internal(eyre!(
                "user `{}` disappeared while sending notification email",
                user_id.0
            )),
        )?;

        let delivered =
            match user.email.and_then(|email| email.parse::<Mailbox>().ok()) {
                Some(mailbox) => {
                    email_queue
                        .send_one(&mut txn, body.clone(), *user_id, mailbox)
                        .await?
                        == NotificationDeliveryStatus::Delivered
                }
                None => false,
            };

        results.push(if delivered {
            Ok(*user_id)
        } else {
            Err(*user_id)
        });
    }

    let delivered = results
        .iter()
        .filter_map(|result| result.as_ref().ok().copied())
        .collect::<Vec<_>>();

    if delivered.is_empty() {
        return Err(ApiError::Internal(eyre!(
            "failed to deliver notification email to any of {} recipients",
            user_ids.len(),
        )));
    }

    NotificationBuilder { body }
        .insert_many_without_delivery(delivered, &mut txn, &redis)
        .await?;

    txn.commit().await?;

    let failed = results
        .into_iter()
        .filter_map(|result| result.err().map(|id| UserId(id.0 as u64)))
        .collect::<Vec<_>>();

    let status = if failed.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::MULTI_STATUS
    };

    Ok(web::Json(failed).customize().with_status(status))
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
