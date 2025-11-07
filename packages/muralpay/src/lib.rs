#![doc = include_str!("../README.md")]

mod account;
mod counterparty;
mod error;
mod organization;
mod payout;
mod payout_method;
mod serde_iso3166;
mod util;

pub use {
    account::*, counterparty::*, error::*, organization::*, payout::*,
    payout_method::*,
};

use rust_decimal::Decimal;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, str::FromStr};
use uuid::Uuid;

pub const API_URL: &str = "https://api.muralpay.com";
pub const SANDBOX_API_URL: &str = "https://api-staging.muralpay.com";

/// Default token symbol for [`TokenAmount::token_symbol`] values.
pub const USDC: &str = "USDC";

#[derive(Debug)]
pub struct MuralPay {
    pub http: reqwest::Client,
    pub api_url: String,
    pub api_key: SecretString,
    pub transfer_api_key: Option<SecretString>,
}

impl MuralPay {
    pub fn new(
        api_url: impl Into<String>,
        api_key: impl Into<SecretString>,
        transfer_api_key: Option<impl Into<SecretString>>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url: api_url.into(),
            api_key: api_key.into(),
            transfer_api_key: transfer_api_key.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Blockchain {
    Ethereum,
    Polygon,
    Base,
    Celo,
}

crate::util::display_as_serialize!(Blockchain);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

crate::util::display_as_serialize!(CurrencyCode);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum FiatAccountType {
    Checking,
    Savings,
}

crate::util::display_as_serialize!(FiatAccountType);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

crate::util::display_as_serialize!(FiatAndRailCode);

impl FromStr for FiatAndRailCode {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_owned()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WalletDetails {
    pub blockchain: Blockchain,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    #[serde(with = "rust_decimal::serde::float")]
    pub token_amount: Decimal,
    pub token_symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct FiatAmount {
    #[serde(with = "rust_decimal::serde::float")]
    pub fiat_amount: Decimal,
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse<Id, T> {
    pub total: u64,
    pub next_id: Option<Id>,
    pub results: Vec<T>,
}
