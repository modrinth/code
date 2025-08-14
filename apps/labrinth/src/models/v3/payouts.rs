use crate::models::ids::PayoutId;
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
    /// the address this payout was sent to: ex: email, paypal email, venmo handle
    pub method_address: Option<String>,
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
            method_address: data.method_address,
            platform_id: data.platform_id,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PayoutMethodType {
    Venmo,
    PayPal,
    Tremendous,
    Unknown,
}

impl std::fmt::Display for PayoutMethodType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl PayoutMethodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayoutMethodType::Venmo => "venmo",
            PayoutMethodType::PayPal => "paypal",
            PayoutMethodType::Tremendous => "tremendous",
            PayoutMethodType::Unknown => "unknown",
        }
    }

    pub fn from_string(string: &str) -> PayoutMethodType {
        match string {
            "venmo" => PayoutMethodType::Venmo,
            "paypal" => PayoutMethodType::PayPal,
            "tremendous" => PayoutMethodType::Tremendous,
            _ => PayoutMethodType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
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
    pub supported_countries: Vec<String>,
    pub image_url: Option<String>,
    pub interval: PayoutInterval,
    pub fee: PayoutMethodFee,
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
