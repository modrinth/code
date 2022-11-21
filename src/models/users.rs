use super::ids::Base62Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct UserId(pub u64);

pub const DELETED_USER: UserId = UserId(127155982985829);

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Badges: u64 {
        // 1 << 0 unused - ignore + replace with something later
        const MIDAS = 1 << 0;
        const EARLY_MODPACK_ADOPTER = 1 << 1;
        const EARLY_RESPACK_ADOPTER = 1 << 2;
        const EARLY_PLUGIN_ADOPTER = 1 << 3;
        const ALPHA_TESTER = 1 << 4;
        const CONTRIBUTOR = 1 << 5;
        const TRANSLATOR = 1 << 6;

        const ALL = 0b1111111;
        const NONE = 0b0;
    }
}

impl Default for Badges {
    fn default() -> Badges {
        Badges::NONE
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: UserId,
    pub github_id: Option<u64>,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: Role,
    pub badges: Badges,
    pub payout_data: Option<UserPayoutData>,
    pub has_flame_anvil_key: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserPayoutData {
    pub balance: Decimal,
    pub payout_wallet: Option<RecipientWallet>,
    pub payout_wallet_type: Option<RecipientType>,
    pub payout_address: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecipientType {
    Email,
    Phone,
    UserHandle,
}

impl std::fmt::Display for RecipientType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_str())
    }
}

impl RecipientType {
    pub fn from_string(string: &str) -> RecipientType {
        match string {
            "user_handle" => RecipientType::UserHandle,
            "phone" => RecipientType::Phone,
            _ => RecipientType::Email,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RecipientType::Email => "email",
            RecipientType::Phone => "phone",
            RecipientType::UserHandle => "user_handle",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecipientWallet {
    Venmo,
    Paypal,
}

impl std::fmt::Display for RecipientWallet {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_str())
    }
}

impl RecipientWallet {
    pub fn from_string(string: &str) -> RecipientWallet {
        match string {
            "venmo" => RecipientWallet::Venmo,
            _ => RecipientWallet::Paypal,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RecipientWallet::Paypal => "paypal",
            RecipientWallet::Venmo => "venmo",
        }
    }

    pub fn as_str_api(&self) -> &'static str {
        match self {
            RecipientWallet::Paypal => "PayPal",
            RecipientWallet::Venmo => "Venmo",
        }
    }
}

use crate::database::models::user_item::User as DBUser;
impl From<DBUser> for User {
    fn from(data: DBUser) -> Self {
        Self {
            id: data.id.into(),
            github_id: data.github_id.map(|i| i as u64),
            username: data.username,
            name: data.name,
            email: None,
            avatar_url: data.avatar_url,
            bio: data.bio,
            created: data.created,
            role: Role::from_string(&data.role),
            badges: data.badges,
            payout_data: None,
            has_flame_anvil_key: None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Developer,
    Moderator,
    Admin,
}

impl std::fmt::Display for Role {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_str())
    }
}

impl Role {
    pub fn from_string(string: &str) -> Role {
        match string {
            "admin" => Role::Admin,
            "moderator" => Role::Moderator,
            _ => Role::Developer,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Developer => "developer",
            Role::Moderator => "moderator",
            Role::Admin => "admin",
        }
    }

    pub fn is_mod(&self) -> bool {
        match self {
            Role::Developer => false,
            Role::Moderator | Role::Admin => true,
        }
    }

    pub fn is_admin(&self) -> bool {
        match self {
            Role::Developer | Role::Moderator => false,
            Role::Admin => true,
        }
    }
}
