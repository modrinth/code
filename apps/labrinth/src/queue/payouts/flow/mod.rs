//! Centralized place where payout rails are defined - their fees, minimum and
//! maximum withdraw amounts, and execution logic.

use eyre::eyre;
use modrinth_util::decimal::Decimal2dp;
use rust_decimal::Decimal;
use sqlx::PgTransaction;
use thiserror::Error;

pub mod mural;
pub mod paypal;
pub mod tremendous;

use crate::{
    database::models::{DBPayoutId, DBUser},
    models::payouts::{PayoutMethodRequest, Withdrawal},
    queue::payouts::PayoutsQueue,
    routes::ApiError,
    util::{error::Context, gotenberg::GotenbergClient},
};

impl PayoutsQueue {
    /// Begins a payout creation flow.
    ///
    /// A payout creation flow is preparation for sending a user some amount of
    /// money, but does not actually send the money until [`PayoutFlow::execute`]
    /// is called. This allows callers to get information like the payout fee,
    /// minimum, and maximum amounts for validation before actually sending the
    /// payout.
    pub async fn create_payout_flow(
        &self,
        withdrawal: Withdrawal,
    ) -> Result<PayoutFlow, ApiError> {
        let get_method = async {
            let method = self
                .get_payout_methods()
                .await
                .wrap_internal_err("failed to fetch payout methods")?
                .into_iter()
                .find(|method| method.id == withdrawal.method_id)
                .wrap_request_err("invalid payout method ID")?;
            Ok::<_, ApiError>(method)
        };

        match withdrawal.method {
            PayoutMethodRequest::PayPal => {
                paypal::create(self, withdrawal.amount, false).await
            }
            PayoutMethodRequest::Venmo => {
                paypal::create(self, withdrawal.amount, true).await
            }
            PayoutMethodRequest::MuralPay { method_details } => {
                mural::create(self, withdrawal.amount, method_details).await
            }
            PayoutMethodRequest::Tremendous { method_details } => {
                tremendous::create(
                    self,
                    withdrawal.amount,
                    method_details,
                    &get_method.await?,
                )
                .await
            }
        }
    }
}

#[derive(Debug)]
pub struct PayoutFlow {
    /// Net amount that the user receives after fees, in USD.
    pub net_usd: Decimal2dp,
    /// Total payout fee, in USD.
    pub total_fee_usd: Decimal2dp,
    /// Minimum payout amount, in USD.
    pub min_amount_usd: Decimal2dp,
    /// Maximum payout amount, in USD.
    pub max_amount_usd: Decimal2dp,
    /// Currency conversion rate from USD to the payout currency.
    pub forex_usd_to_currency: Option<Decimal>,
    inner: PayoutFlowInner,
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant)]
enum PayoutFlowInner {
    PayPal(paypal::PayPalFlow),
    Mural(mural::MuralFlow),
    Tremendous(tremendous::TremendousFlow),
}

struct ExecuteContext<'a> {
    queue: &'a PayoutsQueue,
    user: &'a DBUser,
    payout_id: DBPayoutId,
    transaction: PgTransaction<'a>,
    gotenberg: &'a GotenbergClient,
}

#[derive(Debug)]
pub struct ReadyPayoutFlow {
    inner: PayoutFlowInner,
}

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("withdraw amount below minimum")]
    BelowMin,
    #[error("withdraw amount above maximum")]
    AboveMax,
}

impl PayoutFlow {
    /// Checks that this payout can be sent if the recipient has the specified
    /// balance.
    pub fn validate(
        self,
        balance_usd: Decimal,
    ) -> Result<ReadyPayoutFlow, ValidateError> {
        let gross_usd = self.net_usd + self.total_fee_usd;
        if balance_usd < gross_usd {
            return Err(ValidateError::InsufficientBalance);
        }
        if gross_usd < self.min_amount_usd {
            return Err(ValidateError::BelowMin);
        }
        if gross_usd > self.max_amount_usd {
            return Err(ValidateError::AboveMax);
        }
        Ok(ReadyPayoutFlow { inner: self.inner })
    }
}

impl ReadyPayoutFlow {
    /// Executes this payout.
    pub async fn execute(
        self,
        queue: &PayoutsQueue,
        user: &DBUser,
        payout_id: DBPayoutId,
        transaction: PgTransaction<'_>,
        gotenberg: &GotenbergClient,
    ) -> Result<(), ApiError> {
        let cx = ExecuteContext {
            queue,
            user,
            payout_id,
            transaction,
            gotenberg,
        };

        match self.inner {
            PayoutFlowInner::PayPal(flow) => paypal::execute(cx, flow).await,
            PayoutFlowInner::Mural(flow) => mural::execute(cx, flow).await,
            PayoutFlowInner::Tremendous(flow) => {
                tremendous::execute(cx, flow).await
            }
        }
    }
}

fn get_verified_email(user: &DBUser) -> Result<&str, ApiError> {
    let email = user.email.as_ref().wrap_request_err(
        "you must add an email to your account to withdraw",
    )?;
    if !user.email_verified {
        return Err(ApiError::Request(eyre!(
            "you must verify your email to withdraw"
        )));
    }

    Ok(email)
}
