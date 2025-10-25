use ariadne::ids::UserId;
use eyre::Result;
use muralpay::{MuralError, TokenFeeRequest};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    queue::payouts::PayoutsQueue, routes::ApiError, util::error::Context,
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
                    token_symbol: "USDC".into(),
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
        user_id: UserId,
        amount: muralpay::TokenAmount,
        payout_details: MuralPayoutRequest,
        recipient_info: muralpay::PayoutRecipientInfo,
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

        let payout = muralpay::CreatePayout {
            amount,
            payout_details,
            recipient_info,
            supporting_details: None,
            // TODO
            // Some(muralpay::SupportingDetails {
            //     supporting_document: Some(todo!()),
            //     payout_purpose: Some(muralpay::PayoutPurpose::VendorPayment),
            // }),
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
                err => ApiError::Internal(err.into()),
            })?;
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
}
