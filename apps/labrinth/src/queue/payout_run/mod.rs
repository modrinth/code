//! Logic for executing payout runs, including starting a run, and performing
//! the revenue distribution.
//!
//! # Architecture
//!
//! ## Payout run
//!
//! The general flow for a payout run is as follows:
//! - For a given month (say, January) our ad provider gives us an estimate of
//!   how much revenue and how many impressions we received for each individual
//!   day. The month (payout period) starts in an _open_ state.
//! - After NET 60 has passed (start of March), the January payouts _should_ be
//!   available; usually it takes some time for our ad provider to send the
//!   money, so this is closer to NET 75. During this period, the month (payout
//!   period) is in an _in review_ state.
//! - Once we receive the money from the provider, an admin enters the total
//!   amount we've received into the web UI, adds any manual adjustments (for
//!   campaigns outside of our ad provider's), and starts a payout run.
//! - The payout run is not immediately executed; there is a period of time in
//!   which it can still be cancelled.
//! - Once the payout run is executed, we calculate the exact revenue
//!   distribution to all creators, and fill `payouts_values` with those
//!   amounts.
//!
//! ## Distribution
//!
//! How revenue is distributed:
//! - While a month is still open/in review:
//!   - `estimated.raw_revenue_usd`: how much our ad provider estimates we'll
//!     earn for a specific period
//!     - We also get a value for this per day
//!   - `fees_usd`: how much we pay in fees to Clean.io
//!     - Based on number of impressions; we can get a per-day value for this
//!   - `variance_usd`: a fixed percentage that we subtract from the raw estimated
//!     revenue to account for it being an overestimate
//!     - e.g. if variance is 10%, and we estimate that we'll earn $100k, then
//!       our ad provider will probably give us closer to $90k - the variance
//!       lets us express this difference
//!   - `net_revenue_usd`: raw estimated - fees - variance
//!   - `platform_net_revenue_usd`: net estimated revenue x Modrinth's cut
//!   - `creator_net_revenue_usd`: net estimated revenue x (1 - Modrinth's cut)
//! - After a payout run has been executed:
//!   - We save the per-day raw estimated revenue and impressions in the
//!     database
//!   - `actual.raw_revenue_usd`: how much we got from Aditude, input by an admin
//!     - We compute this per-day by:
//!       ```text
//!       let factor = actual.raw_revenue_usd / raw_estimated_revenue_usd
//!       raw_revenue_usd_for_today = raw_estimated_revenue_usd_for_today * factor
//!       ```
//!   - (fees stay the same, since they're based on impressions, not revenue)
//!   - (variance is ignored, since that's purely an estimation value)
//!   - `adjustments_usd`: sum of all manual adjustments input by the admin
//!   - `actual.net_revenue_usd`: raw actual revenue - fees + adjustments
//!   - `actual.(platform|creator)_net_revenue_usd`: same logic as estimated,
//!     but using the net actual revenue
//!
//! ## Variance
//!
//! We store a table `payouts_variance` with columns:
//! - a date from when this variance value applies (first entry on the Unix
//!   epoch date)
//! - the decimal fraction of variance to apply

use chrono::NaiveDate;
use rust_decimal::{Decimal, dec};
use serde::{Deserialize, Serialize};

mod estimate;

pub use estimate::*;

/// Fraction defining much of the net revenue goes to the platform.
const PLATFORM_REVENUE_SPLIT: Decimal = dec!(0.25);

/// How input revenue is distributed for a specific day.
///
/// This may refer to either estimated or actual revenue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayDistribution {
    /// Amount of revenue input into the algorithm.
    pub raw_revenue_usd: Decimal,
    /// Operational fees to subtract.
    pub fees_usd: Decimal,
    /// Estimation variance to subtract.
    ///
    /// For non-estimates (actual revenue values), this is zero.
    pub variance_usd: Decimal,
    /// Manually-input adjustments on top of raw revenue.
    pub sum_adjustments_usd: Decimal,
    /// Total net revenue that we earned;
    /// `raw_revenue - fees - variance + sum_adjustments`.
    pub net_revenue_usd: Decimal,
    /// How much of the net revenue goes to the platform.
    pub platform_net_revenue_usd: Decimal,
    /// How much of the net revenue goes to creators.
    pub creator_net_revenue_usd: Decimal,
}

#[derive(Debug)]
pub struct PayoutVariance {
    pub starts_at: NaiveDate,
    pub frac: Decimal,
}

#[derive(Debug)]
pub struct PayoutVariances {
    pub fracs: Vec<PayoutVariance>,
    pub default_frac: Decimal,
}

impl PayoutVariances {
    pub const ZERO: Self = Self {
        fracs: Vec::new(),
        default_frac: Decimal::ZERO,
    };
}

/// Compute the [`DayDistribution`] for an input revenue and impressions amount.
///
/// This logic may be used on both estimated and actual revenue. For actual
/// revenue, [`PayoutVariances::ZERO`] should be used, since variance only
/// applies to estimated days.
pub fn distribution_for_day(
    date: NaiveDate,
    raw_revenue_usd: Decimal,
    impressions: u128,
    sum_adjustments_usd: Decimal,
    variances: &PayoutVariances,
) -> DayDistribution {
    let fees_usd = {
        let clean_io_cpm = Decimal::from(8) / Decimal::from(1000);
        clean_io_cpm * Decimal::from(impressions) / Decimal::from(1000)
    };
    let variance_frac = variances
        .fracs
        .iter()
        .find(|v| v.starts_at >= date)
        .map(|v| v.frac)
        .unwrap_or(variances.default_frac);
    let variance_usd = raw_revenue_usd * variance_frac;

    let net_estimated_revenue_usd =
        raw_revenue_usd - fees_usd - variance_usd + sum_adjustments_usd;

    DayDistribution {
        raw_revenue_usd,
        fees_usd,
        variance_usd,
        sum_adjustments_usd,
        net_revenue_usd: net_estimated_revenue_usd,
        platform_net_revenue_usd: net_estimated_revenue_usd
            * PLATFORM_REVENUE_SPLIT,
        creator_net_revenue_usd: net_estimated_revenue_usd
            * (dec!(1) - PLATFORM_REVENUE_SPLIT),
    }
}
