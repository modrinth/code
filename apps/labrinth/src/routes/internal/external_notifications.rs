use crate::database::models::ids::DBUserId;
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::user_item::DBUser;
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::NotificationBody;
use crate::routes::ApiError;
use crate::util::guards::external_notification_key_guard;
use actix_web::web;
use actix_web::{HttpResponse, post};
use ariadne::ids::UserId;
use serde::Deserialize;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create);
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

    if !DBUser::exists_many(&user_ids, &mut *txn).await? {
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
