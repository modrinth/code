use ariadne::ids::UserId;
use chrono::Utc;
use eyre::eyre;
use modrinth_util::decimal::Decimal2dp;
use muralpay::FiatAndRailCode;
use rust_decimal::{Decimal, RoundingStrategy, dec};
use tracing::error;

use crate::{
    database::models::payout_item::DBPayout,
    models::payouts::{
        MuralPayDetails, PayoutMethodFee, PayoutMethodType, PayoutStatus,
    },
    queue::payouts::{
        PayoutsQueue,
        flow::{
            ExecuteContext, PayoutFlow, PayoutFlowInner, get_verified_email,
        },
        mural::MuralPayoutRequest,
    },
    routes::ApiError,
    util::error::Context,
};

pub const PLATFORM_FEE: PayoutMethodFee = PayoutMethodFee {
    percentage: dec!(0.01),
    min: Decimal::ZERO,
    max: None,
};

// USDC has much lower fees.
pub const MIN_USD_BLOCKCHAIN: Decimal2dp = Decimal2dp::new_unchecked(dec!(0.1));

pub fn min_usd_fiat(fiat_and_rail_code: FiatAndRailCode) -> Decimal2dp {
    match fiat_and_rail_code {
        // Due to relatively low volume of Peru withdrawals, fees are higher,
        // so we need to raise the minimum to cover these fees.
        FiatAndRailCode::UsdPeru => Decimal2dp::new(dec!(10.0)),
        _ => Decimal2dp::new(dec!(5.0)),
    }
    .unwrap()
}

pub const MAX_USD: Decimal2dp = Decimal2dp::new_unchecked(dec!(10_000.0));

#[derive(Debug)]
pub(super) struct MuralFlow {
    net_usd: Decimal2dp,
    method_fee_usd: Decimal2dp,
    platform_fee_usd: Decimal2dp,
    payout_details: MuralPayoutRequest,
    recipient_info: muralpay::CreatePayoutRecipientInfo,
}

pub(super) async fn create(
    queue: &PayoutsQueue,
    amount: Decimal,
    details: MuralPayDetails,
) -> Result<PayoutFlow, ApiError> {
    let gross_usd =
        Decimal2dp::new(amount).wrap_request_err("invalid amount")?;
    let platform_fee_usd = Decimal2dp::rounded(
        PLATFORM_FEE.compute_fee(gross_usd),
        RoundingStrategy::AwayFromZero,
    );

    let mural = queue.muralpay.load();
    let mural = mural
        .as_ref()
        .wrap_internal_err("Mural client not available")?;

    let method_fee_usd;
    let forex_usd_to_currency;
    let min_amount_usd;

    match &details.payout_details {
        MuralPayoutRequest::Blockchain { .. } => {
            method_fee_usd = Decimal2dp::ZERO;
            forex_usd_to_currency = None;
            min_amount_usd = MIN_USD_BLOCKCHAIN;
        }
        MuralPayoutRequest::Fiat {
            fiat_and_rail_details,
            ..
        } => {
            let fiat_and_rail_code = fiat_and_rail_details.code();

            let fees = mural
                .client
                .get_fees_for_token_amount(&[muralpay::TokenFeeRequest {
                    amount: muralpay::TokenAmount {
                        token_symbol: muralpay::USDC.into(),
                        token_amount: gross_usd.get(),
                    },
                    fiat_and_rail_code,
                }])
                .await
                .wrap_internal_err("failed to request fees")?;
            let fee = fees
                .into_iter()
                .next()
                .wrap_internal_err("no fees returned")?;

            match fee {
                muralpay::TokenPayoutFee::Success {
                    exchange_rate,
                    fee_total,
                    ..
                } => {
                    method_fee_usd = Decimal2dp::rounded(
                        fee_total.token_amount,
                        RoundingStrategy::AwayFromZero,
                    );
                    forex_usd_to_currency = Some(exchange_rate);
                    min_amount_usd = min_usd_fiat(fiat_and_rail_code);
                }
                muralpay::TokenPayoutFee::Error { message, .. } => {
                    return Err(ApiError::Internal(eyre!(
                        "failed to compute fee: {message}"
                    )));
                }
            }
        }
    };

    let total_fee_usd = method_fee_usd + platform_fee_usd;
    let net_usd = gross_usd - total_fee_usd;

    Ok(PayoutFlow {
        net_usd,
        total_fee_usd,
        min_amount_usd,
        max_amount_usd: MAX_USD,
        forex_usd_to_currency,
        inner: PayoutFlowInner::Mural(MuralFlow {
            net_usd,
            method_fee_usd,
            platform_fee_usd,
            payout_details: details.payout_details,
            recipient_info: details.recipient_info,
        }),
    })
}

pub(super) async fn execute(
    ExecuteContext {
        queue,
        user,
        payout_id,
        mut transaction,
        gotenberg,
    }: ExecuteContext<'_>,
    MuralFlow {
        net_usd,
        method_fee_usd,
        platform_fee_usd,
        payout_details,
        recipient_info,
    }: MuralFlow,
) -> Result<(), ApiError> {
    let user_email = get_verified_email(user)?;
    let sent_to_method_usd = net_usd + method_fee_usd;
    let total_fee_usd = method_fee_usd + platform_fee_usd;

    let mural = queue.muralpay.load();
    let mural = mural
        .as_ref()
        .wrap_internal_err("Mural client not available")?;

    let payment_statement_doc = queue
        .create_mural_payment_statement_doc(
            payout_id,
            net_usd,
            total_fee_usd,
            &recipient_info,
            gotenberg,
        )
        .await?;

    let user_id = UserId::from(user.id);
    let method_id = match &payout_details {
        MuralPayoutRequest::Blockchain { .. } => {
            "blockchain-usdc-polygon".to_string()
        }
        MuralPayoutRequest::Fiat {
            fiat_and_rail_details,
            ..
        } => fiat_and_rail_details.code().to_string(),
    };

    let payout_details = match payout_details {
        crate::queue::payouts::mural::MuralPayoutRequest::Fiat {
            bank_name,
            bank_account_owner,
            fiat_and_rail_details,
        } => muralpay::CreatePayoutDetails::Fiat {
            bank_name,
            bank_account_owner,
            developer_fee: None,
            fiat_and_rail_details,
        },
        crate::queue::payouts::mural::MuralPayoutRequest::Blockchain {
            wallet_address,
        } => {
            muralpay::CreatePayoutDetails::Blockchain {
                wallet_details: muralpay::WalletDetails {
                    // only Polygon chain is currently supported
                    blockchain: muralpay::Blockchain::Polygon,
                    wallet_address,
                },
            }
        }
    };

    let payout = muralpay::CreatePayout {
        amount: muralpay::TokenAmount {
            token_amount: sent_to_method_usd.get(),
            token_symbol: muralpay::USDC.into(),
        },
        payout_details,
        recipient_info,
        supporting_details: Some(muralpay::SupportingDetails {
            supporting_document: Some(format!(
                "data:application/pdf;base64,{}",
                payment_statement_doc.body
            )),
            payout_purpose: Some(muralpay::PayoutPurpose::VendorPayment),
        }),
    };

    let payout_request = mural
        .client
        .create_payout_request(
            mural.source_account_id,
            Some(format!("User {user_id}")),
            &[payout],
        )
        .await
        .map_err(|err| match err {
            muralpay::MuralError::Api(err) => ApiError::Mural(Box::new(err)),
            err => ApiError::Internal(
                eyre!(err).wrap_err("failed to create payout request"),
            ),
        })?;

    // Once the Mural payout request has been created successfully,
    // then we *must* commit *a* payout row into the DB, to link the Mural
    // payout request to the `payout` row, and to subtract the user's balance.
    // Even if we can't execute the payout afterwards.
    // For this, we create a payout, try to execute it, and no matter what
    // happens, insert the payout row.
    // Otherwise if we don't put it into the DB, we've got a ghost Mural
    // payout with no related database entry.
    // However, this doesn't mean that the payout will definitely go through.
    // For this, we need to execute it, and handle errors.

    let mut payout = DBPayout {
        id: payout_id,
        user_id: user.id,
        created: Utc::now(),
        // after the payout has been successfully executed,
        // we wait for Mural's confirmation that the funds have been delivered
        // done in `SyncPayoutStatuses` background task
        status: PayoutStatus::InTransit,
        amount: net_usd.get(),
        fee: Some(total_fee_usd.get()),
        method: Some(PayoutMethodType::MuralPay),
        method_id: Some(method_id),
        method_address: Some(user_email.to_string()),
        platform_id: Some(payout_request.id.to_string()),
    };

    // poor man's async try/catch block
    let result = (async {
        mural
            .client
            .execute_payout_request(payout_request.id)
            .await
            .wrap_internal_err("failed to execute payout request")?;
        Ok::<_, ApiError>(())
    })
    .await;

    if let Err(caught_err) = result {
        payout.status = PayoutStatus::Failed;

        // if execution fails, make sure to immediately cancel the payout request
        // we don't want floating payout requests
        if let Err(err) =
            queue.cancel_mural_payout_request(payout_request.id).await
        {
            error!(
                "Failed to cancel unexecuted payout request: {err:#}\noriginal error: {caught_err:#}"
            );
        }
    }

    payout
        .insert(&mut transaction)
        .await
        .wrap_internal_err("failed to insert payout")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}
