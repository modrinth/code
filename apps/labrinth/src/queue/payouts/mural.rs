use ariadne::ids::UserId;
use base64::Engine;
use chrono::Utc;
use eyre::{Result, eyre};
use muralpay::{MuralError, TokenFeeRequest};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::DBPayoutId,
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
        //     base64::engine::general_purpose::STANDARD
        //         .decode(&payment_statement_doc.body)
        //         .unwrap(),
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
