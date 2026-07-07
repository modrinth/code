use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::install::{
    InstallJobSnapshot, SharedInstanceExternalFileData,
    SharedInstanceInstallData,
};
use crate::state::instances::{InstanceLink, SharedInstanceAttachment};
use crate::state::{
    CacheBehaviour, CachedEntry, ContentSetSyncStatus, ModLoader,
    ModrinthCredentials, ProjectType, SharedInstanceRole, State,
};
use crate::util::fetch::{INSECURE_REQWEST_CLIENT, REQWEST_CLIENT};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

const SHARED_INSTANCE_UNAVAILABLE_ERROR_CODE: &str =
    "shared_instance_unavailable";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedInstanceUsers {
    pub user_ids: Vec<String>,
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

#[tracing::instrument]
pub async fn get_shared_instance_users(
    instance_id: &str,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await? else {
        return Ok(SharedInstanceUsers {
            user_ids: Vec::new(),
        });
    };

    get_remote_users(&attachment.id, &state).await
}

#[tracing::instrument]
pub async fn invite_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let attachment = match shared_attachment(instance_id, &state).await? {
        Some(attachment) => {
            tracing::debug!(
                instance_id,
                shared_instance_id = %attachment.id,
                role = attachment.role.as_str(),
                user_count = user_ids.len(),
                "Using existing shared instance attachment for invite"
            );
            attachment
        }
        None => {
            let metadata = ensure_shareable_instance(instance_id, &state).await?;
            tracing::info!(
                instance_id,
                user_count = user_ids.len(),
                "Creating shared instance before first invite"
            );
            let remote = create_remote_instance(
                shared_instance_name(metadata.instance.name),
                &state,
            )
            .await?;
            let linked_user_id = linked_modrinth_user_id(&state).await?;
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
            publish_shared_instance_inner(instance_id, &state).await?;
            shared_attachment(instance_id, &state)
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError(
                        "Shared instance attachment was not persisted"
                            .to_string(),
                    )
                })?
        }
    };

    ensure_owner(&attachment)?;
    if !user_ids.is_empty() {
        tracing::info!(
            instance_id,
            shared_instance_id = %attachment.id,
            user_count = user_ids.len(),
            "Adding users to shared instance"
        );
        add_remote_users(&attachment.id, user_ids.clone(), &state).await?;
    }
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(SharedInstanceUsers { user_ids })
}

#[tracing::instrument]
pub async fn remove_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await? else {
        return Ok(SharedInstanceUsers {
            user_ids: Vec::new(),
        });
    };
    ensure_owner(&attachment)?;

    if !user_ids.is_empty() {
        remove_remote_users(&attachment.id, user_ids, &state).await?;
    }

    let remaining_users = get_remote_users(&attachment.id, &state).await?;
    if remaining_users.user_ids.is_empty() {
        delete_remote_instance(&attachment.id, &state).await?;
        crate::state::clear_shared_instance(instance_id, &state.pool).await?;
    }

    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(remaining_users)
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

    let version = get_latest_remote_version(&attachment.id, &state).await?;
    let diffs = shared_instance_publish_diffs(&metadata, &version, &state).await?;

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
) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let version = get_latest_remote_version(shared_instance_id, &state).await?;
    let data = shared_instance_install_data(
        shared_instance_id,
        manager_id,
        name,
        version,
        &state,
    )
    .await?;

    crate::install::create_shared_instance(data).await
}

#[tracing::instrument]
pub async fn get_shared_instance_install_preview(
    shared_instance_id: &str,
    name: String,
) -> crate::Result<SharedInstanceInstallPreview> {
    let state = State::get().await?;
    let version = get_latest_remote_version(shared_instance_id, &state).await?;
    let name = match name.trim() {
        "" => "Shared instance".to_string(),
        name => name.to_string(),
    };
    let mut content_version_ids = version.modrinth_ids.clone();
    let mut seen_content_version_ids = HashSet::new();
    content_version_ids.retain(|id| seen_content_version_ids.insert(id.clone()));
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
    let icon_url = modpack.as_ref().and_then(|modpack| modpack.icon_url.clone());

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
        metadata.instance.name,
        version,
        &state,
    )
    .await?;

    crate::install::update_shared_instance(instance_id.to_string(), data).await
}

async fn get_latest_remote_member_version(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<InstanceVersionResponse> {
    let Some(version) =
        get_latest_remote_version_optional_not_found(&attachment.id, state)
            .await?
    else {
        if shared_attachment_matches_current_user(attachment, state).await? {
            crate::state::clear_shared_instance(instance_id, &state.pool)
                .await?;
            emit_instance(instance_id, InstancePayloadType::Edited).await?;
            return Err(crate::ErrorKind::InputError(
                SHARED_INSTANCE_UNAVAILABLE_ERROR_CODE.to_string(),
            )
            .into());
        }

        return Err(crate::ErrorKind::OtherError(format!(
            "Shared instance {} was not found",
            attachment.id
        ))
        .into());
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

async fn shared_instance_install_data(
    shared_instance_id: &str,
    manager_id: Option<String>,
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
    let modpack_version_id = modpack.as_ref().map(|modpack| modpack.version_id.as_str());
    let modrinth_ids = version
        .modrinth_ids
        .into_iter()
        .filter(|id| Some(id.as_str()) != modpack_version_id)
        .collect();

    Ok(SharedInstanceInstallData {
        shared_instance_id: shared_instance_id.to_string(),
        manager_id,
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
    Ok(ModrinthCredentials::get_and_refresh(
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .map(|credentials| credentials.user_id))
}

async fn shared_instance_update_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let (current_version_ids, current_external_files) =
        current_shared_content(metadata, state).await?;
    let (latest_version_ids, latest_external_files) =
        remote_shared_content(version);
    let removed_disabled_project_ids = HashSet::new();
    let removed_disabled_external_files = HashSet::new();
    shared_content_diffs(
        &current_version_ids,
        &current_external_files,
        &latest_version_ids,
        &latest_external_files,
        &removed_disabled_project_ids,
        &removed_disabled_external_files,
        state,
    )
    .await
}

async fn shared_instance_publish_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let (latest_version_ids, latest_external_files) =
        remote_shared_content(version);
    let (current_version_ids, current_external_files) =
        current_publish_content(metadata, state).await?;
    let (removed_disabled_project_ids, removed_disabled_external_files) =
        current_publish_disabled_content(metadata, state).await?;
    shared_content_diffs(
        &latest_version_ids,
        &latest_external_files,
        &current_version_ids,
        &current_external_files,
        &removed_disabled_project_ids,
        &removed_disabled_external_files,
        state,
    )
    .await
}

async fn shared_content_diffs(
    current_version_ids: &[String],
    current_external_files: &HashSet<String>,
    latest_version_ids: &[String],
    latest_external_files: &HashSet<String>,
    removed_disabled_project_ids: &HashSet<String>,
    removed_disabled_external_files: &HashSet<String>,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let current_versions =
        shared_versions_by_project(current_version_ids, state).await?;
    let latest_versions =
        shared_versions_by_project(latest_version_ids, state).await?;
    let project_ids = current_versions
        .keys()
        .chain(latest_versions.keys())
        .cloned()
        .collect::<HashSet<_>>();
    let project_names = shared_project_names(&project_ids, state).await?;

    let mut diffs = Vec::new();
    for project_id in project_ids {
        let current = current_versions.get(&project_id);
        let latest = latest_versions.get(&project_id);
        let project_name = Some(
            project_names
                .get(&project_id)
                .cloned()
                .unwrap_or_else(|| project_id.clone()),
        );

        match (current, latest) {
            (None, Some(latest)) => {
                diffs.push(SharedInstanceUpdateDiff {
                    type_: SharedInstanceUpdateDiffType::Added,
                    project_id: Some(project_id),
                    project_name,
                    file_name: None,
                    current_version_name: None,
                    new_version_name: Some(latest.version_number.clone()),
                    disabled: false,
                });
            }
            (Some(current), None) => {
                let disabled =
                    removed_disabled_project_ids.contains(&project_id);
                diffs.push(SharedInstanceUpdateDiff {
                    type_: SharedInstanceUpdateDiffType::Removed,
                    project_id: Some(project_id),
                    project_name,
                    file_name: None,
                    current_version_name: Some(current.version_number.clone()),
                    new_version_name: None,
                    disabled,
                });
            }
            (Some(current), Some(latest)) if current.id != latest.id => {
                diffs.push(SharedInstanceUpdateDiff {
                    type_: SharedInstanceUpdateDiffType::Updated,
                    project_id: Some(project_id),
                    project_name,
                    file_name: None,
                    current_version_name: Some(current.version_number.clone()),
                    new_version_name: Some(latest.version_number.clone()),
                    disabled: false,
                });
            }
            _ => {}
        }
    }

    for file_name in latest_external_files.difference(current_external_files) {
        diffs.push(SharedInstanceUpdateDiff {
            type_: SharedInstanceUpdateDiffType::Added,
            project_id: None,
            project_name: None,
            file_name: Some(file_name.clone()),
            current_version_name: None,
            new_version_name: None,
            disabled: false,
        });
    }
    for file_name in current_external_files.difference(latest_external_files) {
        diffs.push(SharedInstanceUpdateDiff {
            type_: SharedInstanceUpdateDiffType::Removed,
            project_id: None,
            project_name: None,
            file_name: Some(file_name.clone()),
            current_version_name: None,
            new_version_name: None,
            disabled: removed_disabled_external_files.contains(file_name),
        });
    }

    diffs.sort_by(|a, b| {
        a.project_name
            .as_deref()
            .or(a.file_name.as_deref())
            .cmp(&b.project_name.as_deref().or(b.file_name.as_deref()))
    });
    Ok(diffs)
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

async fn shared_instance_install_modpack(
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Option<crate::install::SharedInstanceInstallModpack>> {
    let Some(modpack_id) = version.modpack_id.as_deref().filter(|id| !id.is_empty()) else {
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
        if entry.source_kind != crate::state::ContentSourceKind::SharedInstance
        {
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

        let is_modpack = item
            .version
            .as_ref()
            .is_some_and(|version| modpack_id.as_deref() == Some(version.id.as_str()));
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
    crate::state::mark_shared_instance_stale(instance_id, &state.pool).await
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
    let response = request_json::<InstanceVersionResponse>(
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
    )
    .await?;

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
        _ => None,
    }
}

async fn ensure_shareable_instance(
    instance_id: &str,
    state: &State,
) -> crate::Result<crate::state::InstanceMetadata> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    ensure_shareable_link(&metadata.link)?;
    Ok(metadata)
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

fn ensure_owner(attachment: &SharedInstanceAttachment) -> crate::Result<()> {
    if attachment.role == SharedInstanceRole::Owner {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Only the owner instance can manage shared instance users".to_string(),
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

async fn get_remote_users(
	shared_instance_id: &str,
	state: &State,
) -> crate::Result<SharedInstanceUsers> {
	let user_ids = request_json::<Vec<String>>(
		"get_instance_users",
		Method::GET,
		&format!("/instances/{shared_instance_id}/users"),
		None,
		state,
	)
	.await?;

	Ok(SharedInstanceUsers { user_ids })
}

async fn get_latest_remote_version(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<InstanceVersionResponse> {
    request_json(
        "get_latest_instance_version",
        Method::GET,
        &format!("/instances/{shared_instance_id}/versions"),
        None,
        state,
    )
    .await
}

async fn get_latest_remote_version_optional_not_found(
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<Option<InstanceVersionResponse>> {
    request_json_optional_not_found(
        "get_latest_instance_version",
        Method::GET,
        &format!("/instances/{shared_instance_id}/versions"),
        None,
        state,
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

async fn request_json_optional_not_found<T>(
    operation: &'static str,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    state: &State,
) -> crate::Result<Option<T>>
where
    T: DeserializeOwned,
{
    let response = send_request(operation, method.clone(), path, body, state)
        .await?;
    if response.status() == StatusCode::NOT_FOUND {
        tracing::warn!(
            operation,
            method = method.as_str(),
            path,
            status = response.status().as_u16(),
            "Shared instances API resource was not found"
        );
        return Ok(None);
    }

    if !response.status().is_success() {
        return shared_instances_request_error(
            operation,
            method,
            path,
            response,
        )
        .await;
    }

    decode_json_response(operation, path, response).await.map(Some)
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
    let credentials =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
            .ok_or(crate::ErrorKind::NoCredentialsError)?;
    let _permit = state.api_semaphore.0.acquire().await?;
    let base_url = service_base_url();
    let url = service_url(base_url, path);
    tracing::debug!(
        operation,
        method = method.as_str(),
        path,
        url = %url,
        user_id = %credentials.user_id,
        has_body = body.is_some(),
        "Sending shared instances API request"
    );
    let mut request = shared_instances_client(base_url)
        .request(method.clone(), &url)
        .bearer_auth(credentials.session);

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
