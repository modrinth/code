use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::env::ENV;
use actix_web::{get, web};
use arc_swap::ArcSwapOption;
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
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

static GLOBALS: LazyLock<Globals> = LazyLock::new(|| Globals {
    tax_compliance_thresholds: [(2025, 600), (2026, 2000)]
        .into_iter()
        .collect(),
    captcha_enabled: !ENV.HCAPTCHA_SECRET.is_empty()
        && ENV.HCAPTCHA_SECRET != "none",
});

struct TaxComplianceCache {
    year: i32,
    value: Option<Decimal>,
}

static TAX_COMPLIANCE_CACHE: ArcSwapOption<TaxComplianceCache> =
    ArcSwapOption::const_empty();

pub fn tax_compliance_payout_threshold() -> Option<Decimal> {
    tax_compliance_payout_threshold_for_year(Utc::now().year())
}

pub fn tax_compliance_payout_threshold_for_year(
    current_year: i32,
) -> Option<Decimal> {
    let cache = TAX_COMPLIANCE_CACHE.load();

    if let Some(cache) = &*cache
        && cache.year == current_year
    {
        return cache.value;
    }

    let value = (|| {
        if let Some(value_this_year) = GLOBALS
            .tax_compliance_thresholds
            .get(&(current_year as u16))
            .copied()
        {
            return Some(Decimal::from(value_this_year));
        }

        let mut years_to_values = GLOBALS
            .tax_compliance_thresholds
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect::<Vec<_>>();
        years_to_values.sort_by_key(|(year, _)| *year);

        let &(_, last_value) = years_to_values.last()?;
        Some(Decimal::from(last_value))
    })();

    TAX_COMPLIANCE_CACHE.store(Some(Arc::new(TaxComplianceCache {
        year: current_year,
        value,
    })));
    value
}

/// Gets configured global non-secret variables for this backend instance.
#[utoipa::path]
#[get("")]
pub async fn get_globals() -> web::Json<Globals> {
    web::Json(GLOBALS.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_rolls_over_by_year() {
        TAX_COMPLIANCE_CACHE.store(None);

        let first = tax_compliance_payout_threshold_for_year(2025);
        assert_eq!(first, Some(Decimal::from(600_u64)));

        let second = tax_compliance_payout_threshold_for_year(2026);
        assert_eq!(second, Some(Decimal::from(2000_u64)));

        let second = tax_compliance_payout_threshold_for_year(2027);
        assert_eq!(second, Some(Decimal::from(2000_u64)));

        TAX_COMPLIANCE_CACHE.store(None);
        let second = tax_compliance_payout_threshold_for_year(2027);
        assert_eq!(second, Some(Decimal::from(2000_u64)));
    }
}
