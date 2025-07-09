use crate::{
    auth::AuthProvider,
    models::users::{Badges, Role, UserPayoutData},
};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LegacyUser {
    pub id: UserId,
    pub username: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: Role,
    pub badges: Badges,

    pub auth_providers: Option<Vec<AuthProvider>>, // this was changed in v3, but not changes ones we want to keep out of v2
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub has_password: Option<bool>,
    pub has_totp: Option<bool>,
    pub payout_data: Option<UserPayoutData>, // this was changed in v3, but not ones we want to keep out of v2

    // DEPRECATED. Always returns None
    pub github_id: Option<u64>,
}

impl From<crate::models::v3::users::User> for LegacyUser {
    fn from(data: crate::models::v3::users::User) -> Self {
        Self {
            id: data.id,
            username: data.username,
            name: None,
            email: data.email,
            email_verified: data.email_verified,
            avatar_url: data.avatar_url,
            bio: data.bio,
            created: data.created,
            role: data.role,
            badges: data.badges,
            payout_data: data.payout_data,
            auth_providers: data.auth_providers,
            has_password: data.has_password,
            has_totp: data.has_totp,
            github_id: data.github_id,
        }
    }
}
