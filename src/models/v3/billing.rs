use crate::models::ids::Base62Id;
use crate::models::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct ProductId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub metadata: ProductMetadata,
    pub prices: Vec<ProductPrice>,
    pub unitary: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ProductMetadata {
    Midas,
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct ProductPriceId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct ProductPrice {
    pub id: ProductPriceId,
    pub product_id: ProductId,
    pub prices: Price,
    pub currency_code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Price {
    OneTime {
        price: i32,
    },
    Recurring {
        intervals: HashMap<PriceDuration, i32>,
    },
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PriceDuration {
    Monthly,
    Yearly,
}

impl PriceDuration {
    pub fn from_string(string: &str) -> PriceDuration {
        match string {
            "monthly" => PriceDuration::Monthly,
            "yearly" => PriceDuration::Yearly,
            _ => PriceDuration::Monthly,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PriceDuration::Monthly => "monthly",
            PriceDuration::Yearly => "yearly",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct UserSubscriptionId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct UserSubscription {
    pub id: UserSubscriptionId,
    pub user_id: UserId,
    pub price_id: ProductPriceId,
    pub interval: PriceDuration,
    pub status: SubscriptionStatus,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub last_charge: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SubscriptionStatus {
    Active,
    PaymentProcessing,
    PaymentFailed,
    Cancelled,
}

impl SubscriptionStatus {
    pub fn from_string(string: &str) -> SubscriptionStatus {
        match string {
            "active" => SubscriptionStatus::Active,
            "payment-processing" => SubscriptionStatus::PaymentProcessing,
            "payment-failed" => SubscriptionStatus::PaymentFailed,
            "cancelled" => SubscriptionStatus::Cancelled,
            _ => SubscriptionStatus::Cancelled,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::PaymentProcessing => "payment-processing",
            SubscriptionStatus::PaymentFailed => "payment-failed",
            SubscriptionStatus::Cancelled => "cancelled",
        }
    }
}
