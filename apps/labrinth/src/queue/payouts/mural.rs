use ariadne::ids::UserId;
use chrono::Utc;
use eyre::{Result, eyre};
use futures::{StreamExt, TryFutureExt, stream::FuturesUnordered};
use muralpay::{MuralError, MuralPay, TokenFeeRequest};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::warn;

use crate::{
    database::models::DBPayoutId,
    models::payouts::{PayoutMethodType, PayoutStatus},
    queue::payouts::{AccountBalance, PayoutFees, PayoutsQueue},
    routes::ApiError,
    util::{
        error::Context,
        gotenberg::{GotenbergClient, PaymentStatement},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MuralPayoutRequest {
    Fiat {
        bank_name: String,
        bank_account_owner: String,
        fiat_and_rail_details: muralpay::FiatAndRailDetails,
    },
    Blockchain {
        wallet_address: String,
    },
}

impl PayoutsQueue {
    pub async fn compute_muralpay_fees(
        &self,
        amount: Decimal,
        fiat_and_rail_code: muralpay::FiatAndRailCode,
    ) -> Result<muralpay::TokenPayoutFee, ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        let fees = muralpay
            .client
            .get_fees_for_token_amount(&[TokenFeeRequest {
                amount: muralpay::TokenAmount {
                    token_symbol: muralpay::USDC.into(),
                    token_amount: amount,
                },
                fiat_and_rail_code,
            }])
            .await
            .wrap_internal_err("failed to request fees")?;
        let fee = fees
            .into_iter()
            .next()
            .wrap_internal_err("no fees returned")?;
        Ok(fee)
    }

    pub async fn create_muralpay_payout_request(
        &self,
        payout_id: DBPayoutId,
        user_id: UserId,
        gross_amount: Decimal,
        fees: PayoutFees,
        payout_details: MuralPayoutRequest,
        recipient_info: muralpay::PayoutRecipientInfo,
        gotenberg: &GotenbergClient,
    ) -> Result<muralpay::PayoutRequest, ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        let payout_details = match payout_details {
            MuralPayoutRequest::Fiat {
                bank_name,
                bank_account_owner,
                fiat_and_rail_details,
            } => muralpay::CreatePayoutDetails::Fiat {
                bank_name,
                bank_account_owner,
                developer_fee: None,
                fiat_and_rail_details,
            },
            MuralPayoutRequest::Blockchain { wallet_address } => {
                muralpay::CreatePayoutDetails::Blockchain {
                    wallet_details: muralpay::WalletDetails {
                        // only Polygon chain is currently supported
                        blockchain: muralpay::Blockchain::Polygon,
                        wallet_address,
                    },
                }
            }
        };

        // Mural takes `fees.method_fee` off the top of the amount we tell them to send
        let sent_to_method = gross_amount - fees.platform_fee;
        // ..so the net is `gross - platform_fee - method_fee`
        let net_amount = gross_amount - fees.total_fee();

        let recipient_address = recipient_info.physical_address();
        let recipient_email = recipient_info.email().to_string();
        let gross_amount_cents = gross_amount * Decimal::from(100);
        let net_amount_cents = net_amount * Decimal::from(100);
        let fees_cents = fees.total_fee() * Decimal::from(100);
        let address_line_3 = format!(
            "{}, {}, {}",
            recipient_address.city,
            recipient_address.state,
            recipient_address.zip
        );

        let payment_statement = PaymentStatement {
            payment_id: payout_id.into(),
            recipient_address_line_1: Some(recipient_address.address1.clone()),
            recipient_address_line_2: recipient_address.address2.clone(),
            recipient_address_line_3: Some(address_line_3),
            recipient_email,
            payment_date: Utc::now(),
            gross_amount_cents: gross_amount_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("gross amount of cents `{gross_amount_cents}` cannot be expressed as an `i64`"))?,
            net_amount_cents: net_amount_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("net amount of cents `{net_amount_cents}` cannot be expressed as an `i64`"))?,
            fees_cents: fees_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("fees amount of cents `{fees_cents}` cannot be expressed as an `i64`"))?,
            currency_code: "USD".into(),
        };
        let payment_statement_doc = gotenberg
            .wait_for_payment_statement(&payment_statement)
            .await
            .wrap_internal_err("failed to generate payment statement")?;

        // TODO
        // std::fs::write(
        //     "/tmp/modrinth-payout-statement.pdf",
        //     base64::Engine::decode(
        //         &base64::engine::general_purpose::STANDARD,
        //         &payment_statement_doc.body,
        //     )
        //     .unwrap(),
        // )
        // .unwrap();

        let payout = muralpay::CreatePayout {
            amount: muralpay::TokenAmount {
                token_amount: sent_to_method,
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

        let payout_request = muralpay
            .client
            .create_payout_request(
                muralpay.source_account_id,
                Some(format!("User {user_id}")),
                &[payout],
            )
            .await
            .map_err(|err| match err {
                MuralError::Api(err) => ApiError::Request(err.into()),
                err => ApiError::Internal(
                    eyre!(err).wrap_err("failed to create payout request"),
                ),
            })?;

        // try to immediately execute the payout request...
        // use a poor man's try/catch block using this `async move {}`
        // to catch any errors within this block
        let result = async move {
            muralpay
                .client
                .execute_payout_request(payout_request.id)
                .await
                .wrap_internal_err("failed to execute payout request")?;
            eyre::Ok(())
        }
        .await;

        // and if it fails, make sure to immediately cancel it -
        // we don't want floating payout requests
        if let Err(err) = result {
            muralpay
                .client
                .cancel_payout_request(payout_request.id)
                .await
                .wrap_internal_err_with(|| {
                    eyre!("failed to cancel unexecuted payout request\noriginal error: {err:#?}")
                })?;
            return Err(ApiError::Internal(err));
        }

        Ok(payout_request)
    }

    pub async fn cancel_muralpay_payout_request(
        &self,
        id: muralpay::PayoutRequestId,
    ) -> Result<()> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_err("Mural Pay client not available")?;

        muralpay.client.cancel_payout_request(id).await?;
        Ok(())
    }

    pub async fn get_mural_balance(
        &self,
    ) -> Result<Option<AccountBalance>, ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        let account = muralpay
            .client
            .get_account(muralpay.source_account_id)
            .await
            .wrap_internal_err("failed to get source account")?;
        let details = account
            .account_details
            .wrap_internal_err("source account does not have details")?;
        let available = details
            .balances
            .iter()
            .map(|balance| {
                if balance.token_symbol == muralpay::USDC {
                    balance.token_amount
                } else {
                    Decimal::ZERO
                }
            })
            .sum::<Decimal>();
        Ok(Some(AccountBalance {
            available,
            pending: Decimal::ZERO,
        }))
    }
}

/// Finds Labrinth payouts which are not complete, fetches their corresponding
/// Mural state, and updates the payout status.
pub async fn sync_pending_payouts_from_mural(
    db: &PgPool,
    mural: &MuralPay,
    limit: u32,
) -> eyre::Result<()> {
    #[derive(Debug)]
    struct UpdatePayoutOp {
        payout_id: i64,
        status: PayoutStatus,
    }

    let mut txn = db
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let rows = sqlx::query!(
        "
        SELECT id, platform_id FROM payouts
        WHERE
            method = $1
            AND status = ANY($2::text[])
        LIMIT $3
        ",
        &PayoutMethodType::MuralPay.to_string(),
        &[
            PayoutStatus::InTransit,
            PayoutStatus::Unknown,
            PayoutStatus::Cancelling
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>(),
        i64::from(limit),
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch incomplete Mural payouts")?;

    let futs = rows.into_iter().map(|row| async move {
        let platform_id = row.platform_id.wrap_err("no platform ID")?;
        let payout_request_id = platform_id.parse::<muralpay::PayoutRequestId>()
            .wrap_err_with(|| eyre!("platform ID '{platform_id:?}' is not a valid payout request ID"))?;
        let payout_request = mural.get_payout_request(payout_request_id).await
            .wrap_err_with(|| eyre!("failed to fetch payout request {payout_request_id}"))?;

        let new_payout_status = match payout_request.status {
            muralpay::PayoutStatus::Canceled => Some(PayoutStatus::Cancelled),
            muralpay::PayoutStatus::Executed => Some(PayoutStatus::Success),
            muralpay::PayoutStatus::Failed => Some(PayoutStatus::Failed),
            _ => None,
        };

        if let Some(status) = new_payout_status {
            eyre::Ok(Some(UpdatePayoutOp {
                payout_id: row.id,
                status
            }))
        } else {
            eyre::Ok(None)
        }
    }.map_err(move |err| eyre!(err).wrap_err(eyre!("failed to update payout with ID '{}'", row.id))));
    let mut futs = futs.collect::<FuturesUnordered<_>>();

    let mut payout_ids = Vec::<i64>::new();
    let mut payout_statuses = Vec::<String>::new();

    while let Some(op) = futs.next().await.transpose()? {
        let Some(op) = op else { continue };
        payout_ids.push(op.payout_id);
        payout_statuses.push(op.status.to_string());
    }

    sqlx::query!(
        "
        UPDATE payouts
        SET status = u.status
        FROM UNNEST($1::bigint[], $2::varchar[]) AS u(id, status)
        WHERE payouts.id = u.id
        ",
        &payout_ids,
        &payout_statuses,
    )
    .execute(&mut *txn)
    .await
    .wrap_internal_err("failed to update payout statuses")?;

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}

/// Queries Mural for canceled or failed payouts, and updates the corresponding
/// Labrinth payouts' statuses.
pub async fn sync_failed_mural_payouts_to_labrinth(
    db: &PgPool,
    mural: &MuralPay,
    limit: u32,
) -> eyre::Result<()> {
    let mut next_id = None;
    loop {
        let search_resp = mural
            .search_payout_requests(
                Some(muralpay::PayoutStatusFilter::PayoutStatus {
                    statuses: vec![
                        muralpay::PayoutStatus::Canceled,
                        muralpay::PayoutStatus::Failed,
                    ],
                }),
                Some(muralpay::SearchParams {
                    limit: Some(u64::from(limit)),
                    next_id,
                }),
            )
            .await
            .wrap_internal_err(
                "failed to fetch failed payout requests from Mural",
            )?;
        next_id = search_resp.next_id;
        if search_resp.results.is_empty() {
            break;
        }

        let mut payout_platform_id = Vec::<String>::new();
        let mut payout_new_status = Vec::<String>::new();

        for payout_req in search_resp.results {
            let new_payout_status = match payout_req.status {
                muralpay::PayoutStatus::Canceled => PayoutStatus::Cancelled,
                muralpay::PayoutStatus::Failed => PayoutStatus::Failed,
                _ => {
                    warn!(
                        "Found payout {} with status {:?}, which should have been filtered out by our Mural request - Mural bug",
                        payout_req.id, payout_req.status
                    );
                    continue;
                }
            };

            payout_platform_id.push(payout_req.id.to_string());
            payout_new_status.push(new_payout_status.to_string());
        }

        sqlx::query!(
            "
            UPDATE payouts
            SET status = u.status
            FROM UNNEST($1::text[], $2::text[]) AS u(platform_id, status)
            WHERE
                payouts.method = $3
                AND payouts.platform_id = u.platform_id
            ",
            &payout_platform_id,
            &payout_new_status,
            PayoutMethodType::MuralPay.as_str(),
        )
        .execute(db)
        .await
        .wrap_internal_err("failed to update payout statuses")?;
    }

    Ok(())
}
