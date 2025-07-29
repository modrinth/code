use super::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserStatus {
    pub user_id: UserId,
    pub profile_name: Option<String>,
    pub last_update: DateTime<Utc>,
}
