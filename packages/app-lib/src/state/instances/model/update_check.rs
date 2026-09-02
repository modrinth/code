use crate::state::ReleaseChannel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentUpdateCheck {
    pub content_entry_id: String,
    pub update_channel: ReleaseChannel,
    pub update_version_id: Option<String>,
    pub checked_at: DateTime<Utc>,
}
