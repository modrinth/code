use crate::models::users::{RecipientType, RecipientWallet};
use crate::routes::ApiError;
use chrono::{DateTime, Duration, Utc};
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
    pub recipient_type: RecipientType,
    pub recipient_wallet: RecipientWallet,
    pub sender_item_id: String,
}

#[derive(Serialize)]
pub struct PayoutAmount {
    pub currency: String,
    pub value: String,
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
        let formatted_key = format!("Basic {}", base64::encode(combined_key));

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
        payout: PayoutItem,
    ) -> Result<(), ApiError> {
        if self.credential_expires < Utc::now() {
            self.refresh_token().await.map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal".to_string(),
                )
            })?;
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
        }

        Ok(())
    }
}
