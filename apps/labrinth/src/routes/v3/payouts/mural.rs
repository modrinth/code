use actix_web::{HttpRequest, get, web};
use muralpay::FiatAndRailCode;
use sqlx::PgPool;
use strum::IntoEnumIterator;

use crate::{
    auth::{AuthenticationError, validate::get_user_record_from_bearer_token},
    database::redis::RedisPool,
    queue::{payouts::PayoutsQueue, session::AuthQueue},
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_bank_details);
}

#[get("/mural/bank-details")]
async fn get_bank_details(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    payouts_queue: web::Data<PayoutsQueue>,
) -> Result<web::Json<muralpay::BankDetailsResponse>, ApiError> {
    // even though we don't use the user, we ensure they're logged in to make API calls
    let (_, _user) = get_user_record_from_bearer_token(
        &req,
        None,
        &**pool,
        &redis,
        &session_queue,
    )
    .await?
    .ok_or_else(|| {
        ApiError::Authentication(AuthenticationError::InvalidCredentials)
    })?;

    let mural = payouts_queue.muralpay.load();
    let mural = mural
        .as_ref()
        .wrap_internal_err("Mural API not available")?;
    let fiat_and_rail_codes = FiatAndRailCode::iter().collect::<Vec<_>>();
    let details = mural
        .client
        .get_bank_details(&fiat_and_rail_codes)
        .await
        .wrap_internal_err("failed to fetch bank details")?;
    Ok(web::Json(details))
}
