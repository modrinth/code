use super::client::*;
use super::diff::*;
use super::install::*;
use super::types::*;
use super::*;
use async_walkdir::WalkDir;
use futures::StreamExt;
use sha2::Digest;

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
    config_paths: Vec<String>,
) -> crate::Result<SharedInstanceAttachment> {
    let state = State::get().await?;
    publish_shared_instance_inner(instance_id, &config_paths, &state).await?;
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

    let (version, snapshot) = tokio::try_join!(
        get_latest_remote_version_optional_unavailable(&attachment.id, &state,),
        collect_publish_snapshot(&metadata, &state),
    )?;
    let version = match version {
        SharedInstanceRemoteResponse::Available(version) => version,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            handle_unavailable_shared_instance_if_current_user(
                instance_id,
                &attachment,
                reason,
                &state,
            )
            .await?;
            return Err(shared_instance_unavailable_error(reason));
        }
    };
    let diffs =
        shared_instance_publish_diffs(&metadata, &version, &snapshot, &state)
            .await?;
    set_shared_instance_publish_status(
        instance_id,
        &attachment,
        version.version,
        !diffs.is_empty(),
        &state,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(Some(SharedInstancePublishPreview {
        shared_instance_id: attachment.id,
        latest_version: version.version,
        diffs,
        config_files: snapshot
            .config_files
            .into_iter()
            .map(|file| file.path)
            .collect(),
    }))
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
        Some(CacheBehaviour::MustRevalidate),
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
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(Some(crate::install::SharedInstanceInstallModpack {
        project_id: modpack_version.project_id,
        version_id: modpack_version.id,
        title: project
            .as_ref()
            .map(|project| project.title.clone())
            .unwrap_or(modpack_version.name),
        icon_url: project.and_then(|project| project.raw_icon_url),
    }))
}

pub(super) fn shared_instance_loader_version(
    loader_version: String,
) -> Option<String> {
    (!loader_version.is_empty()).then_some(loader_version)
}

pub(super) async fn current_shared_content(
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<(Vec<String>, BTreeSet<ExternalFileKey>)> {
    let entries = crate::state::instances::adapters::sqlite::content_rows::get_content_entries(
		&metadata.applied_content_set.id, &state.pool,
	).await?;
    let files = crate::state::instances::adapters::sqlite::content_rows::get_instance_files(
		&metadata.instance.id, &state.pool,
	).await?.into_iter().map(|file| (file.id.clone(), file)).collect::<HashMap<_, _>>();
    let unresolved_hashes = entries
        .iter()
        .filter(|entry| {
            entry.source_kind == ContentSourceKind::ModrinthModpack
                && entry.version_id.is_none()
        })
        .filter_map(|entry| entry.file_id.as_ref().and_then(|id| files.get(id)))
        .filter(|file| file.enabled && !file.missing)
        .map(|file| file.sha1.as_str())
        .collect::<Vec<_>>();
    let resolved_files = CachedEntry::get_file_many(
        &unresolved_hashes,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .into_iter()
    .map(|file| (file.hash.clone(), file))
    .collect::<HashMap<_, _>>();
    let mut version_ids = Vec::new();
    let mut external_files = BTreeSet::new();
    for entry in entries {
        if !entry.source_kind.is_shared_instance_managed() || !entry.enabled {
            continue;
        }
        let Some(file) = entry.file_id.as_ref().and_then(|id| files.get(id))
        else {
            continue;
        };
        if !file.enabled || file.missing {
            continue;
        }
        let version_id = entry.version_id.or_else(|| {
            (entry.source_kind == ContentSourceKind::ModrinthModpack)
                .then(|| {
                    resolved_files
                        .get(&file.sha1)
                        .map(|file| file.version_id.clone())
                })
                .flatten()
        });
        if let Some(version_id) = version_id {
            version_ids.push(version_id);
        } else {
            external_files.insert(ExternalFileKey {
                content_type: entry.project_type.into(),
                path: enabled_file_name(&file.file_name),
            });
        }
    }
    dedupe_strings(&mut version_ids);
    Ok((version_ids, external_files))
}

pub(super) struct CurrentPublishSnapshot {
    pub(super) version_ids: Vec<String>,
    pub(super) external_files: Vec<ExternalFileCandidate>,
    pub(super) disabled_project_ids: HashSet<String>,
    pub(super) disabled_version_ids: Vec<String>,
    pub(super) disabled_external_files: BTreeSet<ExternalFileKey>,
    pub(super) config_files: Vec<ConfigFile>,
    pub(super) removed_files: Vec<SharedInstanceRemovedFile>,
    pub(super) effective_version_ids: Vec<String>,
}

/// Cached metadata can outlive a deleted or hidden version. Only publish version
/// IDs that other members can resolve; retain installed files as uploads.
pub(super) async fn collect_publish_snapshot(
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<CurrentPublishSnapshot> {
    let instance_path = state
        .directories
        .instances_dir()
        .join(&metadata.instance.path);
    let (items, config_files) = if CONFIG_SYNC_ENABLED {
        tokio::try_join!(
            crate::state::list_content(
                &metadata.instance.id,
                None,
                None,
                state,
            ),
            collect_config_files(&instance_path),
        )?
    } else {
        (
            crate::state::list_content(
                &metadata.instance.id,
                None,
                None,
                state,
            )
            .await?,
            Vec::new(),
        )
    };
    let mut items = items;
    let modpack_id = shared_modpack_id(&metadata.link);
    let modpack_files = if let Some(modpack_id) = modpack_id.as_deref() {
        let (managed, files) = tokio::try_join!(
            crate::state::list_linked_modpack_content(
                &metadata.instance.id,
                None,
                None,
                state,
            ),
            shared_modpack_files(modpack_id, state),
        )?;
        items.extend(managed);
        files
    } else {
        Vec::new()
    };
    let mut seen_paths = HashSet::new();
    items.retain(|item| seen_paths.insert(item.file_path.clone()));
    let enabled_versions = items
        .iter()
        .filter(|item| item.enabled)
        .filter_map(|item| {
            item.version.as_ref().map(|version| version.id.as_str())
        })
        .collect::<HashSet<_>>();
    let enabled_paths = items
        .iter()
        .filter(|item| item.enabled)
        .map(|item| item.file_path.trim_end_matches(".disabled"))
        .collect::<HashSet<_>>();
    let mut removed_files = Vec::new();
    for file in &modpack_files {
        if let Some(version_id) = &file.version_id {
            if !enabled_versions.contains(version_id.as_str()) {
                let removed = SharedInstanceRemovedFile::Version {
                    version_id: version_id.clone(),
                };
                if !removed_files.contains(&removed) {
                    removed_files.push(removed);
                }
            }
        } else if !enabled_paths.contains(file.relative_path.as_str())
            && let Some((parent, filename)) = file.relative_path.split_once('/')
        {
            removed_files.push(SharedInstanceRemovedFile::Path {
                parent: parent.to_string(),
                filename: filename.to_string(),
            });
        }
    }
    let inherited_versions = modpack_files
        .iter()
        .filter_map(|file| file.version_id.as_deref())
        .collect::<HashSet<_>>();
    let mut effective_version_ids = Vec::new();
    let installed_version_ids = items
        .iter()
        .filter(|item| item.enabled)
        .filter_map(|item| {
            item.version.as_ref().map(|version| version.id.as_str())
        })
        .collect::<Vec<_>>();
    let available_version_ids = CachedEntry::get_version_many(
        &installed_version_ids,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .into_iter()
    .map(|version| version.id)
    .collect::<HashSet<_>>();
    let mut version_ids = Vec::new();
    let mut seen_version_ids = HashSet::new();
    let mut external_files = Vec::new();
    let mut seen_external_files = HashSet::new();
    let mut disabled_project_ids = HashSet::new();
    let mut disabled_version_ids = Vec::new();
    let mut seen_disabled_version_ids = HashSet::new();
    let mut disabled_external_files = BTreeSet::new();

    for item in items {
        if item.enabled {
            if let Some(version) = item.version.as_ref()
                && available_version_ids.contains(&version.id)
            {
                if seen_version_ids.insert(version.id.clone()) {
                    effective_version_ids.push(version.id.clone());
                    if !inherited_versions.contains(version.id.as_str()) {
                        version_ids.push(version.id.clone());
                    }
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
                    source: ExternalFileSource::InstanceFile(item.file_path),
                });
            }
            continue;
        }

        let is_modpack = item.version.as_ref().is_some_and(|version| {
            modpack_id.as_deref() == Some(version.id.as_str())
        });
        if is_modpack {
            continue;
        }

        if item.version.is_none()
            && let Some(project) = item.project.as_ref()
        {
            disabled_project_ids.insert(project.id.clone());
        }

        if let Some(version) = item.version {
            if seen_disabled_version_ids.insert(version.id.clone()) {
                disabled_version_ids.push(version.id);
            }
            continue;
        }

        if item.file_path.is_empty() {
            continue;
        }

        disabled_external_files.insert(ExternalFileKey {
            content_type: item.project_type.into(),
            path: enabled_file_name(&item.file_name),
        });
    }

    Ok(CurrentPublishSnapshot {
        version_ids,
        external_files,
        disabled_project_ids,
        disabled_version_ids,
        disabled_external_files,
        config_files,
        removed_files,
        effective_version_ids,
    })
}

pub(super) async fn shared_versions_by_id(
    version_ids: &[String],
    allow_missing: bool,
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

    let fetched_ids = versions
        .iter()
        .map(|version| version.id.as_str())
        .collect::<HashSet<_>>();
    if !allow_missing
        && let Some(missing) = version_ids
            .iter()
            .find(|id| !fetched_ids.contains(id.as_str()))
    {
        return Err(crate::ErrorKind::InputError(format!(
            "Shared content version {missing} was not found"
        ))
        .into());
    }
    Ok(versions
        .into_iter()
        .map(|version| (version.id.clone(), version))
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
    config_paths: &[String],
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

    let result = publish_current_content(
        instance_id,
        &attachment.id,
        config_paths,
        state,
    )
    .await;

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
    config_paths: &[String],
    state: &State,
) -> crate::Result<i32> {
    let _store_lease = state.content_store.lease().await;
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
    let snapshot = collect_publish_snapshot(&metadata, state).await?;
    let config_files = if CONFIG_SYNC_ENABLED {
        config_file_candidates(
            &metadata.instance.path,
            &snapshot.config_files,
            config_paths,
            state,
        )
        .await?
    } else {
        Vec::new()
    };
    let modrinth_ids = snapshot.version_ids;
    let mut external_files = snapshot.external_files;
    external_files.extend(config_files);
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
			"removed_files": snapshot.removed_files,
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
                handle_unavailable_shared_instance_if_current_user(
                    instance_id,
                    attachment,
                    reason,
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
    } else if !response.ready {
        tracing::debug!(
            "Shared instance version {} was not ready but had no external files",
            response.version
        );
    }

    Ok(response.version)
}

pub(super) async fn collect_config_files(
    instance_path: &std::path::Path,
) -> crate::Result<Vec<ConfigFile>> {
    let config_path = instance_path.join(CONFIG_DIRECTORY);
    crate::util::io::create_dir_all(&config_path).await?;
    let mut files = Vec::new();
    let mut walker = WalkDir::new(&config_path);

    while let Some(entry) = walker.next().await {
        let entry = entry.map_err(|error| {
            crate::ErrorKind::OtherError(format!(
                "Failed to read config directory: {error}"
            ))
        })?;
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let entry_path = entry.path();
        let relative_path = entry_path.strip_prefix(&config_path)?;
        let path = relative_path.to_string_lossy().replace('\\', "/");
        if !is_shareable_config_path(&format!("{CONFIG_DIRECTORY}/{path}")) {
            continue;
        }
        files.push(ConfigFile { path });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

/// Each selected config is uploaded as its own file under the instance root,
/// matching how linked servers publish configs.
async fn config_file_candidates(
    instance_path: &str,
    local_files: &[ConfigFile],
    selected_paths: &[String],
    state: &State,
) -> crate::Result<Vec<ExternalFileCandidate>> {
    let local_paths = local_files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<HashSet<_>>();
    let selected_paths = selected_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if let Some(missing) = selected_paths
        .iter()
        .find(|path| !local_paths.contains(*path))
    {
        return Err(crate::ErrorKind::InputError(format!(
            "Config file is unavailable for sharing: {missing}"
        ))
        .into());
    }
    ensure_config_file_count(&selected_paths)?;

    let config_path = state
        .directories
        .instances_dir()
        .join(instance_path)
        .join(CONFIG_DIRECTORY);
    let mut total_size = 0_u64;
    let mut candidates = Vec::with_capacity(selected_paths.len());
    for path in selected_paths {
        let source = config_path.join(path);
        let size = crate::util::io::metadata(&source).await?.len();
        total_size = total_size.saturating_add(size);
        if size > MAX_CONFIG_BUNDLE_FILE_SIZE
            || total_size > MAX_CONFIG_BUNDLE_TOTAL_SIZE
        {
            return Err(crate::ErrorKind::InputError(
                "Selected config files exceed the size limit for sharing"
                    .to_string(),
            )
            .into());
        }
        candidates.push(ExternalFileCandidate {
            file_name: format!("{CONFIG_DIRECTORY}/{path}"),
            file_type: CONFIG_FILE_TYPE.to_string(),
            source: ExternalFileSource::ConfigFile(source),
        });
    }

    Ok(candidates)
}

fn ensure_config_file_count(paths: &BTreeSet<&str>) -> crate::Result<()> {
    if paths.len() <= MAX_CONFIG_BUNDLE_ENTRIES {
        return Ok(());
    }

    let mut folder_entry_counts = HashMap::new();
    for path in paths {
        if let Some((folder, _)) = path.split_once('/') {
            *folder_entry_counts.entry(folder).or_insert(0_usize) += 1;
        }
    }

    if let Some((folder, count)) = folder_entry_counts
        .into_iter()
        .filter(|(_, count)| *count > MAX_CONFIG_BUNDLE_ENTRIES)
        .max_by_key(|(_, count)| *count)
    {
        return Err(crate::ErrorKind::InputError(format!(
			"The \"{folder}\" config folder has too many files to share ({count}; maximum {MAX_CONFIG_BUNDLE_ENTRIES}). Select fewer files from this folder."
		))
		.into());
    }

    Err(crate::ErrorKind::InputError(format!(
		"Too many config files were selected to share ({}; maximum {MAX_CONFIG_BUNDLE_ENTRIES}). Select fewer files.",
		paths.len()
	))
	.into())
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
        let path = match &candidate.source {
            ExternalFileSource::InstanceFile(file_path) => {
                let instance = crate::state::instances::adapters::sqlite::instance_rows::get_instance_by_path(instance_path, &state.pool).await?
					.ok_or_else(|| crate::state::content_store::input("Unknown instance"))?;
                let file = crate::state::instances::adapters::sqlite::content_rows::get_instance_file_by_relative_path(&instance.id, file_path, &state.pool).await?
					.ok_or_else(|| crate::state::content_store::input("Shared content file is not registered"))?;
                state.content_store.read_path(&file, instance_path).await?
            }
            ExternalFileSource::ConfigFile(path) => {
                crate::state::content_store::ReadableContent::Local(
                    path.clone(),
                )
            }
        };
        let upload_url_string = upload.url.as_deref().filter(|url| !url.is_empty()).ok_or_else(|| {
			crate::ErrorKind::InputError(format!("Shared instance external file {} is missing its upload URL", upload.file_name))
		})?;
        let upload_url =
            url::Url::parse(upload_url_string).map_err(|error| {
                crate::ErrorKind::OtherError(format!(
                    "Invalid shared instance external file upload URL: {error}"
                ))
            })?;
        let mut file = tokio::fs::File::open(path.path()).await?;
        let mut hasher = sha2::Sha512::new();
        let mut buffer = vec![0_u8; 64 * 1024];
        let mut size = 0_u64;
        loop {
            use tokio::io::AsyncReadExt;
            let read = file.read(&mut buffer).await?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            size += read as u64;
        }
        let file_sha512 = format!("{:x}", hasher.finalize());
        use tokio::io::AsyncSeekExt;
        file.rewind().await?;
        let body =
            reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(file));
        let response = send_body_request_to_url(
            "upload_external_file",
            Method::PUT,
            upload_url.path(),
            upload_url_string,
            body,
            Some(size),
            Some(&file_sha512),
            state,
        )
        .await?;
        if !response.status().is_success() {
            return shared_instances_request_error(
                "upload_external_file",
                Method::PUT,
                upload_url.path(),
                response,
            )
            .await;
        }
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
            publish_shared_instance_inner(instance_id, &[], state).await?;
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
