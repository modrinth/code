use super::ids::Base62Id;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct UserId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub github_id: u64,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: String,
    pub bio: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub role: Role,
}

#[derive(Serialize, Deserialize)]
pub enum Role {
    Developer,
    Moderator,
    Admin,
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Developer => String::from("developer"),
            Role::Moderator => String::from("moderator"),
            Role::Admin => String::from("admin"),
        }
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
}
