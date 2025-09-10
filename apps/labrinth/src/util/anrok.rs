use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use thiserror::Error;

pub fn transaction_id_stripe_pi(pi: &stripe::PaymentIntentId) -> String {
    format!("stripe:charge:{pi}")
}

pub fn transaction_id_stripe_pyr(charge: &stripe::RefundId) -> String {
    format!("stripe:refund:{charge}")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceResponse {
    pub tax_amount_to_collect: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub line1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub postal_code: Option<String>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LineItem {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    pub product_external_id: String,
    #[serde_as(as = "DisplayFromStr")]
    pub quantity: u32,
    #[serde(rename = "amount")]
    pub amount_in_smallest_denominations: i64,
}

impl LineItem {
    pub const fn new(
        product_external_id: String,
        amount_in_smallest_denominations: i64,
    ) -> Self {
        Self {
            id: None,
            product_external_id,
            quantity: 1,
            amount_in_smallest_denominations,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFields {
    pub customer_address: Address,
    pub currency_code: String,
    pub accounting_date: DateTime<Utc>,
    pub line_items: Vec<LineItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(flatten)]
    pub fields: TransactionFields,
    pub id: String,
}

#[derive(Error, Debug)]
pub enum AnrokError {
    #[error("Anrok API Error: {0}")]
    Conflict(String),
    #[error("Anrok API Error: Bad request: {0}")]
    BadRequest(String),
    #[error("Rate limit exceeded using Anrok API")]
    RateLimit,
    #[error("Anrok API error: {0}")]
    Other(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    api_key: String,
    api_url: String,
}

impl Client {
    pub fn from_env() -> Result<Self, dotenvy::Error> {
        let api_key = dotenvy::var("ANROK_API_KEY")?;
        let api_url = dotenvy::var("ANROK_API_URL")?
            .trim_start_matches('/')
            .to_owned();

        Ok(Self {
            client: reqwest::Client::builder()
                .user_agent("Modrinth")
                .build()
                .expect("AnrokClient to build"),
            api_key,
            api_url,
        })
    }

    pub async fn create_ephemeral_txn(
        &self,
        body: &TransactionFields,
    ) -> Result<InvoiceResponse, AnrokError> {
        self.make_request(
            Method::POST,
            "/v1/seller/transactions/createEphemeral",
            Some(body),
        )
        .await
    }

    pub async fn create_or_update_txn(
        &self,
        body: &Transaction,
    ) -> Result<InvoiceResponse, AnrokError> {
        self.make_request(
            Method::POST,
            "/v1/seller/transactions/createOrUpdate",
            Some(body),
        )
        .await
    }

    async fn make_request<T: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, AnrokError> {
        #[derive(Deserialize)]
        struct ConflictResponse {
            #[serde(rename = "type")]
            type_: String,
        }

        let mut builder = self
            .client
            .request(method, format!("{}/{}", self.api_url, path))
            .bearer_auth(&self.api_key);

        if let Some(body) = body {
            builder = builder.json(&body);
        }

        let response = builder.send().await?;

        match response.status() {
            StatusCode::CONFLICT => {
                return Err(AnrokError::Conflict(
                    response.json::<ConflictResponse>().await?.type_,
                ));
            }

            StatusCode::BAD_REQUEST => {
                return Err(AnrokError::BadRequest(
                    response.json::<String>().await.unwrap_or_default(),
                ));
            }

            StatusCode::TOO_MANY_REQUESTS => return Err(AnrokError::RateLimit),

            s if !s.is_success() => {
                if let Err(error) = response.error_for_status_ref() {
                    return Err(AnrokError::Other(error));
                }
            }

            _ => {}
        }

        let body = response.json::<R>().await?;
        Ok(body)
    }
}
