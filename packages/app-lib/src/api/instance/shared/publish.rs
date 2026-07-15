use super::client::*;
use super::diff::*;
use super::install::*;
use super::types::*;
use super::*;

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

pub(super) async fn detach_local_shared_instance(
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

pub(super) async fn remote_publish_content(
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

pub(super) async fn modpack_dependency_version_ids(
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

pub(super) async fn shared_instance_install_modpack(
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

pub(super) fn shared_instance_loader_version(
    loader_version: String,
) -> Option<String> {
    (!loader_version.is_empty()).then_some(loader_version)
}

pub(super) async fn current_shared_content(
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

pub(super) async fn current_publish_content(
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

pub(super) async fn current_publish_disabled_content(
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

pub(super) async fn shared_versions_by_project(
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

pub(super) async fn shared_project_names(
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

pub(super) fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

pub(super) fn enabled_file_name(file_name: &str) -> String {
    file_name
        .strip_suffix(".disabled")
        .unwrap_or(file_name)
        .to_string()
}

pub(super) fn shared_instance_name(name: String) -> String {
    match name.trim() {
        "" => "Shared instance".to_string(),
        name => name.to_string(),
    }
}

pub(super) async fn set_shared_instance_publish_status(
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

pub(super) async fn publish_shared_instance_inner(
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

pub(super) async fn publish_current_content(
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

pub(super) async fn collect_publish_content(
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

pub(super) fn shared_modpack_id(link: &InstanceLink) -> Option<String> {
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

pub(super) fn ensure_shareable_link(link: &InstanceLink) -> crate::Result<()> {
    if matches!(link, InstanceLink::ImportedModpack { .. }) {
        return Err(crate::ErrorKind::InputError(
            "You must unlink this modpack to share your instance".to_string(),
        )
        .into());
    }

    Ok(())
}

pub(super) async fn upload_external_files(
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
        let response = REQWEST_CLIENT
            .put(&upload.url)
            .body(bytes)
            .send()
            .await?;

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

pub(super) async fn mark_external_files_ready(
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

pub(super) async fn shared_attachment(
    instance_id: &str,
    state: &State,
) -> crate::Result<Option<SharedInstanceAttachment>> {
    Ok(crate::state::get_instance(instance_id, &state.pool)
        .await?
        .and_then(|metadata| metadata.shared_instance))
}

pub(crate) async fn sync_shared_instance_icon(
    instance_id: &str,
    icon_path: Option<&str>,
    state: &State,
) -> crate::Result<()> {
    let Some(attachment) = shared_attachment(instance_id, state).await? else {
        return Ok(());
    };
    if attachment.role != SharedInstanceRole::Owner {
        return Ok(());
    }

    update_remote_instance_icon(&attachment.id, icon_path, state).await
}

pub(super) async fn shared_instance_for_invites(
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
                crate::state::SharedInstanceAttachmentInput {
                    id: remote.id.clone(),
                    role: SharedInstanceRole::Owner,
                    manager_id: None,
                    server_manager_name: None,
                    server_manager_icon_url: None,
                    linked_user_id,
                    status: ContentSetSyncStatus::Unknown,
                    applied_version: None,
                    latest_version: None,
                },
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

    ensure_owner(&attachment)?;
    update_remote_instance_icon(
        &attachment.id,
        metadata.instance.icon_path.as_deref(),
        state,
    )
    .await?;

    Ok((metadata, attachment))
}

pub(super) fn ensure_owner(
    attachment: &SharedInstanceAttachment,
) -> crate::Result<()> {
    if attachment.role == SharedInstanceRole::Owner {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Only the owner instance can manage shared instance users".to_string(),
    )
    .into())
}

pub(super) fn ensure_member(
    attachment: &SharedInstanceAttachment,
) -> crate::Result<()> {
    if attachment.role.is_member() {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Only shared instance members can unlink from shared instances"
            .to_string(),
    )
    .into())
}

pub(super) fn file_type(project_type: ProjectType) -> String {
    project_type.get_name().to_string()
}
