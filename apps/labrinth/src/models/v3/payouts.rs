use std::{cmp, collections::HashMap, fmt};

use crate::{models::ids::PayoutId, queue::payouts::mural::MuralPayoutRequest};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Clone)]
pub struct Payout {
    pub id: PayoutId,
    pub user_id: UserId,
    pub status: PayoutStatus,
    pub created: DateTime<Utc>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,

    #[serde(with = "rust_decimal::serde::float_option")]
    pub fee: Option<Decimal>,
    pub method: Option<PayoutMethodType>,
    /// Platform-dependent identifier for the submethod.
    ///
    /// See [`crate::routes::v3::payouts::TransactionItem::Withdrawal::method_id`].
    pub method_id: Option<String>,
    /// Address this payout was sent to: ex: email, paypal email, venmo handle.
    pub method_address: Option<String>,
    /// Platform-provided opaque identifier for the transaction linked to this payout.
    ///
    /// - Tremendous: reward ID
    /// - Mural: payout request UUID
    /// - PayPal/Venmo: transaction ID
    pub platform_id: Option<String>,
}

impl Payout {
    pub fn from(data: crate::database::models::payout_item::DBPayout) -> Self {
        Self {
            id: data.id.into(),
            user_id: data.user_id.into(),
            status: data.status,
            created: data.created,
            amount: data.amount,
            fee: data.fee,
            method: data.method,
            method_id: data.method_id,
            method_address: data.method_address,
            platform_id: data.platform_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "method", rename_all = "lowercase")]
#[expect(
    clippy::large_enum_variant,
    reason = "acceptable since values of this type are not moved much"
)]
pub enum PayoutMethodRequest {
    Venmo,
    PayPal,
    Tremendous { method_details: TremendousDetails },
    MuralPay { method_details: MuralPayDetails },
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum PayoutMethodType {
    Venmo,
    PayPal,
    Tremendous,
    MuralPay,
}

impl PayoutMethodRequest {
    pub fn method_type(&self) -> PayoutMethodType {
        match self {
            Self::Venmo => PayoutMethodType::Venmo,
            Self::PayPal => PayoutMethodType::PayPal,
            Self::Tremendous { .. } => PayoutMethodType::Tremendous,
            Self::MuralPay { .. } => PayoutMethodType::MuralPay,
        }
    }
}

impl std::fmt::Display for PayoutMethodType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TremendousDetails {
    pub delivery_email: String,
    #[schema(inline)]
    pub currency: Option<TremendousCurrency>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TremendousCurrency {
    Usd,
    Gbp,
    Cad,
    Eur,
    Aud,
    Chf,
    Czk,
    Dkk,
    Mxn,
    Nok,
    Nzd,
    Pln,
    Sek,
    Sgd,
}

impl fmt::Display for TremendousCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        let s = s.as_str().ok_or(fmt::Error)?;
        write!(f, "{s}")
    }
}

#[derive(Debug, Deserialize)]
pub struct TremendousForexResponse {
    pub forex: HashMap<String, Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MuralPayDetails {
    pub payout_details: MuralPayoutRequest,
    pub recipient_info: muralpay::CreatePayoutRecipientInfo,
}

impl PayoutMethodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayoutMethodType::Venmo => "venmo",
            PayoutMethodType::PayPal => "paypal",
            PayoutMethodType::Tremendous => "tremendous",
            PayoutMethodType::MuralPay => "muralpay",
        }
    }

    pub fn from_string(string: &str) -> Option<PayoutMethodType> {
        match string {
            "venmo" => Some(PayoutMethodType::Venmo),
            "paypal" => Some(PayoutMethodType::PayPal),
            "tremendous" => Some(PayoutMethodType::Tremendous),
            "muralpay" => Some(PayoutMethodType::MuralPay),
            _ => None,
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    utoipa::ToSchema,
    sqlx::Type,
)]
#[serde(rename_all = "kebab-case")]
#[sqlx(rename_all = "kebab-case")]
pub enum PayoutStatus {
    Success,
    InTransit,
    Cancelled,
    Cancelling,
    Failed,
    Unknown,
}

impl std::fmt::Display for PayoutStatus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl PayoutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayoutStatus::Success => "success",
            PayoutStatus::InTransit => "in-transit",
            PayoutStatus::Cancelled => "cancelled",
            PayoutStatus::Cancelling => "cancelling",
            PayoutStatus::Failed => "failed",
            PayoutStatus::Unknown => "unknown",
        }
    }

    pub fn from_string(string: &str) -> PayoutStatus {
        match string {
            "success" => PayoutStatus::Success,
            "in-transit" => PayoutStatus::InTransit,
            "cancelled" => PayoutStatus::Cancelled,
            "cancelling" => PayoutStatus::Cancelling,
            "failed" => PayoutStatus::Failed,
            _ => PayoutStatus::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PayoutMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: PayoutMethodType,
    pub name: String,
    pub category: Option<String>,
    #[serde(skip_serializing)]
    pub supported_countries: Vec<String>,
    pub image_url: Option<String>,
    pub image_logo_url: Option<String>,
    pub interval: PayoutInterval,
    pub fee: PayoutMethodFee,
    pub currency_code: Option<String>,
    /// USD to the given `currency_code`.
    #[serde(with = "rust_decimal::serde::float_option")]
    pub exchange_rate: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PayoutMethodFee {
    #[serde(with = "rust_decimal::serde::float")]
    pub percentage: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub min: Decimal,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub max: Option<Decimal>,
}

impl PayoutMethodFee {
    pub fn compute_fee(&self, value: impl Into<Decimal>) -> Decimal {
        cmp::min(
            cmp::max(self.min, self.percentage * value.into()),
            self.max.unwrap_or(Decimal::MAX),
        )
    }
}

#[derive(Clone)]
pub struct PayoutDecimal(pub Decimal);

impl Serialize for PayoutDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        rust_decimal::serde::float::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for PayoutDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let decimal = rust_decimal::serde::float::deserialize(deserializer)?;
        Ok(PayoutDecimal(decimal))
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PayoutInterval {
    Standard {
        #[serde(with = "rust_decimal::serde::float")]
        min: Decimal,
        #[serde(with = "rust_decimal::serde::float")]
        max: Decimal,
    },
    Fixed {
        values: Vec<PayoutDecimal>,
    },
}
