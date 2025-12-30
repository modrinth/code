use eyre::eyre;
use rust_decimal::Decimal;
use sqlx::{PgPool, PgTransaction};

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
    pub async fn create_payout_flow(
        &self,
        withdrawal: Withdrawal,
    ) -> Result<PayoutFlow, ApiError> {
        match withdrawal.method {
            PayoutMethodRequest::PayPal => {
                paypal::create(self, withdrawal.amount, false).await
            }
            PayoutMethodRequest::Venmo => {
                paypal::create(self, withdrawal.amount, false).await
            }
            PayoutMethodRequest::MuralPay { method_details } => {
                mural::create(self, withdrawal.amount, method_details).await
            }
            PayoutMethodRequest::Tremendous { method_details } => {
                tremendous::create(self, withdrawal.amount, method_details)
                    .await
            }
        }
    }
}

#[derive(Debug)]
pub struct PayoutFlow {
    pub total_fee: Decimal,
    pub forex_usd_to_currency: Option<Decimal>,
    inner: PayoutFlowInner,
}

#[derive(Debug)]
enum PayoutFlowInner {
    PayPal(paypal::PayPalFlow),
    Mural(mural::MuralFlow),
    Tremendous(tremendous::TremendousFlow),
}

struct ExecuteContext<'a> {
    queue: &'a PayoutsQueue,
    user: &'a DBUser,
    db: &'a PgPool,
    payout_id: DBPayoutId,
    transaction: PgTransaction<'a>,
    gotenberg: &'a GotenbergClient,
}

impl PayoutFlow {
    pub async fn execute(
        self,
        queue: &PayoutsQueue,
        user: &DBUser,
        db: &PgPool,
        payout_id: DBPayoutId,
        transaction: PgTransaction<'_>,
        gotenberg: &GotenbergClient,
    ) -> Result<(), ApiError> {
        let cx = ExecuteContext {
            queue,
            user,
            db,
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
