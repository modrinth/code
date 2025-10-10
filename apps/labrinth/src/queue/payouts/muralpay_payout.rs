use ariadne::ids::UserId;
use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{queue::payouts::PayoutsQueue, util::error::Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub async fn create_muralpay_payout_request(
        &self,
        user_id: UserId,
        amount: muralpay::TokenAmount,
        payout_details: MuralPayoutRequest,
        recipient_info: muralpay::PayoutRecipientInfo,
    ) -> Result<muralpay::PayoutRequest> {
        let muralpay = self.muralpay.read().await;
        let muralpay = muralpay
            .as_ref()
            .wrap_err("Mural Pay client not available")?;

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
            recipient_info: recipient_info.into(),
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
            .await?;
        Ok(payout_request)
    }

    pub async fn cancel_muralpay_payout_request(
        &self,
        id: muralpay::PayoutRequestId,
    ) -> Result<()> {
        let muralpay = self.muralpay.read().await;
        let muralpay = muralpay
            .as_ref()
            .wrap_err("Mural Pay client not available")?;

        muralpay.client.cancel_payout_request(id).await?;
        Ok(())
    }
}
