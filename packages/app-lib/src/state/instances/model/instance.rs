use crate::state::{
    LauncherFeatureVersion, ProfileInstallStage, ReleaseChannel,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub path: String,
    pub applied_content_set_id: Option<String>,
    pub install_stage: ProfileInstallStage,
    pub launcher_feature_version: LauncherFeatureVersion,
    pub update_channel: ReleaseChannel,
    pub name: String,
    pub icon_path: Option<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    pub submitted_time_played: u64,
    pub recent_time_played: u64,
}
