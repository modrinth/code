use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
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
    /// We are still waiting on the ad provider to issue payouts to us.
    Pending,
    /// The ad provider should have issued payouts to us by now, and we will
    /// soon run the payouts.
    Review,
    /// Payouts run is currently being performed.
    Running,
    /// Payouts run is complete and payouts have been distributed to users.
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Adjustment {}
