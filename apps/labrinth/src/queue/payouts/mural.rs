use chrono::Utc;
use eyre::{Result, eyre};
use modrinth_util::decimal::Decimal2dp;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::DBPayoutId,
    queue::payouts::PayoutsQueue,
    routes::{ApiError, internal::gotenberg::GotenbergDocument},
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
    pub async fn create_mural_payment_statement_doc(
        &self,
        payout_id: DBPayoutId,
        net_usd: Decimal2dp,
        total_fee_usd: Decimal2dp,
        recipient_info: &muralpay::CreatePayoutRecipientInfo,
        gotenberg: &GotenbergClient,
    ) -> Result<GotenbergDocument, ApiError> {
        let gross_usd = net_usd + total_fee_usd;

        let recipient_address = recipient_info.physical_address();
        let recipient_email = recipient_info.email().to_string();
        let gross_cents = gross_usd.get() * Decimal::from(100);
        let net_cents = net_usd.get() * Decimal::from(100);
        let fees_cents = total_fee_usd.get() * Decimal::from(100);
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
            gross_amount_cents: gross_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("gross amount of cents `{gross_cents}` cannot be expressed as an `i64`"))?,
            net_amount_cents: net_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("net amount of cents `{net_cents}` cannot be expressed as an `i64`"))?,
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

        Ok(payment_statement_doc)
    }

    pub async fn cancel_mural_payout_request(
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
