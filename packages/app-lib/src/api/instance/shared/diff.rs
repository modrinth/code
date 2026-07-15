use super::client::*;
use super::publish::*;
use super::types::*;
use super::*;

#[derive(Clone, Copy)]
enum ConfigDiffDirection {
    Update,
    Publish,
}

fn shared_config_diffs(
    local: &[ConfigFile],
    remote: &[ConfigFile],
    direction: ConfigDiffDirection,
) -> Vec<SharedInstanceUpdateDiff> {
    let (current, latest) = match direction {
        ConfigDiffDirection::Update => (local, remote),
        ConfigDiffDirection::Publish => (remote, local),
    };
    let current = current
        .iter()
        .map(|file| (file.path.clone(), file.hash.clone()))
        .collect::<HashMap<_, _>>();
    let latest = latest
        .iter()
        .map(|file| (file.path.clone(), file.hash.clone()))
        .collect::<HashMap<_, _>>();
    let changed_file_count = latest
        .iter()
        .filter(|(path, hash)| current.get(*path) != Some(*hash))
        .count()
        + current
            .keys()
            .filter(|path| !latest.contains_key(*path))
            .count();
    if changed_file_count == 0 {
        return Vec::new();
    }

    vec![SharedInstanceUpdateDiff {
        type_: SharedInstanceUpdateDiffType::ConfigFilesUpdated,
        project_id: None,
        project_name: None,
        file_name: None,
        current_version_name: None,
        new_version_name: None,
        config_file_count: Some(changed_file_count),
        disabled: false,
    }]
}

async fn remote_config_files(
    version: &InstanceVersionResponse,
) -> crate::Result<Vec<ConfigFile>> {
    let Some(bundle) = version
        .external_files
        .iter()
        .find(|file| file.file_type == CONFIG_BUNDLE_FILE_TYPE)
    else {
        return Ok(Vec::new());
    };
    let metadata = bundle.metadata.clone().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Shared instance config bundle is missing file metadata"
                .to_string(),
        )
    })?;
    let mut files = serde_json::from_value::<Vec<ConfigFile>>(metadata)?;
    files.retain(|file| {
        !is_excluded_config_path(std::path::Path::new(&file.path))
    });
    Ok(files)
}

pub(super) async fn shared_instance_update_diffs(
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
    if CONFIG_SYNC_ENABLED {
        let instance_path = state
            .directories
            .instances_dir()
            .join(&metadata.instance.path);
        let (local_config_files, remote_config_files) = tokio::try_join!(
            collect_config_files(&instance_path),
            remote_config_files(version),
        )?;
        diffs.extend(shared_config_diffs(
            &local_config_files,
            &remote_config_files,
            ConfigDiffDirection::Update,
        ));
    }

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

pub(super) async fn shared_instance_publish_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    snapshot: &CurrentPublishSnapshot,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let remote_modpack_id =
        version.modpack_id.as_deref().filter(|id| !id.is_empty());
    let current_modpack_id = shared_modpack_id(&metadata.link);
    let modpack_unlinked =
        remote_modpack_id.is_some() && current_modpack_id.is_none();
    let disabled_versions = async {
        if snapshot.disabled_version_ids.is_empty() {
            Ok(HashMap::new())
        } else {
            shared_versions_by_project(&snapshot.disabled_version_ids, state)
                .await
        }
    };
    let ((latest_version_ids, latest_external_files), disabled_versions) =
        tokio::try_join!(
        remote_publish_content(version, modpack_unlinked, state),
        disabled_versions,
    )?;
    let current_external_files = snapshot
        .external_files
        .iter()
        .map(|file| file.file_name.clone())
        .collect::<HashSet<_>>();
    let current_version_ids = snapshot
        .version_ids
        .iter()
        .filter(|id| current_modpack_id.as_deref() != Some(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let mut removed_disabled_project_ids =
        snapshot.disabled_project_ids.clone();
    removed_disabled_project_ids.extend(disabled_versions.into_keys());
    let mut diffs = shared_content_diffs(
        &latest_version_ids,
        &latest_external_files,
        &current_version_ids,
        &current_external_files,
        &removed_disabled_project_ids,
        &snapshot.disabled_external_files,
        false,
        state,
    )
    .await?;
    if CONFIG_SYNC_ENABLED {
        diffs.extend(shared_config_diffs(
            &snapshot.config_files,
            &remote_config_files(version).await?,
            ConfigDiffDirection::Publish,
        ));
    }

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

pub(super) async fn shared_instance_configuration_diffs(
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
                    config_file_count: None,
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

pub(super) fn configuration_diff(
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
        config_file_count: None,
        disabled: false,
    }
}

pub(super) async fn shared_modpack_version_label(
    version_id: Option<&str>,
    state: &State,
) -> Option<String> {
    let version_id = version_id?;
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

pub(super) fn normalized_loader_version(
    loader_version: Option<&str>,
) -> Option<&str> {
    loader_version.filter(|version| !version.is_empty())
}

pub(super) fn shared_loader_label(
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

pub(super) async fn shared_content_diffs(
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
                    config_file_count: None,
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
                    config_file_count: None,
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

pub(super) fn shared_update_diff_type(
    kind: ContentSetDiffKind,
) -> SharedInstanceUpdateDiffType {
    match kind {
        ContentSetDiffKind::Added => SharedInstanceUpdateDiffType::Added,
        ContentSetDiffKind::Removed => SharedInstanceUpdateDiffType::Removed,
        ContentSetDiffKind::Updated => SharedInstanceUpdateDiffType::Updated,
    }
}

pub(super) async fn shared_content_snapshot(
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

pub(super) fn remote_shared_content(
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
            .filter(|file| file.file_type != CONFIG_BUNDLE_FILE_TYPE)
            .map(|file| file.file_name.clone())
            .collect(),
    )
}
