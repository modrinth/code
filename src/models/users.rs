use super::ids::Base62Id;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct UserId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub github_id: UserId,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: String,
    pub bio: String,
    pub created: chrono::DateTime<chrono::Utc>,
}