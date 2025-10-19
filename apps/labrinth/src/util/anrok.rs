use chrono::{DateTime, Utc};
use rand::Rng;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use thiserror::Error;
use tracing::trace;

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
    pub version: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmptyResponse {}

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

impl Address {
    pub fn from_stripe_address(address: &stripe::Address) -> Self {
        Self {
            country: address.country.clone(),
            line1: address.line1.clone(),
            city: address.city.clone(),
            region: address.state.clone(),
            postal_code: address.postal_code.clone(),
        }
    }
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
    pub is_tax_included_in_amount: bool,
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
            is_tax_included_in_amount: false,
        }
    }

    pub const fn new_including_tax_amount(
        product_external_id: String,
        amount_in_smallest_denominations: i64,
    ) -> Self {
        Self {
            id: None,
            product_external_id,
            quantity: 1,
            amount_in_smallest_denominations,
            is_tax_included_in_amount: true,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub enum AccountingTimeZone {
    #[default]
    #[serde(rename = "UTC")]
    Utc,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFields {
    pub customer_address: Address,
    pub currency_code: String,
    pub accounting_time: DateTime<Utc>,
    pub accounting_time_zone: AccountingTimeZone,
    pub line_items: Vec<LineItem>,
    pub customer_name: Option<String>,
    pub customer_id: Option<String>,
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

impl AnrokError {
    pub fn is_conflict_and<F>(&self, pred: F) -> bool
    where
        F: FnOnce(&str) -> bool,
    {
        if let AnrokError::Conflict(message) = self {
            pred(message)
        } else {
            false
        }
    }
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

    pub async fn negate_or_create_partial_negation(
        &self,
        original_txn_anrok_id: String,
        original_txn_version: i32,
        original_txn_tax_amount_with_tax: i64,
        body: &Transaction,
    ) -> Result<(), AnrokError> {
        let refund_amount = body
            .fields
            .line_items
            .iter()
            .map(|l| l.amount_in_smallest_denominations)
            .sum::<i64>();

        if -refund_amount == original_txn_tax_amount_with_tax {
            self.create_full_negation(
                original_txn_anrok_id,
                original_txn_version,
                body.id.clone(),
            )
            .await?;
        } else {
            self.create_or_update_txn(body).await?;
        }

        Ok(())
    }

    pub async fn create_full_negation(
        &self,
        original_txn_anrok_id: String,
        original_txn_version: i32,
        new_txn_id: String,
    ) -> Result<EmptyResponse, AnrokError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct NegationBody {
            original_transaction_id: String,
            new_transaction_id: String,
            original_transaction_expected_version: i32,
        }

        self.make_request(
            Method::POST,
            "/v1/seller/transactions/createNegation",
            Some(&NegationBody {
                original_transaction_id: original_txn_anrok_id,
                new_transaction_id: new_txn_id,
                original_transaction_expected_version: original_txn_version,
            }),
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

    pub async fn void_txn(
        &self,
        id: String,
        version: i32,
    ) -> Result<EmptyResponse, AnrokError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            transaction_expected_version: i32,
        }

        self.make_request(
            Method::POST,
            &format!("/v1/seller/transactions/id:{id}/void"),
            Some(&Body {
                transaction_expected_version: version,
            }),
        )
        .await
    }

    async fn make_request<T: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, AnrokError> {
        let mut n = 0u64;

        loop {
            if n >= 3 {
                return Err(AnrokError::RateLimit);
            }

            match self.make_request_inner(method.clone(), path, body).await {
                Err(AnrokError::RateLimit) => {
                    n += 1;
                    // 1000 + ~500, 2000 + ~1000, 5000 + ~2500
                    let base = (n - 1).pow(2) * 1000 + 1000;
                    let random = rand::thread_rng().gen_range(0..(base / 2));
                    tokio::time::sleep(std::time::Duration::from_millis(
                        base + random,
                    ))
                    .await;
                }

                other => return other,
            }
        }
    }

    async fn make_request_inner<T: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, AnrokError> {
        let then = std::time::Instant::now();

        #[derive(Deserialize)]
        struct ConflictResponse {
            #[serde(rename = "type")]
            type_: String,
        }

        let mut builder = self
            .client
            .request(method.clone(), format!("{}/{}", self.api_url, path))
            .bearer_auth(&self.api_key);

        if let Some(body) = body {
            builder = builder.json(&body);
        }

        let response = builder.send().await?;

        trace!(
            http.status = %response.status().as_u16(),
            http.method = %method,
            http.path = %path,
            duration = format!("{}ms", then.elapsed().as_millis()),
            "Received Anrok response",
        );

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
