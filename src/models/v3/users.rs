use super::ids::Base62Id;
use crate::{auth::AuthProvider, bitflags_serde_impl};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct UserId(pub u64);

pub const DELETED_USER: UserId = UserId(127155982985829);

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct Badges: u64 {
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

bitflags_serde_impl!(Badges, u64);

impl Default for Badges {
    fn default() -> Badges {
        Badges::NONE
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: Role,
    pub badges: Badges,

    pub auth_providers: Option<Vec<AuthProvider>>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub has_password: Option<bool>,
    pub has_totp: Option<bool>,
    pub payout_data: Option<UserPayoutData>,
    pub stripe_customer_id: Option<String>,

    // DEPRECATED. Always returns None
    pub github_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserPayoutData {
    pub paypal_address: Option<String>,
    pub paypal_country: Option<String>,
    pub venmo_handle: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
}

use crate::database::models::user_item::User as DBUser;
impl From<DBUser> for User {
    fn from(data: DBUser) -> Self {
        Self {
            id: data.id.into(),
            username: data.username,
            name: data.name,
            email: None,
            email_verified: None,
            avatar_url: data.avatar_url,
            bio: data.bio,
            created: data.created,
            role: Role::from_string(&data.role),
            badges: data.badges,
            payout_data: None,
            auth_providers: None,
            has_password: None,
            has_totp: None,
            github_id: None,
            stripe_customer_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
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
