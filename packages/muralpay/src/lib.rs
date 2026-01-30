#![doc = include_str!("../README.md")]

#[cfg(feature = "client")]
macro_rules! maybe_mock {
    ($self:expr, $fn:ident ( $($args:expr),* $(,)? )) => {
        #[cfg(feature = "mock")]
        if let Some(mock) = &*($self).mock.load() {
            return (mock.$fn)($($args),*);
        }
    };
}

mod account;
mod counterparty;
mod organization;
mod payout;
mod payout_method;
mod serde_iso3166;
mod transaction;
mod util;

pub use {
    account::*, counterparty::*, organization::*, payout::*, payout_method::*,
    transaction::*,
};
use {
    rust_decimal::Decimal,
    serde::{Deserialize, Serialize},
    std::{ops::Deref, str::FromStr},
    uuid::Uuid,
};

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

pub const API_URL: &str = "https://api.muralpay.com";
pub const SANDBOX_API_URL: &str = "https://api-staging.muralpay.com";

/// Default token symbol for [`TokenAmount::token_symbol`] values.
pub const USDC: &str = "USDC";

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter,
)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WalletDetails {
    pub blockchain: Blockchain,
    pub wallet_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    #[serde(with = "rust_decimal::serde::float")]
    pub token_amount: Decimal,
    pub token_symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse<Id, T> {
    pub total: u64,
    pub next_id: Option<Id>,
    pub results: Vec<T>,
}
