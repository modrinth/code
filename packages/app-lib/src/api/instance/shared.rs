use super::content_set_diff::{
    ContentSetDiffEntry, ContentSetDiffKind, ContentSetDiffOptions,
    ContentSetSnapshot, ContentSetSnapshotVersion, diff_content_sets,
};
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::install::{
    InstallJobSnapshot, SharedInstanceExternalFileData,
    SharedInstanceInstallData,
};
use crate::state::instances::{InstanceLink, SharedInstanceAttachment};
use crate::state::{
    AppliedContentSetPatch, CacheBehaviour, CachedEntry, ContentSetSyncStatus,
    ContentSourceKind, EditInstance, ModLoader, ModrinthCredentials,
    ProjectType, SharedInstanceRole, State,
};
use crate::util::fetch::{INSECURE_REQWEST_CLIENT, REQWEST_CLIENT};
use crate::SharedInstanceUnavailableReason;
use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

const SHARED_INSTANCE_INVITE_MAX_AGE_SECONDS: i32 = 604800;
const SHARED_INSTANCE_INVITE_MAX_USES: i32 = 10;

#[derive(Clone, Copy, Debug)]
enum SharedInstancesRequestAuth {
    ModrinthSession,
    None,
}

impl SharedInstancesRequestAuth {
    fn label(self) -> &'static str {
        match self {
            Self::ModrinthSession => "modrinth_session",
            Self::None => "none",
        }
    }
}

impl SharedInstanceUnavailableReason {
    fn from_status(status: StatusCode) -> Option<Self> {
        match status {
            StatusCode::NOT_FOUND => Some(Self::Deleted),
            StatusCode::UNAUTHORIZED => Some(Self::AccessRevoked),
            _ => None,
        }
    }

}

#[derive(Debug)]
enum SharedInstanceRemoteResponse<T> {
    Available(T),
    Unavailable(SharedInstanceUnavailableReason),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedInstanceUsers {
    pub user_ids: Vec<String>,
    #[serde(default)]
    pub users: Vec<SharedInstanceUser>,
    #[serde(default)]
    pub tokens: i32,
}

impl SharedInstanceUsers {
    fn empty() -> Self {
        Self {
            user_ids: Vec::new(),
            users: Vec::new(),
            tokens: 0,
        }
    }

    fn from_users(users: Vec<SharedInstanceUser>, tokens: i32) -> Self {
        let user_ids = users.iter().map(|user| user.id.clone()).collect();

        Self {
            user_ids,
            users,
            tokens,
        }
    }

    fn from_user_ids(user_ids: Vec<String>) -> Self {
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

    fn include_pending_invites(&mut self, user_ids: Vec<String>) {
        for user_id in user_ids {
            if self.user_ids.iter().any(|id| id == &user_id) {
                continue;
            }

            self.user_ids.push(user_id.clone());
            self.users.push(SharedInstanceUser {
                id: user_id,
                joined_at: None,
                join_type: SharedInstanceJoinType::Invite,
                last_played: None,
            });
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
}

#[derive(Clone, Debug)]
struct ExternalFileCandidate {
    file_name: String,
    file_type: String,
    file_path: String,
}

#[derive(Clone, Debug, Serialize)]
struct ExternalFileData {
    file_name: String,
    file_type: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CreateInstanceResponse {
    #[serde(alias = "instance_id")]
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CreateInstanceInviteResponse {
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct InstanceInviteInfoResponse {
    instance_id: String,
    instance_name: String,
    #[serde(default)]
    managers: Vec<InstanceInviteManagerResponse>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InstanceInviteManagerResponse {
    User { id: String },
    Server { name: String, icon: Option<String> },
}

#[derive(Clone, Debug, Deserialize)]
struct InstanceVersionResponse {
    version: i32,
    #[serde(default)]
    modrinth_ids: Vec<String>,
    ready: bool,
    #[serde(default)]
    external_files: Vec<ExternalFileResponse>,
    #[serde(default)]
    modpack_id: Option<String>,
    game_version: String,
    loader: ModLoader,
    loader_version: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ExternalFileResponse {
    file_name: String,
    file_type: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize)]
struct RemoteSharedInstanceUsers {
    #[serde(default)]
    users: Vec<SharedInstanceUser>,
    #[serde(default)]
    tokens: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum RemoteSharedInstanceUsersResponse {
    Current(RemoteSharedInstanceUsers),
    Legacy(Vec<String>),
}

impl RemoteSharedInstanceUsersResponse {
    fn into_shared_users(self) -> SharedInstanceUsers {
        match self {
            Self::Current(response) => {
                SharedInstanceUsers::from_users(response.users, response.tokens)
            }
            Self::Legacy(user_ids) => {
                SharedInstanceUsers::from_user_ids(user_ids)
            }
        }
    }
}

#[tracing::instrument]
pub async fn get_shared_instance_users(
    instance_id: &str,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await? else {
        return Ok(SharedInstanceUsers::empty());
    };

    get_remote_users(&attachment.id, &state).await
}

#[tracing::instrument]
pub async fn invite_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let (metadata, attachment) =
        shared_instance_for_invites(instance_id, user_ids.len(), &state)
            .await?;

    ensure_owner(&attachment)?;
    if !user_ids.is_empty() {
        ensure_ready_remote_version_for_invite(
            instance_id,
            &attachment,
            &state,
        )
        .await?;
        update_remote_instance(
            &attachment.id,
            shared_instance_name(metadata.instance.name.clone()),
            &state,
        )
        .await?;
        tracing::info!(
            instance_id,
            shared_instance_id = %attachment.id,
            user_count = user_ids.len(),
            "Adding users to shared instance"
        );
        add_remote_users(&attachment.id, user_ids.clone(), &state).await?;
    }
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    let mut users = get_remote_users(&attachment.id, &state).await?;
    users.include_pending_invites(user_ids);
    Ok(users)
}

#[tracing::instrument]
pub async fn create_shared_instance_invite_link(
    instance_id: &str,
    max_age_seconds: Option<i32>,
    max_uses: Option<i32>,
    replace_invite_id: Option<String>,
) -> crate::Result<SharedInstanceInviteLink> {
    let state = State::get().await?;
    let (metadata, attachment) =
        shared_instance_for_invites(instance_id, 0, &state).await?;
    ensure_owner(&attachment)?;
    ensure_ready_remote_version_for_invite(instance_id, &attachment, &state)
        .await?;
    update_remote_instance(
        &attachment.id,
        shared_instance_name(metadata.instance.name),
        &state,
    )
    .await?;

    let max_age_seconds =
        max_age_seconds.unwrap_or(SHARED_INSTANCE_INVITE_MAX_AGE_SECONDS);
    let max_uses = max_uses.unwrap_or(SHARED_INSTANCE_INVITE_MAX_USES);
    if max_age_seconds <= 0
        || max_age_seconds > SHARED_INSTANCE_INVITE_MAX_AGE_SECONDS
    {
        return Err(crate::ErrorKind::InputError(
            "Invite expiry must be between now and seven days".to_string(),
        )
        .into());
    }
    if max_uses <= 0 {
        return Err(crate::ErrorKind::InputError(
            "Invite max uses must be greater than zero".to_string(),
        )
        .into());
    }

    if let Some(invite_id) = replace_invite_id {
        request_empty(
            "delete_instance_invite",
            Method::DELETE,
            &format!("/instances/{}/invites/{invite_id}", attachment.id),
            None,
            &state,
        )
        .await?;
    }

    let created_at = Utc::now();

    let response = request_json::<CreateInstanceInviteResponse>(
        "create_instance_invite",
        Method::POST,
        &format!("/instances/{}/invites", attachment.id),
        Some(json!({
            "max_age": max_age_seconds,
            "max_uses": max_uses,
        })),
        &state,
    )
    .await?;

    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(SharedInstanceInviteLink {
        invite_id: response.id,
        expires_at: created_at
            + chrono::Duration::seconds(i64::from(max_age_seconds)),
        max_uses,
    })
}

#[tracing::instrument]
pub async fn remove_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await? else {
        return Ok(SharedInstanceUsers::empty());
    };
    ensure_owner(&attachment)?;

    if !user_ids.is_empty() {
        remove_remote_users(&attachment.id, user_ids, &state).await?;
    }

    let remaining_users = get_remote_users(&attachment.id, &state).await?;
    if !has_shared_instance_recipients(&remaining_users, &attachment, &state)
        .await?
    {
        delete_remote_instance(&attachment.id, &state).await?;
        crate::state::clear_shared_instance(instance_id, &state.pool).await?;
        emit_instance(instance_id, InstancePayloadType::Edited).await?;

        return Ok(SharedInstanceUsers::empty());
    }

    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(remaining_users)
}

#[tracing::instrument]
pub async fn unpublish_shared_instance(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(());
    };
    ensure_owner(&attachment)?;

    delete_remote_instance(&attachment.id, &state).await?;
    detach_local_shared_instance(instance_id, metadata.link, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn unlink_shared_instance(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(());
    };
    ensure_member(&attachment)?;

    detach_local_shared_instance(instance_id, metadata.link, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

async fn detach_local_shared_instance(
    instance_id: &str,
    link: InstanceLink,
    state: &State,
) -> crate::Result<()> {
    let link_patch = match link {
        InstanceLink::SharedInstance {
            modpack_project_id: Some(project_id),
            modpack_version_id: Some(version_id),
        } => Some((
            InstanceLink::ModrinthModpack {
                project_id,
                version_id,
            },
            ContentSourceKind::ModrinthModpack,
        )),
        InstanceLink::SharedInstance { .. } => {
            Some((InstanceLink::Unmanaged, ContentSourceKind::Local))
        }
        _ => None,
    };

    if let Some((link, source_kind)) = link_patch {
        crate::state::edit_instance(
            instance_id,
            EditInstance {
                link: Some(link),
                content_set_patch: Some(AppliedContentSetPatch {
                    source_kind: Some(source_kind),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &state.pool,
        )
        .await?;
    }

    crate::state::clear_shared_instance(instance_id, &state.pool).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn publish_shared_instance(
    instance_id: &str,
) -> crate::Result<SharedInstanceAttachment> {
    let state = State::get().await?;
    publish_shared_instance_inner(instance_id, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    shared_attachment(instance_id, &state)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Shared instance attachment was not persisted".to_string(),
            )
            .into()
        })
}

#[tracing::instrument]
pub async fn get_shared_instance_publish_preview(
    instance_id: &str,
) -> crate::Result<Option<SharedInstancePublishPreview>> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(None);
    };
    ensure_owner(&attachment)?;

    let version = match get_latest_remote_version_optional_unavailable(
        &attachment.id,
        &state,
    )
    .await?
    {
        SharedInstanceRemoteResponse::Available(version) => version,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            clear_shared_instance_if_current_user(
                instance_id,
                &attachment,
                &state,
            )
            .await?;
            return Err(shared_instance_unavailable_error(reason));
        }
    };
    let diffs =
        shared_instance_publish_diffs(&metadata, &version, &state).await?;
    set_shared_instance_publish_status(
        instance_id,
        &attachment,
        version.version,
        !diffs.is_empty(),
        &state,
    )
    .await?;

    Ok(Some(SharedInstancePublishPreview {
        shared_instance_id: attachment.id,
        latest_version: version.version,
        diffs,
    }))
}

#[tracing::instrument]
pub async fn install_shared_instance(
    shared_instance_id: &str,
    name: String,
    manager_id: Option<String>,
    server_manager_name: Option<String>,
    server_manager_icon_url: Option<String>,
) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let version = get_latest_remote_version(shared_instance_id, &state).await?;
    let data = shared_instance_install_data(
        shared_instance_id,
        manager_id,
        server_manager_name,
        server_manager_icon_url,
        name,
        version,
        &state,
    )
    .await?;

    crate::install::create_shared_instance(data).await
}

fn shared_instance_invite_install_name(
    shared_instance_id: &str,
    name: String,
) -> String {
    let name = name.trim();
    if name.is_empty() || name == "Shared instance" {
        return format!("Shared instance {shared_instance_id}");
    }

    name.to_string()
}

#[tracing::instrument]
pub async fn get_shared_instance_install_preview(
    shared_instance_id: &str,
    name: String,
) -> crate::Result<SharedInstanceInstallPreview> {
    let state = State::get().await?;
    let version = get_latest_remote_version(shared_instance_id, &state).await?;
    shared_instance_install_preview_from_version(name, version, &state).await
}

#[tracing::instrument]
pub async fn accept_shared_instance_invite_for_install(
    invite_id: &str,
) -> crate::Result<SharedInstanceInviteInstallPreview> {
    let state = State::get().await?;
    let invite = get_shared_instance_invite_info(invite_id, &state).await?;
    let shared_instance_id = invite.instance_id;
    let (manager_id, server_manager_name, server_manager_icon_url) =
        shared_instance_invite_manager(invite.managers);
    let name = shared_instance_invite_install_name(
        &shared_instance_id,
        invite.instance_name,
    );
    accept_shared_instance_invite(&shared_instance_id, invite_id, &state)
        .await?;
    let version =
        get_latest_remote_version(&shared_instance_id, &state).await?;
    let preview =
        shared_instance_install_preview_from_version(name, version, &state)
            .await?;

    Ok(SharedInstanceInviteInstallPreview {
        shared_instance_id,
        manager_id,
        server_manager_name,
        server_manager_icon_url,
        preview,
    })
}

fn shared_instance_invite_manager(
    managers: Vec<InstanceInviteManagerResponse>,
) -> (Option<String>, Option<String>, Option<String>) {
    match managers.into_iter().next() {
        Some(InstanceInviteManagerResponse::User { id }) => {
            (Some(id), None, None)
        }
        Some(InstanceInviteManagerResponse::Server { name, icon }) => {
            (None, Some(name), icon)
        }
        None => (None, None, None),
    }
}

async fn shared_instance_install_preview_from_version(
    name: String,
    version: InstanceVersionResponse,
    state: &State,
) -> crate::Result<SharedInstanceInstallPreview> {
    let name = match name.trim() {
        "" => "Shared instance".to_string(),
        name => name.to_string(),
    };
    let mut content_version_ids = version.modrinth_ids.clone();
    let mut seen_content_version_ids = HashSet::new();
    content_version_ids
        .retain(|id| seen_content_version_ids.insert(id.clone()));
    let modpack = shared_instance_install_modpack(&version, &state).await?;
    let modpack_dependency_count = modpack
        .as_ref()
        .map(|modpack| modpack.dependency_count)
        .unwrap_or_default();
    let modpack_version_id =
        modpack.as_ref().map(|modpack| modpack.version_id.clone());
    if let Some(modpack_version_id) = modpack_version_id.as_deref() {
        content_version_ids.retain(|id| id != modpack_version_id);
    }
    let icon_url = modpack
        .as_ref()
        .and_then(|modpack| modpack.icon_url.clone());

    let external_files = version
        .external_files
        .iter()
        .map(|file| SharedInstanceExternalFilePreview {
            file_name: file.file_name.clone(),
            file_type: file.file_type.clone(),
        })
        .collect::<Vec<_>>();
    let external_file_count = external_files.len();

    Ok(SharedInstanceInstallPreview {
        name,
        icon_url,
        game_version: version.game_version,
        loader: version.loader,
        mod_count: modpack_dependency_count
            + content_version_ids.len()
            + external_file_count,
        external_file_count,
        modpack_version_id,
        content_version_ids,
        external_files,
    })
}

#[tracing::instrument]
pub async fn get_shared_instance_update_preview(
    instance_id: &str,
) -> crate::Result<Option<SharedInstanceUpdatePreview>> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(None);
    };
    if attachment.role != SharedInstanceRole::Member {
        return Ok(None);
    }

    let version =
        get_latest_remote_member_version(instance_id, &attachment, &state)
            .await?;
    if !version.ready {
        return Ok(Some(SharedInstanceUpdatePreview {
            shared_instance_id: attachment.id,
            current_version: attachment.applied_version,
            latest_version: version.version,
            update_available: false,
            diffs: Vec::new(),
        }));
    }

    let update_available = attachment
        .applied_version
        .is_none_or(|current| current < version.version);
    let diffs = if update_available {
        shared_instance_update_diffs(&metadata, &version, &state).await?
    } else {
        Vec::new()
    };

    Ok(Some(SharedInstanceUpdatePreview {
        shared_instance_id: attachment.id,
        current_version: attachment.applied_version,
        latest_version: version.version,
        update_available,
        diffs,
    }))
}

#[tracing::instrument]
pub async fn update_shared_instance(
    instance_id: &str,
) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let attachment = metadata.shared_instance.clone().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Instance is not attached to a shared instance".to_string(),
        )
    })?;
    if attachment.role != SharedInstanceRole::Member {
        return Err(crate::ErrorKind::InputError(
            "Only shared instance members can update from shared instances"
                .to_string(),
        )
        .into());
    }

    let version =
        get_latest_remote_member_version(instance_id, &attachment, &state)
            .await?;
    let data = shared_instance_install_data(
        &attachment.id,
        attachment.manager_id.clone(),
        attachment.server_manager_name.clone(),
        attachment.server_manager_icon_url.clone(),
        metadata.instance.name,
        version,
        &state,
    )
    .await?;
    crate::install::update_shared_instance(instance_id.to_string(), data).await
}

async fn ensure_ready_remote_version_for_invite(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<()> {
    match get_latest_remote_version_optional_unavailable(&attachment.id, state)
        .await?
    {
        SharedInstanceRemoteResponse::Available(_) => Ok(()),
        SharedInstanceRemoteResponse::Unavailable(
            SharedInstanceUnavailableReason::Deleted,
        ) => {
            tracing::info!(
                instance_id,
                shared_instance_id = %attachment.id,
                "Shared instance has no ready version before invite; publishing current content"
            );
            publish_shared_instance_inner(instance_id, state).await
        }
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            clear_shared_instance_if_current_user(
                instance_id,
                attachment,
                state,
            )
            .await?;
            Err(shared_instance_unavailable_error(reason))
        }
    }
}

#[tracing::instrument]
pub async fn accept_pending_shared_instance_invite(
    shared_instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;

    if accept_pending_remote_invite(shared_instance_id, &state).await? {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "No pending invite found for shared instance".to_string(),
    )
    .into())
}

#[tracing::instrument]
pub async fn decline_pending_shared_instance_invite(
    shared_instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    decline_pending_remote_invite(shared_instance_id, &state).await
}

async fn get_latest_remote_member_version(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<InstanceVersionResponse> {
    let version = match get_latest_remote_version_optional_unavailable(
        &attachment.id,
        state,
    )
    .await?
    {
        SharedInstanceRemoteResponse::Available(version) => version,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            if shared_attachment_matches_current_user(attachment, state).await?
            {
                clear_shared_instance_if_current_user(
                    instance_id,
                    attachment,
                    state,
                )
                .await?;
                return Err(shared_instance_unavailable_error(reason));
            }

            return Err(crate::ErrorKind::OtherError(format!(
                "{}: {}",
                shared_instance_unavailable_message(reason),
                attachment.id
            ))
            .into());
        }
    };

    Ok(version)
}

async fn shared_attachment_matches_current_user(
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<bool> {
    let Some(linked_user_id) = attachment.linked_user_id.as_deref() else {
        return Ok(false);
    };

    Ok(linked_modrinth_user_id(state)
        .await?
        .as_deref()
        .is_some_and(|current_user_id| current_user_id == linked_user_id))
}

async fn has_shared_instance_recipients(
    users: &SharedInstanceUsers,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<bool> {
    if users.tokens > 0 {
        return Ok(true);
    }

    let current_user_id = linked_modrinth_user_id(state).await?;

    Ok(users.user_ids.iter().any(|user_id| {
        Some(user_id.as_str()) != attachment.linked_user_id.as_deref()
            && Some(user_id.as_str()) != attachment.manager_id.as_deref()
            && Some(user_id.as_str()) != current_user_id.as_deref()
    }))
}

async fn clear_shared_instance_if_current_user(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<()> {
    if shared_attachment_matches_current_user(attachment, state).await? {
        crate::state::clear_shared_instance(instance_id, &state.pool).await?;
        emit_instance(instance_id, InstancePayloadType::Edited).await?;
    }

    Ok(())
}

fn shared_instance_unavailable_error(
    reason: SharedInstanceUnavailableReason,
) -> crate::Error {
    crate::ErrorKind::SharedInstanceUnavailable(reason).into()
}

fn shared_instance_unavailable_message(
    reason: SharedInstanceUnavailableReason,
) -> &'static str {
    match reason {
        SharedInstanceUnavailableReason::Deleted => {
            "Shared instance was deleted"
        }
        SharedInstanceUnavailableReason::AccessRevoked => {
            "Shared instance access was revoked"
        }
    }
}

async fn shared_instance_install_data(
    shared_instance_id: &str,
    manager_id: Option<String>,
    server_manager_name: Option<String>,
    server_manager_icon_url: Option<String>,
    name: String,
    version: InstanceVersionResponse,
    state: &State,
) -> crate::Result<SharedInstanceInstallData> {
    if !version.ready {
        return Err(crate::ErrorKind::InputError(
            "Shared instance version is not ready to install".to_string(),
        )
        .into());
    }

    let name = shared_instance_name(name);
    let linked_user_id = linked_modrinth_user_id(state).await?;
    let modpack = shared_instance_install_modpack(&version, state).await?;
    let modpack_version_id =
        modpack.as_ref().map(|modpack| modpack.version_id.as_str());
    let modrinth_ids = version
        .modrinth_ids
        .into_iter()
        .filter(|id| Some(id.as_str()) != modpack_version_id)
        .collect();

    Ok(SharedInstanceInstallData {
        shared_instance_id: shared_instance_id.to_string(),
        manager_id,
        server_manager_name,
        server_manager_icon_url,
        linked_user_id,
        name,
        version: version.version,
        modrinth_ids,
        external_files: version
            .external_files
            .into_iter()
            .map(|file| SharedInstanceExternalFileData {
                file_name: file.file_name,
                file_type: file.file_type,
                url: file.url,
            })
            .collect(),
        modpack,
        game_version: version.game_version,
        loader: version.loader,
        loader_version: shared_instance_loader_version(version.loader_version),
    })
}

async fn linked_modrinth_user_id(
    state: &State,
) -> crate::Result<Option<String>> {
    Ok(
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
            .map(|credentials| credentials.user_id),
    )
}

async fn shared_instance_update_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let remote_modpack_id =
        version.modpack_id.as_deref().filter(|id| !id.is_empty());
    let current_modpack_id = shared_modpack_id(&metadata.link);
    let modpack_unlinked =
        current_modpack_id.is_some() && remote_modpack_id.is_none();
    let (current_version_ids, current_external_files) =
        current_shared_content(metadata, modpack_unlinked, state).await?;
    let (latest_version_ids, latest_external_files) =
        remote_shared_content(version);
    let removed_disabled_project_ids = HashSet::new();
    let removed_disabled_external_files = HashSet::new();
    let mut diffs = shared_content_diffs(
        &current_version_ids,
        &current_external_files,
        &latest_version_ids,
        &latest_external_files,
        &removed_disabled_project_ids,
        &removed_disabled_external_files,
        true,
        state,
    )
    .await?;

    let mut configuration_diffs = shared_instance_configuration_diffs(
        current_modpack_id.as_deref(),
        remote_modpack_id,
        &metadata.applied_content_set.game_version,
        &version.game_version,
        metadata.applied_content_set.loader,
        version.loader,
        metadata.applied_content_set.loader_version.as_deref(),
        Some(version.loader_version.as_str()),
        state,
    )
    .await?;
    configuration_diffs.append(&mut diffs);

    Ok(configuration_diffs)
}

async fn shared_instance_publish_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let remote_modpack_id =
        version.modpack_id.as_deref().filter(|id| !id.is_empty());
    let current_modpack_id = shared_modpack_id(&metadata.link);
    let modpack_unlinked =
        remote_modpack_id.is_some() && current_modpack_id.is_none();
    let (latest_version_ids, latest_external_files) =
        remote_publish_content(version, modpack_unlinked, state).await?;
    let (current_version_ids, current_external_files) =
        current_publish_content(metadata, state).await?;
    let (removed_disabled_project_ids, removed_disabled_external_files) =
        current_publish_disabled_content(metadata, state).await?;
    let mut diffs = shared_content_diffs(
        &latest_version_ids,
        &latest_external_files,
        &current_version_ids,
        &current_external_files,
        &removed_disabled_project_ids,
        &removed_disabled_external_files,
        false,
        state,
    )
    .await?;

    let mut configuration_diffs = shared_instance_configuration_diffs(
        remote_modpack_id,
        current_modpack_id.as_deref(),
        &version.game_version,
        &metadata.applied_content_set.game_version,
        version.loader,
        metadata.applied_content_set.loader,
        Some(version.loader_version.as_str()),
        metadata.applied_content_set.loader_version.as_deref(),
        state,
    )
    .await?;
    configuration_diffs.append(&mut diffs);

    Ok(configuration_diffs)
}

async fn shared_instance_configuration_diffs(
    current_modpack_id: Option<&str>,
    new_modpack_id: Option<&str>,
    current_game_version: &str,
    new_game_version: &str,
    current_loader: ModLoader,
    new_loader: ModLoader,
    current_loader_version: Option<&str>,
    new_loader_version: Option<&str>,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let mut diffs = Vec::new();

    if current_modpack_id != new_modpack_id {
        match (current_modpack_id, new_modpack_id) {
            (None, Some(_)) => diffs.push(configuration_diff(
                SharedInstanceUpdateDiffType::ModpackLinked,
                None,
                shared_modpack_version_label(new_modpack_id, state).await,
            )),
            (Some(_), None) => diffs.push(configuration_diff(
                SharedInstanceUpdateDiffType::ModpackUnlinked,
                shared_modpack_version_label(current_modpack_id, state).await,
                None,
            )),
            (Some(current_modpack_id), Some(new_modpack_id)) => {
                let current =
                    shared_modpack_version_details(current_modpack_id, state)
                        .await;
                let new =
                    shared_modpack_version_details(new_modpack_id, state).await;
                let project_name = new
                    .as_ref()
                    .and_then(|details| details.project_name.clone())
                    .or_else(|| {
                        current
                            .as_ref()
                            .and_then(|details| details.project_name.clone())
                    });

                diffs.push(SharedInstanceUpdateDiff {
                    type_: SharedInstanceUpdateDiffType::ModpackUpdated,
                    project_id: None,
                    project_name,
                    file_name: None,
                    current_version_name: current
                        .map(|details| details.version_name),
                    new_version_name: new.map(|details| details.version_name),
                    disabled: false,
                });
            }
            (None, None) => unreachable!(),
        }
    }

    if current_game_version != new_game_version {
        diffs.push(configuration_diff(
            SharedInstanceUpdateDiffType::GameVersionUpdated,
            Some(current_game_version.to_string()),
            Some(new_game_version.to_string()),
        ));
    }

    let current_loader_version =
        normalized_loader_version(current_loader_version);
    let new_loader_version = normalized_loader_version(new_loader_version);
    if current_loader != new_loader
        || current_loader_version != new_loader_version
    {
        diffs.push(configuration_diff(
            SharedInstanceUpdateDiffType::LoaderUpdated,
            Some(shared_loader_label(current_loader, current_loader_version)),
            Some(shared_loader_label(new_loader, new_loader_version)),
        ));
    }

    Ok(diffs)
}

fn configuration_diff(
    type_: SharedInstanceUpdateDiffType,
    current_version_name: Option<String>,
    new_version_name: Option<String>,
) -> SharedInstanceUpdateDiff {
    SharedInstanceUpdateDiff {
        type_,
        project_id: None,
        project_name: None,
        file_name: None,
        current_version_name,
        new_version_name,
        disabled: false,
    }
}

async fn shared_modpack_version_label(
    version_id: Option<&str>,
    state: &State,
) -> Option<String> {
    let Some(version_id) = version_id else {
        return None;
    };
    let details = shared_modpack_version_details(version_id, state).await?;

    Some(match details.project_name {
        Some(project_name) => {
            format!("{project_name} {}", details.version_name)
        }
        None => details.version_name,
    })
}

struct SharedModpackVersionDetails {
    project_name: Option<String>,
    version_name: String,
}

async fn shared_modpack_version_details(
    version_id: &str,
    state: &State,
) -> Option<SharedModpackVersionDetails> {
    let Some(version) = CachedEntry::get_version(
        version_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await
    .ok()
    .flatten() else {
        return Some(SharedModpackVersionDetails {
            project_name: None,
            version_name: version_id.to_string(),
        });
    };
    let project = CachedEntry::get_project(
        &version.project_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await
    .ok()
    .flatten();

    Some(SharedModpackVersionDetails {
        project_name: Some(
            project.map(|project| project.title).unwrap_or(version.name),
        ),
        version_name: version.version_number,
    })
}

fn normalized_loader_version(loader_version: Option<&str>) -> Option<&str> {
    loader_version.filter(|version| !version.is_empty())
}

fn shared_loader_label(
    loader: ModLoader,
    loader_version: Option<&str>,
) -> String {
    let loader_name = match loader {
        ModLoader::Vanilla => "Vanilla",
        ModLoader::Forge => "Forge",
        ModLoader::Fabric => "Fabric",
        ModLoader::Quilt => "Quilt",
        ModLoader::NeoForge => "NeoForge",
    };

    match loader_version {
        Some(version) => format!("{loader_name} {version}"),
        None => loader_name.to_string(),
    }
}

async fn shared_content_diffs(
    current_version_ids: &[String],
    current_external_files: &HashSet<String>,
    latest_version_ids: &[String],
    latest_external_files: &HashSet<String>,
    removed_disabled_project_ids: &HashSet<String>,
    removed_disabled_external_files: &HashSet<String>,
    common_external_files_are_updated: bool,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let current = shared_content_snapshot(
        current_version_ids,
        current_external_files,
        state,
    )
    .await?;
    let latest = shared_content_snapshot(
        latest_version_ids,
        latest_external_files,
        state,
    )
    .await?;
    let options = ContentSetDiffOptions {
        removed_disabled_project_ids: removed_disabled_project_ids.clone(),
        removed_disabled_external_files: removed_disabled_external_files
            .clone(),
        common_external_files_are_updated,
    };
    let content_diffs = diff_content_sets(&current, &latest, &options);
    let project_ids = content_diffs
        .iter()
        .filter_map(|diff| match diff {
            ContentSetDiffEntry::Project { project_id, .. } => {
                Some(project_id.clone())
            }
            ContentSetDiffEntry::ExternalFile { .. } => None,
        })
        .collect::<HashSet<_>>();
    let project_names = shared_project_names(&project_ids, state).await?;

    let mut diffs = Vec::new();
    for diff in content_diffs {
        match diff {
            ContentSetDiffEntry::Project {
                kind,
                project_id,
                current_version_name,
                new_version_name,
                disabled,
            } => {
                let project_name = Some(
                    project_names
                        .get(&project_id)
                        .cloned()
                        .unwrap_or_else(|| project_id.clone()),
                );
                diffs.push(SharedInstanceUpdateDiff {
                    type_: shared_update_diff_type(kind),
                    project_id: Some(project_id),
                    project_name,
                    file_name: None,
                    current_version_name,
                    new_version_name,
                    disabled,
                });
            }
            ContentSetDiffEntry::ExternalFile {
                kind,
                file_name,
                disabled,
            } => {
                diffs.push(SharedInstanceUpdateDiff {
                    type_: shared_update_diff_type(kind),
                    project_id: None,
                    project_name: None,
                    file_name: Some(file_name),
                    current_version_name: None,
                    new_version_name: None,
                    disabled,
                });
            }
        }
    }

    diffs.sort_by(|a, b| {
        a.project_name
            .as_deref()
            .or(a.file_name.as_deref())
            .cmp(&b.project_name.as_deref().or(b.file_name.as_deref()))
    });
    Ok(diffs)
}

fn shared_update_diff_type(
    kind: ContentSetDiffKind,
) -> SharedInstanceUpdateDiffType {
    match kind {
        ContentSetDiffKind::Added => SharedInstanceUpdateDiffType::Added,
        ContentSetDiffKind::Removed => SharedInstanceUpdateDiffType::Removed,
        ContentSetDiffKind::Updated => SharedInstanceUpdateDiffType::Updated,
    }
}

async fn shared_content_snapshot(
    version_ids: &[String],
    external_files: &HashSet<String>,
    state: &State,
) -> crate::Result<ContentSetSnapshot> {
    let versions = shared_versions_by_project(version_ids, state)
        .await?
        .into_values()
        .map(ContentSetSnapshotVersion::from)
        .collect();

    Ok(ContentSetSnapshot {
        versions,
        external_files: external_files.clone(),
    })
}

fn remote_shared_content(
    version: &InstanceVersionResponse,
) -> (Vec<String>, HashSet<String>) {
    let mut version_ids = version.modrinth_ids.clone();
    if let Some(modpack_id) = version.modpack_id.as_deref() {
        version_ids.retain(|id| id != modpack_id);
    }
    dedupe_strings(&mut version_ids);

    (
        version_ids,
        version
            .external_files
            .iter()
            .map(|file| file.file_name.clone())
            .collect(),
    )
}

async fn remote_publish_content(
    version: &InstanceVersionResponse,
    include_modpack_dependencies: bool,
    state: &State,
) -> crate::Result<(Vec<String>, HashSet<String>)> {
    let mut version_ids = version.modrinth_ids.clone();
    if let Some(modpack_id) =
        version.modpack_id.as_deref().filter(|id| !id.is_empty())
    {
        version_ids.retain(|id| id != modpack_id);

        if include_modpack_dependencies {
            version_ids.extend(
                modpack_dependency_version_ids(modpack_id, state).await?,
            );
        }
    }
    dedupe_strings(&mut version_ids);

    Ok((
        version_ids,
        version
            .external_files
            .iter()
            .map(|file| file.file_name.clone())
            .collect(),
    ))
}

async fn modpack_dependency_version_ids(
    modpack_id: &str,
    state: &State,
) -> crate::Result<Vec<String>> {
    let modpack_version = CachedEntry::get_version(
        modpack_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Shared instance modpack version was not found".to_string(),
        )
    })?;

    Ok(modpack_version
        .dependencies
        .into_iter()
        .filter_map(|dependency| dependency.version_id)
        .collect())
}

async fn shared_instance_install_modpack(
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Option<crate::install::SharedInstanceInstallModpack>> {
    let Some(modpack_id) =
        version.modpack_id.as_deref().filter(|id| !id.is_empty())
    else {
        return Ok(None);
    };
    let modpack_version = CachedEntry::get_version(
        modpack_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Shared instance modpack version was not found".to_string(),
        )
    })?;
    let project = CachedEntry::get_project(
        &modpack_version.project_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(Some(crate::install::SharedInstanceInstallModpack {
        project_id: modpack_version.project_id,
        version_id: modpack_version.id,
        dependency_count: modpack_version.dependencies.len(),
        title: project
            .as_ref()
            .map(|project| project.title.clone())
            .unwrap_or(modpack_version.name),
        icon_url: project.and_then(|project| project.icon_url),
    }))
}

fn shared_instance_loader_version(loader_version: String) -> Option<String> {
    (!loader_version.is_empty()).then_some(loader_version)
}

async fn current_shared_content(
    metadata: &crate::state::InstanceMetadata,
    include_linked_modpack_content: bool,
    state: &State,
) -> crate::Result<(Vec<String>, HashSet<String>)> {
    let entries =
        crate::state::instances::adapters::sqlite::content_rows::get_content_entries(
            &metadata.applied_content_set.id,
            &state.pool,
        )
        .await?;
    let files = crate::state::instances::adapters::sqlite::content_rows::get_instance_files(
        &metadata.instance.id,
        &state.pool,
    )
    .await?
    .into_iter()
    .map(|file| (file.id.clone(), file))
    .collect::<HashMap<_, _>>();
    let mut version_ids = Vec::new();
    let mut external_files = HashSet::new();

    for entry in entries {
        let include_entry = entry.source_kind
            == crate::state::ContentSourceKind::SharedInstance
            || (include_linked_modpack_content
                && entry.source_kind
                    == crate::state::ContentSourceKind::ModrinthModpack);
        if !include_entry {
            continue;
        }

        if let Some(version_id) = entry.version_id {
            version_ids.push(version_id);
            continue;
        }

        let Some(file_id) = entry.file_id else {
            continue;
        };
        if let Some(file) = files.get(&file_id) {
            external_files.insert(file.file_name.clone());
        }
    }
    if include_linked_modpack_content
        && let Some(modpack_id) = shared_modpack_id(&metadata.link)
    {
        version_ids
            .extend(modpack_dependency_version_ids(&modpack_id, state).await?);
    }
    dedupe_strings(&mut version_ids);

    Ok((version_ids, external_files))
}

async fn current_publish_content(
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<(Vec<String>, HashSet<String>)> {
    let (mut version_ids, external_files) =
        collect_publish_content(&metadata.instance.id, state).await?;
    if let Some(modpack_id) = shared_modpack_id(&metadata.link) {
        version_ids.retain(|id| id != &modpack_id);
    }
    dedupe_strings(&mut version_ids);

    Ok((
        version_ids,
        external_files
            .into_iter()
            .map(|file| file.file_name)
            .collect(),
    ))
}

async fn current_publish_disabled_content(
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<(HashSet<String>, HashSet<String>)> {
    let items =
        crate::state::list_content(&metadata.instance.id, None, None, state)
            .await?;
    let modpack_id = shared_modpack_id(&metadata.link);
    let mut project_ids = HashSet::new();
    let mut version_ids = Vec::new();
    let mut seen_version_ids = HashSet::new();
    let mut external_files = HashSet::new();

    for item in items {
        if item.enabled {
            continue;
        }

        let is_modpack = item.version.as_ref().is_some_and(|version| {
            modpack_id.as_deref() == Some(version.id.as_str())
        });
        if is_modpack {
            continue;
        }

        if let Some(project) = item.project.as_ref() {
            project_ids.insert(project.id.clone());
        }

        if let Some(version) = item.version {
            if seen_version_ids.insert(version.id.clone()) {
                version_ids.push(version.id);
            }
            continue;
        }

        if item.file_path.is_empty() {
            continue;
        }

        external_files.insert(enabled_file_name(&item.file_name));
    }

    project_ids.extend(
        shared_versions_by_project(&version_ids, state)
            .await?
            .into_keys(),
    );

    Ok((project_ids, external_files))
}

async fn shared_versions_by_project(
    version_ids: &[String],
    state: &State,
) -> crate::Result<HashMap<String, crate::state::Version>> {
    let version_id_refs =
        version_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &version_id_refs,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(versions
        .into_iter()
        .map(|version| (version.project_id.clone(), version))
        .collect())
}

async fn shared_project_names(
    project_ids: &HashSet<String>,
    state: &State,
) -> crate::Result<HashMap<String, String>> {
    let project_id_refs =
        project_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let projects = CachedEntry::get_project_many(
        &project_id_refs,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(projects
        .into_iter()
        .map(|project| (project.id, project.title))
        .collect())
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

fn enabled_file_name(file_name: &str) -> String {
    file_name
        .strip_suffix(".disabled")
        .unwrap_or(file_name)
        .to_string()
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn shared_instance_name(name: String) -> String {
    match name.trim() {
        "" => "Shared instance".to_string(),
        name => name.to_string(),
    }
}

pub(crate) async fn mark_shared_instance_stale(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let Some(metadata) =
        crate::state::get_instance(instance_id, &state.pool).await?
    else {
        return Ok(());
    };
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(());
    };
    if attachment.role != SharedInstanceRole::Owner {
        return Ok(());
    }

    let version = match get_latest_remote_version_optional_unavailable(
        &attachment.id,
        state,
    )
    .await
    {
        Ok(SharedInstanceRemoteResponse::Available(version)) => version,
        Ok(SharedInstanceRemoteResponse::Unavailable(reason)) => {
            tracing::warn!(
                instance_id,
                shared_instance_id = %attachment.id,
                reason = ?reason,
                "Shared instance was unavailable while reconciling stale status"
            );
            clear_shared_instance_if_current_user(
                instance_id,
                &attachment,
                state,
            )
            .await?;
            return Ok(());
        }
        Err(error) => {
            tracing::warn!(
                instance_id,
                shared_instance_id = %attachment.id,
                error = %error,
                "Failed to check shared instance diff before marking stale"
            );
            return crate::state::mark_shared_instance_stale(
                instance_id,
                &state.pool,
            )
            .await;
        }
    };

    let has_changes = match shared_instance_publish_diffs(
        &metadata, &version, state,
    )
    .await
    {
        Ok(diffs) => !diffs.is_empty(),
        Err(error) => {
            tracing::warn!(
                instance_id,
                shared_instance_id = %attachment.id,
                error = %error,
                "Failed to calculate shared instance diff before marking stale"
            );
            return crate::state::mark_shared_instance_stale(
                instance_id,
                &state.pool,
            )
            .await;
        }
    };

    set_shared_instance_publish_status(
        instance_id,
        &attachment,
        version.version,
        has_changes,
        state,
    )
    .await
}

async fn set_shared_instance_publish_status(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    latest_version: i32,
    has_changes: bool,
    state: &State,
) -> crate::Result<()> {
    let (status, applied_version) = if has_changes {
        (ContentSetSyncStatus::Stale, attachment.applied_version)
    } else {
        (ContentSetSyncStatus::UpToDate, Some(latest_version))
    };

    crate::state::set_shared_instance_sync_status(
        instance_id,
        status,
        applied_version,
        Some(latest_version),
        &state.pool,
    )
    .await
}

async fn publish_shared_instance_inner(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let attachment =
        shared_attachment(instance_id, state)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(
                    "Instance is not attached to a shared instance".to_string(),
                )
            })?;
    ensure_owner(&attachment)?;
    tracing::info!(
        instance_id,
        shared_instance_id = %attachment.id,
        applied_version = attachment.applied_version,
        latest_version = attachment.latest_version,
        "Publishing shared instance content"
    );

    crate::state::set_shared_instance_sync_status(
        instance_id,
        ContentSetSyncStatus::Applying,
        attachment.applied_version,
        attachment.latest_version,
        &state.pool,
    )
    .await?;

    let result =
        publish_current_content(instance_id, &attachment.id, state).await;

    match result {
        Ok(version) => {
            tracing::info!(
                instance_id,
                shared_instance_id = %attachment.id,
                version,
                "Published shared instance content"
            );
            crate::state::set_shared_instance_sync_status(
                instance_id,
                ContentSetSyncStatus::UpToDate,
                Some(version),
                Some(version),
                &state.pool,
            )
            .await?;
            Ok(())
        }
        Err(error) => {
            tracing::warn!(
                instance_id,
                shared_instance_id = %attachment.id,
                error = %error,
                "Failed to publish shared instance content"
            );
            crate::state::set_shared_instance_sync_status(
                instance_id,
                ContentSetSyncStatus::Error,
                attachment.applied_version,
                attachment.latest_version,
                &state.pool,
            )
            .await?;
            Err(error)
        }
    }
}

async fn publish_current_content(
    instance_id: &str,
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<i32> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    ensure_shareable_link(&metadata.link)?;
    update_remote_instance(
        shared_instance_id,
        shared_instance_name(metadata.instance.name.clone()),
        state,
    )
    .await?;
    let modpack_id = shared_modpack_id(&metadata.link);
    let (modrinth_ids, external_files) =
        collect_publish_content(instance_id, state).await?;
    tracing::debug!(
        instance_id,
        shared_instance_id,
        modpack_id = modpack_id.as_deref().unwrap_or("none"),
        modrinth_id_count = modrinth_ids.len(),
        external_file_count = external_files.len(),
        "Creating shared instance version"
    );
    let external_file_data = external_files
        .iter()
        .map(|file| ExternalFileData {
            file_name: file.file_name.clone(),
            file_type: file.file_type.clone(),
        })
        .collect::<Vec<_>>();
    let response = request_json_optional_unavailable::<InstanceVersionResponse>(
        "create_instance_version",
        Method::POST,
        &format!("/instances/{shared_instance_id}/versions"),
        Some(json!({
            "modrinth_ids": modrinth_ids,
            "external_files": external_file_data,
            "modpack_id": modpack_id,
            "game_version": metadata.applied_content_set.game_version.clone(),
            "loader": metadata.applied_content_set.loader.as_str(),
            "loader_version": metadata
                .applied_content_set
                .loader_version
                .clone()
                .unwrap_or_default(),
        })),
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await?;
    let response = match response {
        SharedInstanceRemoteResponse::Available(response) => response,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            if let Some(attachment) = metadata.shared_instance.as_ref() {
                clear_shared_instance_if_current_user(
                    instance_id,
                    attachment,
                    state,
                )
                .await?;
            }

            return Err(shared_instance_unavailable_error(reason));
        }
    };

    if !response.external_files.is_empty() {
        upload_external_files(
            &metadata.instance.path,
            &external_files,
            &response.external_files,
            state,
        )
        .await?;
        mark_external_files_ready(
            shared_instance_id,
            response.version,
            &response.external_files,
            state,
        )
        .await?;
    } else if !response.ready {
        tracing::debug!(
            "Shared instance version {} was not ready but had no external files",
            response.version
        );
    }

    Ok(response.version)
}

async fn collect_publish_content(
    instance_id: &str,
    state: &State,
) -> crate::Result<(Vec<String>, Vec<ExternalFileCandidate>)> {
    let items =
        crate::state::list_content(instance_id, None, None, state).await?;

    let mut modrinth_ids = Vec::new();
    let mut seen_modrinth_ids = HashSet::new();
    let mut external_files = Vec::new();
    let mut seen_external_files = HashSet::new();

    for item in items {
        if !item.enabled {
            continue;
        }

        if let Some(version) = item.version {
            if seen_modrinth_ids.insert(version.id.clone()) {
                modrinth_ids.push(version.id);
            }
            continue;
        }

        if item.file_path.is_empty() {
            continue;
        }

        let file_type = file_type(item.project_type);
        let external_key = format!("{}:{file_type}", item.file_path);
        if seen_external_files.insert(external_key) {
            external_files.push(ExternalFileCandidate {
                file_name: item.file_name,
                file_type,
                file_path: item.file_path,
            });
        }
    }

    Ok((modrinth_ids, external_files))
}

fn shared_modpack_id(link: &InstanceLink) -> Option<String> {
    match link {
        InstanceLink::ModrinthModpack { version_id, .. } => {
            Some(version_id.clone())
        }
        InstanceLink::ServerProjectModpack {
            content_version_id, ..
        } => Some(content_version_id.clone()),
        InstanceLink::SharedInstance {
            modpack_version_id: Some(version_id),
            ..
        } => Some(version_id.clone()),
        _ => None,
    }
}

fn ensure_shareable_link(link: &InstanceLink) -> crate::Result<()> {
    if matches!(link, InstanceLink::ImportedModpack { .. }) {
        return Err(crate::ErrorKind::InputError(
            "You must unlink this modpack to share your instance".to_string(),
        )
        .into());
    }

    Ok(())
}

async fn upload_external_files(
    instance_path: &str,
    candidates: &[ExternalFileCandidate],
    uploads: &[ExternalFileResponse],
    state: &State,
) -> crate::Result<()> {
    for upload in uploads {
        let candidate = candidates
            .iter()
            .find(|candidate| {
                candidate.file_name == upload.file_name
                    && candidate.file_type == upload.file_type
            })
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Shared instance service requested unknown external file {}",
                    upload.file_name
                ))
            })?;
        let path = state
            .directories
            .instances_dir()
            .join(instance_path)
            .join(&candidate.file_path);
        let bytes = crate::util::io::read(path).await?;
        let response =
            REQWEST_CLIENT.put(&upload.url).body(bytes).send().await?;

        if !response.status().is_success() {
            return Err(crate::ErrorKind::OtherError(format!(
                "External file upload failed with status {}",
                response.status()
            ))
            .into());
        }
    }

    Ok(())
}

async fn mark_external_files_ready(
    shared_instance_id: &str,
    version: i32,
    uploads: &[ExternalFileResponse],
    state: &State,
) -> crate::Result<()> {
    for upload in uploads {
        request_empty(
            "mark_version_file_ready",
            Method::POST,
            &format!(
                "/instances/{shared_instance_id}/versions/{version}/files"
            ),
            Some(json!({ "file_name": &upload.file_name })),
            state,
        )
        .await?;
    }

    Ok(())
}

async fn shared_attachment(
    instance_id: &str,
    state: &State,
) -> crate::Result<Option<SharedInstanceAttachment>> {
    Ok(crate::state::get_instance(instance_id, &state.pool)
        .await?
        .and_then(|metadata| metadata.shared_instance))
}

async fn shared_instance_for_invites(
    instance_id: &str,
    user_count: usize,
    state: &State,
) -> crate::Result<(crate::state::InstanceMetadata, SharedInstanceAttachment)> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let attachment = match metadata.shared_instance.clone() {
        Some(attachment) => {
            tracing::debug!(
                instance_id,
                shared_instance_id = %attachment.id,
                role = attachment.role.as_str(),
                user_count,
                "Using existing shared instance attachment for invite"
            );
            attachment
        }
        None => {
            ensure_shareable_link(&metadata.link)?;
            tracing::info!(
                instance_id,
                user_count,
                "Creating shared instance before first invite"
            );
            let remote = create_remote_instance(
                shared_instance_name(metadata.instance.name.clone()),
                state,
            )
            .await?;
            let linked_user_id = linked_modrinth_user_id(state).await?;
            tracing::info!(
                instance_id,
                shared_instance_id = %remote.id,
                "Created remote shared instance"
            );
            crate::state::attach_shared_instance(
                instance_id,
                &remote.id,
                SharedInstanceRole::Owner,
                None,
                None,
                None,
                linked_user_id,
                ContentSetSyncStatus::Unknown,
                None,
                None,
                &state.pool,
            )
            .await?;
            tracing::debug!(
                instance_id,
                shared_instance_id = %remote.id,
                "Attached local instance as shared instance owner"
            );
            publish_shared_instance_inner(instance_id, state).await?;
            shared_attachment(instance_id, state)
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError(
                        "Shared instance attachment was not persisted"
                            .to_string(),
                    )
                })?
        }
    };

    Ok((metadata, attachment))
}

fn ensure_owner(attachment: &SharedInstanceAttachment) -> crate::Result<()> {
    if attachment.role == SharedInstanceRole::Owner {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Only the owner instance can manage shared instance users".to_string(),
    )
    .into())
}

fn ensure_member(attachment: &SharedInstanceAttachment) -> crate::Result<()> {
    if attachment.role == SharedInstanceRole::Member {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Only shared instance members can unlink from shared instances"
            .to_string(),
    )
    .into())
}

fn file_type(project_type: ProjectType) -> String {
    project_type.get_name().to_string()
}

async fn create_remote_instance(
    name: String,
    state: &State,
) -> crate::Result<CreateInstanceResponse> {
    request_json(
        "create_instance",
        Method::POST,
        "/instances",
        Some(json!({ "name": name })),
        state,
    )
    .await
}

async fn delete_remote_instance(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "delete_instance",
        Method::DELETE,
        &format!("/instances/{shared_instance_id}"),
        None,
        state,
    )
    .await
}

async fn update_remote_instance(
    shared_instance_id: &str,
    name: String,
    state: &State,
) -> crate::Result<()> {
    let operation = "update_instance";
    let method = Method::PATCH;
    let path = format!("/instances/{shared_instance_id}");
    let response = send_request(
        operation,
        method.clone(),
        &path,
        Some(json!({ "name": name })),
        state,
    )
    .await?;

    if response.status().is_success() {
        return Ok(());
    }

    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        let body = response.text().await.unwrap_or_default();
        tracing::warn!(
            operation,
            method = method.as_str(),
            path,
            status = StatusCode::METHOD_NOT_ALLOWED.as_u16(),
            response_body = %body,
            "Shared instances API does not support remote instance updates; skipping name sync"
        );
        return Ok(());
    }

    shared_instances_request_error(operation, method, &path, response).await
}

async fn get_remote_users(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<SharedInstanceUsers> {
    let users = request_json::<RemoteSharedInstanceUsersResponse>(
        "get_instance_users",
        Method::GET,
        &format!("/instances/{shared_instance_id}/users"),
        None,
        state,
    )
    .await?;

    Ok(users.into_shared_users())
}

async fn get_latest_remote_version(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<InstanceVersionResponse> {
    match get_latest_remote_version_optional_unavailable(
        shared_instance_id,
        state,
    )
    .await?
    {
        SharedInstanceRemoteResponse::Available(version) => Ok(version),
        SharedInstanceRemoteResponse::Unavailable(
            reason @ SharedInstanceUnavailableReason::AccessRevoked,
        ) => {
            if !accept_pending_remote_invite(shared_instance_id, state).await? {
                return Err(shared_instance_unavailable_error(reason));
            }

            match get_latest_remote_version_optional_unavailable(
                shared_instance_id,
                state,
            )
            .await?
            {
                SharedInstanceRemoteResponse::Available(version) => Ok(version),
                SharedInstanceRemoteResponse::Unavailable(reason) => {
                    Err(shared_instance_unavailable_error(reason))
                }
            }
        }
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            Err(shared_instance_unavailable_error(reason))
        }
    }
}

async fn get_latest_remote_version_optional_unavailable(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<SharedInstanceRemoteResponse<InstanceVersionResponse>> {
    get_latest_remote_version_optional_unavailable_with_auth(
        shared_instance_id,
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await
}

async fn get_latest_remote_version_optional_unavailable_with_auth(
    shared_instance_id: &str,
    state: &State,
    auth: SharedInstancesRequestAuth,
) -> crate::Result<SharedInstanceRemoteResponse<InstanceVersionResponse>> {
    request_json_optional_unavailable(
        "get_latest_instance_version",
        Method::GET,
        &format!("/instances/{shared_instance_id}/versions"),
        None,
        state,
        auth,
    )
    .await
}

async fn add_remote_users(
    shared_instance_id: &str,
    user_ids: Vec<String>,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "add_instance_users",
        Method::POST,
        &format!("/instances/{shared_instance_id}/users"),
        Some(json!({ "user_ids": user_ids })),
        state,
    )
    .await
}

async fn remove_remote_users(
    shared_instance_id: &str,
    user_ids: Vec<String>,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "remove_instance_users",
        Method::DELETE,
        &format!("/instances/{shared_instance_id}/users"),
        Some(json!({ "user_ids": user_ids })),
        state,
    )
    .await
}

async fn accept_pending_remote_invite(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<bool> {
    let operation = "accept_pending_instance_invite";
    let method = Method::POST;
    let path = format!("/instances/{shared_instance_id}/invites/pending");
    let response =
        send_request(operation, method.clone(), &path, None, state).await?;

    match response.status() {
        StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
        StatusCode::NOT_FOUND => Ok(false),
        status if status.is_success() => Ok(true),
        _ => {
            shared_instances_request_error(operation, method, &path, response)
                .await
        }
    }
}

async fn accept_shared_instance_invite(
    shared_instance_id: &str,
    invite_id: &str,
    state: &State,
) -> crate::Result<()> {
    let path = format!("/instances/{shared_instance_id}/invites/{invite_id}");
    let operation = "accept_instance_invite";
    let method = Method::POST;

    let response = send_request_with_auth(
        operation,
        method.clone(),
        &path,
        None,
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await?;
    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if status == StatusCode::BAD_REQUEST && body.contains("already has access")
    {
        return Ok(());
    }

    tracing::warn!(
        operation,
        method = method.as_str(),
        path,
        status = status.as_u16(),
        response_body = %body,
        "Shared instances API request failed"
    );
    Err(crate::ErrorKind::OtherError(format!(
        "Shared instances API request {operation} {method} {path} failed with status {status}: {body}"
    ))
    .into())
}

async fn get_shared_instance_invite_info(
    invite_id: &str,
    state: &State,
) -> crate::Result<InstanceInviteInfoResponse> {
    let operation = "get_instance_invite";
    let method = Method::GET;
    let path = format!("/invites/{invite_id}");
    let response = send_request_with_auth(
        operation,
        method.clone(),
        &path,
        None,
        state,
        SharedInstancesRequestAuth::None,
    )
    .await?;
    if !response.status().is_success() {
        return shared_instances_request_error(
            operation, method, &path, response,
        )
        .await;
    }

    decode_json_response(operation, &path, response).await
}

async fn decline_pending_remote_invite(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    request_empty(
        "decline_pending_instance_invite",
        Method::DELETE,
        &format!("/instances/{shared_instance_id}/invites/pending"),
        None,
        state,
    )
    .await
}

async fn request_json<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let response = request(operation, method, path, body, state).await?;
    decode_json_response(operation, path, response).await
}

async fn request_json_optional_unavailable<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
    auth: SharedInstancesRequestAuth,
) -> crate::Result<SharedInstanceRemoteResponse<T>>
where
    T: DeserializeOwned,
{
    let response = send_request_with_auth(
        operation,
        method.clone(),
        path,
        body,
        state,
        auth,
    )
    .await?;
    if let Some(reason) =
        SharedInstanceUnavailableReason::from_status(response.status())
    {
        if reason == SharedInstanceUnavailableReason::AccessRevoked
            && matches!(auth, SharedInstancesRequestAuth::ModrinthSession)
            && !active_modrinth_session_is_valid(state).await?
        {
            tracing::warn!(
                operation,
                method = method.as_str(),
                path,
                status = response.status().as_u16(),
                "Shared instances API returned unauthorized while Modrinth auth is unavailable"
            );
            return Err(crate::ErrorKind::NoCredentialsError.into());
        }

        tracing::warn!(
            operation,
            method = method.as_str(),
            path,
            status = response.status().as_u16(),
            "Shared instances API resource is unavailable"
        );
        return Ok(SharedInstanceRemoteResponse::Unavailable(reason));
    }

    if !response.status().is_success() {
        return shared_instances_request_error(
            operation, method, path, response,
        )
        .await;
    }

    decode_json_response(operation, path, response)
        .await
        .map(SharedInstanceRemoteResponse::Available)
}

async fn active_modrinth_session_is_valid(
    state: &State,
) -> crate::Result<bool> {
    let Some(credentials) =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
    else {
        return Ok(false);
    };

    let _permit = state.api_semaphore.0.acquire().await?;
    let response = INSECURE_REQWEST_CLIENT
        .get(concat!(env!("MODRINTH_API_URL"), "user"))
        .header("Authorization", &credentials.session)
        .send()
        .await?;

    if response.status() == StatusCode::UNAUTHORIZED {
        ModrinthCredentials::remove(&credentials.user_id, &state.pool).await?;
        return Ok(false);
    }

    if response.status().is_success() {
        return Ok(true);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(crate::ErrorKind::OtherError(format!(
        "Modrinth auth validation failed with status {status}: {body}"
    ))
    .into())
}

async fn decode_json_response<T>(
    operation: &'static str,
    path: &str,
    response: reqwest::Response,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    let body = response.text().await?;
    tracing::debug!(
        operation,
        path,
        status = status.as_u16(),
        response_body = %body,
        "Decoding shared instances API response"
    );

    serde_json::from_str::<T>(&body).map_err(|error| {
        crate::ErrorKind::OtherError(format!(
            "Shared instances API request {operation} {path} failed to decode JSON response with status {status}: {error}; body: {body}"
        ))
        .into()
    })
}

async fn request_empty(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<()> {
    request(operation, method, path, body, state).await?;
    Ok(())
}

async fn request(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    let response =
        send_request(operation, method.clone(), path, body, state).await?;
    if response.status().is_success() {
        return Ok(response);
    }

    shared_instances_request_error(operation, method, path, response).await
}

async fn send_request(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    send_request_with_auth(
        operation,
        method,
        path,
        body,
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await
}

async fn send_request_with_auth(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
    auth: SharedInstancesRequestAuth,
) -> crate::Result<reqwest::Response> {
    let modrinth_credentials =
        if matches!(auth, SharedInstancesRequestAuth::ModrinthSession) {
            Some(
                ModrinthCredentials::get_and_refresh(
                    &state.pool,
                    &state.api_semaphore,
                )
                .await?
                .ok_or(crate::ErrorKind::NoCredentialsError)?,
            )
        } else {
            None
        };
    let _permit = state.api_semaphore.0.acquire().await?;
    let base_url = service_base_url();
    let url = service_url(base_url, path);
    let mut request =
        shared_instances_client(base_url).request(method.clone(), &url);
    let mut user_id = None;

    match auth {
        SharedInstancesRequestAuth::ModrinthSession => {
            let credentials = modrinth_credentials
                .expect("Modrinth session credentials were loaded");
            user_id = Some(credentials.user_id);
            request = request.bearer_auth(credentials.session);
        }
        SharedInstancesRequestAuth::None => {}
    }

    tracing::debug!(
        operation,
        method = method.as_str(),
        path,
        url = %url,
        user_id = user_id.as_deref(),
        auth = auth.label(),
        has_body = body.is_some(),
        "Sending shared instances API request"
    );

    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request.send().await?;
    if response.status().is_success() {
        tracing::debug!(
            operation,
            method = method.as_str(),
            path,
            url = %url,
            status = response.status().as_u16(),
            "Shared instances API request succeeded"
        );
    }
    Ok(response)
}

async fn shared_instances_request_error<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    response: reqwest::Response,
) -> crate::Result<T> {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    tracing::warn!(
        operation,
        method = method.as_str(),
        path,
        status = status.as_u16(),
        response_body = %body,
        "Shared instances API request failed"
    );
    Err(crate::ErrorKind::OtherError(format!(
        "Shared instances API request {operation} {method} {path} failed with status {status}: {body}"
    ))
    .into())
}

fn service_url(base_url: &str, path: &str) -> String {
    format!("{base_url}/v1{path}")
}

fn service_base_url() -> &'static str {
    env!("SHARED_INSTANCES_API_BASE_URL").trim_end_matches('/')
}

fn shared_instances_client(base_url: &str) -> &'static reqwest::Client {
    if base_url.starts_with("https://") {
        &REQWEST_CLIENT
    } else {
        &INSECURE_REQWEST_CLIENT
    }
}
