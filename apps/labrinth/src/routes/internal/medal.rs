use actix_web::{HttpResponse, post, web};
use ariadne::ids::UserId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::models::users_redeemals::{
    Offer, RedeemalLookupFields, Status, UserRedeemal,
};
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
    web::Query(MedalQuery { username }): web::Query<MedalQuery>,
) -> Result<HttpResponse, ApiError> {
    // Check the offer hasn't been redeemed yet, then insert into the table.

    let mut txn = pool.begin().await?;

    let maybe_fields =
        RedeemalLookupFields::redeemal_status_by_username_and_offer(
            &mut *txn,
            &username,
            Offer::Medal,
        )
        .await?;

    let _redeemal = match maybe_fields {
        None => return Err(ApiError::NotFound),
        Some(fields) => {
            if fields.redeemal_status.is_some() {
                return Err(ApiError::Conflict(
                    "User already redeemed this offer".to_string(),
                ));
            }

            let mut redeemal = UserRedeemal {
                id: 0,
                user_id: fields.user_id,
                offer: Offer::Medal,
                redeemed: Utc::now(),
                status: Status::Pending,
            };

            redeemal.insert(&mut *txn).await?;
            redeemal
        }
    };

    txn.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
