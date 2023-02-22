use crate::routes::ApiError;
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct PayoutsQueue {
    credential: PaypalCredential,
    credential_expires: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
struct PaypalCredential {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Serialize)]
pub struct PayoutItem {
    pub amount: PayoutAmount,
    pub receiver: String,
    pub note: String,
    pub recipient_type: String,
    pub recipient_wallet: String,
    pub sender_item_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PayoutAmount {
    pub currency: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
}

// Batches payouts and handles token refresh
impl PayoutsQueue {
    pub fn new() -> Self {
        PayoutsQueue {
            credential: Default::default(),
            credential_expires: Utc::now() - Duration::days(30),
        }
    }

    pub async fn refresh_token(&mut self) -> Result<(), ApiError> {
        let client = reqwest::Client::new();

        let combined_key = format!(
            "{}:{}",
            dotenvy::var("PAYPAL_CLIENT_ID")?,
            dotenvy::var("PAYPAL_CLIENT_SECRET")?
        );
        let formatted_key = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(combined_key)
        );

        let mut form = HashMap::new();
        form.insert("grant_type", "client_credentials");

        let credential: PaypalCredential = client
            .post(&format!("{}oauth2/token", dotenvy::var("PAYPAL_API_URL")?))
            .header("Accept", "application/json")
            .header("Accept-Language", "en_US")
            .header("Authorization", formatted_key)
            .form(&form)
            .send()
            .await
            .map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal".to_string(),
                )
            })?
            .json()
            .await
            .map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal (deser error)"
                        .to_string(),
                )
            })?;

        self.credential_expires =
            Utc::now() + Duration::seconds(credential.expires_in);
        self.credential = credential;

        Ok(())
    }

    pub async fn send_payout(
        &mut self,
        mut payout: PayoutItem,
    ) -> Result<Decimal, ApiError> {
        if self.credential_expires < Utc::now() {
            self.refresh_token().await.map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal".to_string(),
                )
            })?;
        }

        let wallet = payout.recipient_wallet.clone();

        let fee = if wallet == *"Venmo" {
            Decimal::ONE / Decimal::from(4)
        } else {
            std::cmp::min(
                std::cmp::max(
                    Decimal::ONE / Decimal::from(4),
                    (Decimal::from(2) / Decimal::ONE_HUNDRED)
                        * payout.amount.value,
                ),
                Decimal::from(20),
            )
        };

        payout.amount.value -= fee;
        payout.amount.value = payout.amount.value.round_dp(2);

        if payout.amount.value <= Decimal::ZERO {
            return Err(ApiError::InvalidInput(
                "You do not have enough funds to make this payout!".to_string(),
            ));
        }

        let client = reqwest::Client::new();

        let res = client.post(&format!("{}payments/payouts", dotenvy::var("PAYPAL_API_URL")?))
            .header("Authorization", format!("{} {}", self.credential.token_type, self.credential.access_token))
            .json(&json! ({
                    "sender_batch_header": {
                        "sender_batch_id": format!("{}-payouts", Utc::now().to_rfc3339()),
                        "email_subject": "You have received a payment from Modrinth!",
                        "email_message": "Thank you for creating projects on Modrinth. Please claim this payment within 30 days.",
                    },
                    "items": vec![payout]
                }))
            .send().await.map_err(|_| ApiError::Payments("Error while sending payout to PayPal".to_string()))?;

        if !res.status().is_success() {
            #[derive(Deserialize)]
            struct PayPalError {
                pub body: PayPalErrorBody,
            }

            #[derive(Deserialize)]
            struct PayPalErrorBody {
                pub message: String,
            }

            let body: PayPalError = res.json().await.map_err(|_| {
                ApiError::Payments(
                    "Error while registering payment in PayPal!".to_string(),
                )
            })?;

            return Err(ApiError::Payments(format!(
                "Error while registering payment in PayPal: {}",
                body.body.message
            )));
        } else if wallet != *"Venmo" {
            #[derive(Deserialize)]
            struct PayPalLink {
                href: String,
            }

            #[derive(Deserialize)]
            struct PayoutsResponse {
                pub links: Vec<PayPalLink>,
            }

            #[derive(Deserialize)]
            struct PayoutDataItem {
                payout_item_fee: PayoutAmount,
            }

            #[derive(Deserialize)]
            struct PayoutData {
                pub items: Vec<PayoutDataItem>,
            }

            // Calculate actual fee + refund if we took too big of a fee.
            if let Ok(res) = res.json::<PayoutsResponse>().await {
                if let Some(link) = res.links.first() {
                    if let Ok(res) = client
                        .get(&link.href)
                        .header(
                            "Authorization",
                            format!(
                                "{} {}",
                                self.credential.token_type,
                                self.credential.access_token
                            ),
                        )
                        .send()
                        .await
                    {
                        if let Ok(res) = res.json::<PayoutData>().await {
                            if let Some(data) = res.items.first() {
                                if (fee - data.payout_item_fee.value)
                                    > Decimal::ZERO
                                {
                                    return Ok(fee - data.payout_item_fee.value);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Decimal::ZERO)
    }
}
