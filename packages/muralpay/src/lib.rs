#![doc = include_str!("../README.md")]

mod account;
mod organization;
mod payout;
use std::ops::Deref;

pub use {account::*, organization::*, payout::*};

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const API_URL: &str = "https://api.muralpay.com";
pub const SANDBOX_API_URL: &str = "https://api-staging.muralpay.com";

#[derive(Debug)]
pub struct MuralPay {
    pub http: reqwest::Client,
    pub api_url: String,
    pub api_key: SecretString,
}

impl MuralPay {
    pub fn new(
        api_url: impl Into<String>,
        api_key: impl Into<SecretString>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url: api_url.into(),
            api_key: api_key.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTime(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Blockchain {
    Ethereum,
    Polygon,
    Base,
    Celo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CurrencyCode {
    Usd,
    Cop,
    Ars,
    Eur,
    Mxn,
    Brl,
    Clp,
    Pen,
    Bob,
    Crc,
    Zar,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FiatAndRailCode {
    Usd,
    Cop,
    Ars,
    Eur,
    Mxn,
    Brl,
    Clp,
    Pen,
    Bob,
    Crc,
    Zar,
    UsdPeru,
    UsdChina,
    UsdPanama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDetails {
    pub blockchain: Blockchain,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub token_amount: u64,
    pub token_symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAmount {
    pub fiat_amount: u64,
    pub fiat_currency_code: CurrencyCode,
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams<Id> {
    pub limit: Option<u64>,
    pub next_id: Option<Id>,
}

impl<Id: Deref<Target = Uuid> + Clone> SearchParams<Id> {
    pub fn to_query(&self) -> Vec<(&'static str, String)> {
        [
            self.limit.map(|limit| ("limit", limit.to_string())),
            self.next_id
                .clone()
                .map(|id| ("nextId", id.hyphenated().to_string())),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse<Id, T> {
    pub total: u64,
    pub next_id: Option<Id>,
    pub results: Vec<T>,
}
