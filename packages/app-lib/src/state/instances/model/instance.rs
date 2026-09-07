use crate::state::{
    InstanceInstallStage, LauncherFeatureVersion, ReleaseChannel,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstanceIconBackground {
    Color {
        value: String,
    },
    #[serde(rename = "linear-top-down-gradient")]
    LinearTopDownGradient {
        top_color: String,
        bottom_color: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstanceIconConfig {
    pub background: InstanceIconBackground,
    pub symbol: String,
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct InstanceSyncedOptions {
    pub command_history: bool,
    pub multiplayer_servers: bool,
    pub creative_hotbars: bool,
    pub screenshots: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncedOption {
    CommandHistory,
    MultiplayerServers,
    CreativeHotbars,
    Screenshots,
}

impl SyncedOption {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CommandHistory => "command_history",
            Self::MultiplayerServers => "multiplayer_servers",
            Self::CreativeHotbars => "creative_hotbars",
            Self::Screenshots => "screenshots",
        }
    }

    pub const ALL: [Self; 4] = [
        Self::CommandHistory,
        Self::MultiplayerServers,
        Self::CreativeHotbars,
        Self::Screenshots,
    ];
}

pub type InstanceSyncedOption = SyncedOption;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub path: String,
    pub applied_content_set_id: Option<String>,
    pub install_stage: InstanceInstallStage,
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

pub(crate) fn playtime_to_storage(
    value: u64,
    column: &str,
) -> crate::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::ErrorKind::InputError(format!(
            "Expected {column} to fit in SQLite INTEGER"
        ))
        .into()
    })
}
