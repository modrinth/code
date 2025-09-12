use actix_web::{HttpResponse, post, web};
use ariadne::ids::UserId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::warn;

use crate::database::models::users_redeemals::{
    Offer, RedeemalLookupFields, Status, UserRedeemal,
};
use crate::database::redis::RedisPool;
use crate::queue::billing::try_process_user_redeemal;
use crate::routes::ApiError;
use crate::util::guards::medal_key_guard;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("medal").service(verify).service(redeem));
}

#[derive(Deserialize)]
struct MedalQuery {
    username: String,
}

#[post("verify", guard = "medal_key_guard")]
pub async fn verify(
    pool: web::Data<PgPool>,
    web::Query(MedalQuery { username }): web::Query<MedalQuery>,
) -> Result<HttpResponse, ApiError> {
    let maybe_fields =
        RedeemalLookupFields::redeemal_status_by_username_and_offer(
            &**pool,
            &username,
            Offer::Medal,
        )
        .await?;

    #[derive(Serialize)]
    struct VerifyResponse {
        user_id: UserId,
        redeemed: bool,
    }

    match maybe_fields {
        None => Err(ApiError::NotFound),
        Some(fields) => Ok(HttpResponse::Ok().json(VerifyResponse {
            user_id: fields.user_id.into(),
            redeemed: fields.redeemal_status.is_some(),
        })),
    }
}

#[post("redeem", guard = "medal_key_guard")]
pub async fn redeem(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    web::Query(MedalQuery { username }): web::Query<MedalQuery>,
) -> Result<HttpResponse, ApiError> {
    // Check the offer hasn't been redeemed yet, then insert into the table.
    // In a transaction to avoid double inserts.

    let mut txn = pool.begin().await?;

    let maybe_fields =
        RedeemalLookupFields::redeemal_status_by_username_and_offer(
            &mut *txn,
            &username,
            Offer::Medal,
        )
        .await?;

    let user_id = match maybe_fields {
        None => return Err(ApiError::NotFound),
        Some(fields) => {
            if fields.redeemal_status.is_some() {
                return Err(ApiError::Conflict(
                    "User already redeemed this offer".to_string(),
                ));
            }

            fields.user_id
        }
    };

    // Link user to offer redeemal.
    let mut redeemal = UserRedeemal {
        id: 0,
        user_id,
        offer: Offer::Medal,
        redeemed: Utc::now(),
        status: Status::Pending,
        last_attempt: None,
        n_attempts: 0,
    };

    redeemal.insert(&mut *txn).await?;

    txn.commit().await?;

    // Immediately try to process the redeemal
    if let Err(error) = try_process_user_redeemal(&pool, &redis, redeemal).await
    {
        warn!(%error, "Medal redeemal processing failed");

        Ok(HttpResponse::Accepted().finish())
    } else {
        Ok(HttpResponse::Created().finish())
    }
}
