use actix_web::web;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

mod admin;
mod fetch;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(fetch::get_runs)
        .service(admin::start_run)
        .service(admin::cancel_runs);
}

/// Manual admin-input adjustment to a payout period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    /// Total value of the adjustment.
    #[serde(with = "rust_decimal::serde::float")]
    pub amount_usd: Decimal,
    /// Why this adjustment was applied.
    ///
    /// Only visible to admins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
