use actix_web::{HttpRequest, HttpResponse, post, web};
use ariadne::ids::UserId;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::models::users_redeemals::{Offer, RedeemalLookupFields};
use crate::routes::ApiError;
use crate::util::guards::medal_key_guard;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("medal").service(verify).service(redeem));
}

#[derive(Deserialize)]
struct MedalVerifyQuery {
    username: String,
}

#[post("verify", guard = "medal_key_guard")]
pub async fn verify(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Query(MedalVerifyQuery { username }): web::Query<MedalVerifyQuery>,
) -> Result<HttpResponse, ApiError> {
    let maybe_fields =
        RedeemalLookupFields::redeemal_status_by_user_username_and_offer(
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
    _req: HttpRequest,
    _pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::NotImplemented().finish())
}
