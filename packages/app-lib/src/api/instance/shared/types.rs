use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedInstanceUsers {
    pub user_ids: Vec<String>,
    #[serde(default)]
    pub users: Vec<SharedInstanceUser>,
    #[serde(default)]
    pub tokens: i32,
}

impl SharedInstanceUsers {
    pub(super) fn empty() -> Self {
        Self {
            user_ids: Vec::new(),
            users: Vec::new(),
            tokens: 0,
        }
    }

    pub(super) fn from_users(
        users: Vec<SharedInstanceUser>,
        tokens: i32,
    ) -> Self {
        let user_ids = users.iter().map(|user| user.id.clone()).collect();

        Self {
            user_ids,
            users,
            tokens,
        }
    }

    pub(super) fn from_user_ids(user_ids: Vec<String>) -> Self {
        let users = user_ids
            .iter()
            .map(|user_id| SharedInstanceUser {
                id: user_id.clone(),
                joined_at: None,
                join_type: SharedInstanceJoinType::Invite,
                last_played: None,
            })
            .collect();

        Self {
            user_ids,
            users,
            tokens: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedInstanceUser {
    pub id: String,
    pub joined_at: Option<DateTime<Utc>>,
    pub join_type: SharedInstanceJoinType,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SharedInstanceJoinType {
    Owner,
    Invite,
    Link,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstanceInstallPreview {
    pub shared_instance_id: String,
    pub version: i32,
    pub name: String,
    pub icon_url: Option<String>,
    pub game_version: String,
    pub loader: ModLoader,
    pub mod_count: usize,
    pub external_file_count: usize,
    pub modpack_version_id: Option<String>,
    pub content_version_ids: Vec<String>,
    pub external_files: Vec<SharedInstanceExternalFilePreview>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstanceExternalFilePreview {
    pub file_name: String,
    pub file_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstanceUpdatePreview {
    pub shared_instance_id: String,
    pub current_version: Option<i32>,
    pub latest_version: i32,
    pub update_available: bool,
    pub diffs: Vec<SharedInstanceUpdateDiff>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstancePublishPreview {
    pub shared_instance_id: String,
    pub latest_version: i32,
    pub diffs: Vec<SharedInstanceUpdateDiff>,
    pub config_files: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstanceInviteLink {
    pub invite_id: String,
    pub expires_at: DateTime<Utc>,
    pub max_uses: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstanceInviteInstallPreview {
    pub shared_instance_id: String,
    pub manager_id: Option<String>,
    pub server_manager_name: Option<String>,
    pub server_manager_icon_url: Option<String>,
    pub instance_icon_url: Option<String>,
    pub preview: SharedInstanceInstallPreview,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedInstanceUpdateDiff {
    #[serde(rename = "type")]
    pub type_: SharedInstanceUpdateDiffType,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub file_name: Option<String>,
    pub current_version_name: Option<String>,
    pub new_version_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_file_count: Option<usize>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedInstanceUpdateDiffType {
    Added,
    Removed,
    Updated,
    ModpackLinked,
    ModpackUpdated,
    ModpackUnlinked,
    GameVersionUpdated,
    LoaderUpdated,
    ConfigFilesUpdated,
}

pub(super) fn is_false(value: &bool) -> bool {
    !*value
}
