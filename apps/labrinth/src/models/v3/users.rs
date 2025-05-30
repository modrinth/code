use crate::{auth::AuthProvider, bitflags_serde_impl};
use ariadne::ids::UserId;
pub use ariadne::users::UserStatus;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    pub allow_friend_requests: Option<bool>,

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

use crate::database::models::user_item::DBUser;
impl From<DBUser> for User {
    fn from(data: DBUser) -> Self {
        Self {
            id: data.id.into(),
            username: data.username,
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
            allow_friend_requests: None,
        }
    }
}

impl User {
    pub fn from_full(db_user: DBUser) -> Self {
        let mut auth_providers = Vec::new();

        if db_user.github_id.is_some() {
            auth_providers.push(AuthProvider::GitHub)
        }
        if db_user.gitlab_id.is_some() {
            auth_providers.push(AuthProvider::GitLab)
        }
        if db_user.discord_id.is_some() {
            auth_providers.push(AuthProvider::Discord)
        }
        if db_user.google_id.is_some() {
            auth_providers.push(AuthProvider::Google)
        }
        if db_user.microsoft_id.is_some() {
            auth_providers.push(AuthProvider::Microsoft)
        }
        if db_user.steam_id.is_some() {
            auth_providers.push(AuthProvider::Steam)
        }
        if db_user.paypal_id.is_some() {
            auth_providers.push(AuthProvider::PayPal)
        }

        Self {
            id: UserId::from(db_user.id),
            username: db_user.username,
            email: db_user.email,
            email_verified: Some(db_user.email_verified),
            avatar_url: db_user.avatar_url,
            bio: db_user.bio,
            created: db_user.created,
            role: Role::from_string(&db_user.role),
            badges: db_user.badges,
            auth_providers: Some(auth_providers),
            has_password: Some(db_user.password.is_some()),
            has_totp: Some(db_user.totp_secret.is_some()),
            github_id: None,
            payout_data: Some(UserPayoutData {
                paypal_address: db_user.paypal_email,
                paypal_country: db_user.paypal_country,
                venmo_handle: db_user.venmo_handle,
                balance: Decimal::ZERO,
            }),
            stripe_customer_id: db_user.stripe_customer_id,
            allow_friend_requests: Some(db_user.allow_friend_requests),
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

#[derive(Serialize, Deserialize)]
pub struct UserFriend {
    // The user who accepted the friend request
    pub id: UserId,
    /// THe user who sent the friend request
    pub friend_id: UserId,
    pub accepted: bool,
    pub created: DateTime<Utc>,
}

impl UserFriend {
    pub fn from(data: crate::database::models::friend_item::DBFriend) -> Self {
        Self {
            id: data.friend_id.into(),
            friend_id: data.user_id.into(),
            accepted: data.accepted,
            created: data.created,
        }
    }
}
