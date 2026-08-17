use actix_web::{get, web};

use crate::{queue::payout_run::DayDistribution, routes::ApiError};

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRuns {
    pub periods: Vec<PayoutRunPeriod>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRunPeriod {
    pub period: YearMonth,
    pub status: PayoutRunStatus,
    pub days: Vec<PayoutRunDay>,
    pub adjustments: Vec<PayoutRunAdjustment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRunDay {
    pub date: NaiveDate,
    pub estimated: DayDistribution,
    pub actual: Option<DayDistribution>,
}

/// Manual admin-input adjustment to a [`PayoutRunPeriod`].
#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRunAdjustment {
    /// Total value of the adjustment.
    pub amount_usd: Decimal,
    /// Why this adjustment was applied.
    ///
    /// Only visible to admins.
    pub description: Option<String>,
}

#[get("/")]
pub async fn get_runs() -> Result<web::Json<PayoutRuns>, ApiError> {}
