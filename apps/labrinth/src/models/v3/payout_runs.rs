use ariadne::ids::UserId;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::util::time::YearMonth;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PayoutRun {
    /// What period this payout run is for.
    ///
    /// Payout runs are always for the period of a specific year and month -
    /// they are not associated with any specific day.
    pub period_start: YearMonth,
    /// What state this run is in.
    pub status: PayoutRunStatus,
    #[serde(flatten)]
    pub report: PayoutRunReport,
    /// When this run started running.
    ///
    /// Only accessible to admins.
    pub started_at: Option<DateTime<Utc>>,
    /// What user started this run.
    ///
    /// Only accessible to admins.
    pub started_by: Option<UserId>,
    /// When this run completed.
    ///
    /// Only accessible to admins.
    pub completed_at: Option<DateTime<Utc>>,
    /// What payout adjustments were specified in this run.
    ///
    /// Only accessible to admins.
    pub adjustments: Option<Vec<Adjustment>>,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PayoutRunStatus {
    /// The payout period is still receiving revenue estimates.
    Open,
    /// The payout period is closed, but is still within Net-60 terms.
    Pending,
    /// The ad provider should have issued payouts to us by now, and we will
    /// soon run the payouts.
    Review,
    /// Payouts run is complete and payouts have been distributed to users.
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PayoutRunReport {
	pub days: Vec<DayRevenue>,
	#[serde(with = "rust_decimal::serde::float")]
	pub fees_deducted_usd: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_adjustment_usd: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub net_estimated_revenue_usd: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub creator_net_estimated_revenue_usd: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub modrinth_net_estimated_revenue_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PayoutRunCompletion {
    #[serde(with = "rust_decimal::serde::float")]
    pub revenue_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DayRevenue {
    pub date: NaiveDate,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Adjustment {}
