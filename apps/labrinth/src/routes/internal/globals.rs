use std::{collections::HashMap, sync::LazyLock};

use crate::env::ENV;
use actix_web::{get, web};
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(get_globals);
}

/// See [`get`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Globals {
    /// Map of years to how much a creator can withdraw in that year, in USD,
    /// before they must fill in a tax compliance form.
    ///
    /// If the current year is not contained in this map:
    /// - if the year is before the first year in the map, the threshold is the first year's.
    /// - if the year is after the last year in the map, the threshold is the last year's threshold.
    pub tax_compliance_thresholds: HashMap<u16, u64>,
    /// If this backend instance has a Captcha enabled for password login.
    ///
    /// In production, this will always be true. On local testing builds, this
    /// will always be false.
    pub captcha_enabled: bool,
}

pub static GLOBALS: LazyLock<Globals> = LazyLock::new(|| Globals {
    tax_compliance_thresholds: [(2025, 600), (2026, 2000)]
        .into_iter()
        .collect(),
    captcha_enabled: !ENV.HCAPTCHA_SECRET.is_empty()
        && ENV.HCAPTCHA_SECRET != "none",
});

/// Gets configured global non-secret variables for this backend instance.
#[utoipa::path]
#[get("")]
pub async fn get_globals() -> web::Json<Globals> {
    web::Json(GLOBALS.clone())
}
