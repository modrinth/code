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
//!   - `raw_estimated_revenue_usd`: how much our ad provider estimates we'll
//!     earn for a specific period
//!     - We also get a value for this per day
//!   - `fees_usd`: how much we pay in fees to Clean.io
//!     - Based on number of impressions; we can get a per-day value for this
//!   - `variance_usd`: a fixed percentage that we subtract from the raw estimated
//!     revenue to account for it being an overestimate
//!     - e.g. if variance is 10%, and we estimate that we'll earn $100k, then
//!       our ad provider will probably give us closer to $90k - the variance
//!       lets us express this difference
//!   - `net_estimated_revenue_usd`: raw estimated - fees - variance
//!   - `platform_net_estimated_revenue_usd`: net estimated revenue x Modrinth's cut
//!   - `creator_net_estimated_revenue_usd`: net estimated revenue x (1 - Modrinth's cut)
//! - After a payout run has been executed:
//!   - We save the per-day raw estimated revenue and impressions in the
//!     database
//!   - `raw_actual_revenue_usd`: how much we got from Aditude, input by an admin
//!     - We compute this per-day by:
//!       ```text
//!       let factor = raw_actual_revenue_usd / raw_estimated_revenue_usd
//!       raw_actual_revenue_usd_for_today = raw_estimated_revenue_usd_for_today * factor
//!       ```
//!   - (fees stay the same, since they're based on impressions, not revenue)
//!   - (variance is ignored, since that's purely an estimation value)
//!   - `adjustments_usd`: sum of all manual adjustments input by the admin
//!   - `net_actual_revenue_usd`: raw actual revenue - fees + adjustments
//!   - `(platform|creator)_net_estimated_revenue_usd`: same logic as estimated,
//!     but using the net actual revenue
//!
//! ## Variance
//!
//! We store a table `payouts_variance` with columns:
//! - a timestamp from when this variance value applies (first entry at Unix
//!   epoch)
//! - the decimal fraction of variance to apply

mod estimate;
