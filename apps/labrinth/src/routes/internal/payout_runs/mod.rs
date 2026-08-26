mod admin;
mod fetch;

pub use crate::queue::payout_run::{Adjustment, PayoutRunPayload};
pub use admin::*;
pub use fetch::*;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_runs)
        .service(calculate_run)
        .service(start_run)
        .service(cancel_runs);
}
