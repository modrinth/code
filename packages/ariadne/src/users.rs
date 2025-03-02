use super::ids::Base62Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct UserId(pub u64);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserStatus {
    pub user_id: UserId,
    pub profile_name: Option<String>,
    pub last_update: DateTime<Utc>,
}
