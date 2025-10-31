use actix_web::{get, web};
use muralpay::FiatAndRailCode;
use strum::IntoEnumIterator;

use crate::{
    queue::payouts::PayoutsQueue, routes::ApiError, util::error::Context,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_bank_details);
}

#[get("/mural/bank-details")]
async fn get_bank_details(
    payouts_queue: web::Data<PayoutsQueue>,
) -> Result<web::Json<muralpay::BankDetailsResponse>, ApiError> {
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
