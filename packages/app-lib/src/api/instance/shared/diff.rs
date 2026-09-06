use super::client::*;
use super::publish::*;
use super::types::*;
use super::*;

struct SharedContentSnapshot {
    version_ids: Vec<String>,
    external_files: BTreeSet<ExternalFileKey>,
    configuration: ContentSetConfiguration,
}

pub(super) async fn shared_instance_update_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let before_configuration = local_configuration(metadata);
    let after_configuration = remote_configuration(version);
    let modpack_unlinked = before_configuration.modpack_version_id.is_some()
        && after_configuration.modpack_version_id.is_none();
    let (version_ids, external_files) =
        current_shared_content(metadata, modpack_unlinked, state).await?;
    let before = SharedContentSnapshot {
        version_ids,
        external_files,
        configuration: before_configuration,
    };
    let (version_ids, external_files) = remote_shared_content(version)?;
    let after = SharedContentSnapshot {
        version_ids,
        external_files,
        configuration: after_configuration,
    };

    shared_content_diffs(
        &before,
        &after,
        &HashSet::new(),
        &BTreeSet::new(),
        CommonExternalFilePolicy::AssumeUpdated,
        state,
    )
    .await
}

pub(super) async fn shared_instance_publish_diffs(
    metadata: &crate::state::InstanceMetadata,
    version: &InstanceVersionResponse,
    snapshot: &CurrentPublishSnapshot,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let before_configuration = remote_configuration(version);
    let after_configuration = local_configuration(metadata);
    let modpack_unlinked = before_configuration.modpack_version_id.is_some()
        && after_configuration.modpack_version_id.is_none();
    let disabled_versions = async {
        if snapshot.disabled_version_ids.is_empty() {
            Ok(HashMap::new())
        } else {
            shared_versions_by_project(&snapshot.disabled_version_ids, state)
                .await
        }
    };
    let ((version_ids, external_files), disabled_versions) = tokio::try_join!(
        remote_publish_content(version, modpack_unlinked, state),
        disabled_versions,
    )?;
    let before = SharedContentSnapshot {
        version_ids,
        external_files,
        configuration: before_configuration,
    };
    let after = SharedContentSnapshot {
        version_ids: snapshot
            .version_ids
            .iter()
            .filter(|id| {
                after_configuration.modpack_version_id.as_deref()
                    != Some(id.as_str())
            })
            .cloned()
            .collect(),
        external_files: snapshot
            .external_files
            .iter()
            .map(|file| {
                shared_external_file_key(&file.file_type, &file.file_name)
            })
            .collect::<crate::Result<_>>()?,
        configuration: after_configuration,
    };
    let mut removed_disabled_project_ids =
        snapshot.disabled_project_ids.clone();
    removed_disabled_project_ids.extend(disabled_versions.into_keys());

    shared_content_diffs(
        &before,
        &after,
        &removed_disabled_project_ids,
        &snapshot.disabled_external_files,
        CommonExternalFilePolicy::AssumeUnchanged,
        state,
    )
    .await
}

fn local_configuration(
    metadata: &crate::state::InstanceMetadata,
) -> ContentSetConfiguration {
    ContentSetConfiguration {
        modpack_version_id: shared_modpack_id(&metadata.link)
            .filter(|id| !id.is_empty()),
        game_version: metadata.applied_content_set.game_version.clone(),
        loader: LoaderReference {
            name: metadata.applied_content_set.loader.as_str().to_string(),
            version: metadata.applied_content_set.loader_version.clone(),
        },
    }
}

fn remote_configuration(
    version: &InstanceVersionResponse,
) -> ContentSetConfiguration {
    ContentSetConfiguration {
        modpack_version_id: version
            .modpack_id
            .clone()
            .filter(|id| !id.is_empty()),
        game_version: version.game_version.clone(),
        loader: LoaderReference {
            name: version.loader.as_str().to_string(),
            version: Some(version.loader_version.clone()),
        },
    }
}

async fn shared_configuration_diffs(
    changes: Vec<ConfigurationDiff>,
    state: &State,
) -> Vec<SharedInstanceUpdateDiff> {
    let mut diffs = Vec::new();
    for change in changes {
        match change {
            ConfigurationDiff::Modpack(Change::Added { after }) => {
                diffs.push(configuration_diff(
                    SharedInstanceUpdateDiffType::ModpackLinked,
                    None,
                    shared_modpack_version_label(Some(&after), state).await,
                ))
            }
            ConfigurationDiff::Modpack(Change::Removed { before }) => diffs
                .push(configuration_diff(
                    SharedInstanceUpdateDiffType::ModpackUnlinked,
                    shared_modpack_version_label(Some(&before), state).await,
                    None,
                )),
            ConfigurationDiff::Modpack(Change::Updated { before, after }) => {
                let current =
                    shared_modpack_version_details(&before, state).await;
                let new = shared_modpack_version_details(&after, state).await;
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
            ConfigurationDiff::GameVersion(change) => {
                diffs.push(configuration_diff(
                    SharedInstanceUpdateDiffType::GameVersionUpdated,
                    change.before().cloned(),
                    change.after().cloned(),
                ))
            }
            ConfigurationDiff::Loader(change) => {
                diffs.push(configuration_diff(
                    SharedInstanceUpdateDiffType::LoaderUpdated,
                    change.before().map(shared_loader_label),
                    change.after().map(shared_loader_label),
                ))
            }
        }
    }
    diffs
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

fn shared_loader_label(loader: &LoaderReference) -> String {
    let loader_name = match loader.name.as_str() {
        "vanilla" => "Vanilla",
        "forge" => "Forge",
        "fabric" => "Fabric",
        "quilt" => "Quilt",
        "neoforge" => "NeoForge",
        name => name,
    };
    match loader.version.as_deref() {
        Some(version) => format!("{loader_name} {version}"),
        None => loader_name.to_string(),
    }
}

async fn shared_content_diffs(
    before: &SharedContentSnapshot,
    after: &SharedContentSnapshot,
    removed_disabled_project_ids: &HashSet<String>,
    removed_disabled_external_files: &BTreeSet<ExternalFileKey>,
    common_external_files: CommonExternalFilePolicy,
    state: &State,
) -> crate::Result<Vec<SharedInstanceUpdateDiff>> {
    let (before_versions, after_versions) = tokio::try_join!(
        shared_versions_by_project(&before.version_ids, state),
        shared_versions_by_project(&after.version_ids, state),
    )?;
    let to_snapshot =
        |source: &SharedContentSnapshot,
         versions: &HashMap<String, crate::state::Version>| {
            ContentSetSnapshot {
                projects: versions
                    .iter()
                    .map(|(project_id, version)| {
                        (project_id.clone(), version.id.clone())
                    })
                    .collect(),
                external_files: source.external_files.clone(),
            }
        };
    let diff = diff_content_sets(
        &to_snapshot(before, &before_versions),
        &to_snapshot(after, &after_versions),
        &ContentSetDiffOptions {
            common_external_files,
        },
    )
    .with_additional(diff_configuration(
        &before.configuration,
        &after.configuration,
    ));
    if !diff.has_changes() {
        return Ok(Vec::new());
    }
    let project_ids = diff
        .content
        .iter()
        .filter_map(|entry| match entry {
            ContentSetDiffEntry::Project { project_id, .. } => {
                Some(project_id.clone())
            }
            ContentSetDiffEntry::ExternalFile { .. } => None,
        })
        .collect::<HashSet<_>>();
    let project_names = shared_project_names(&project_ids, state).await?;
    let mut content_diffs = Vec::new();
    for entry in diff.content {
        match entry {
            ContentSetDiffEntry::Project { project_id, change } => {
                let disabled = change.kind() == ContentSetDiffKind::Removed
                    && removed_disabled_project_ids.contains(&project_id);
                content_diffs.push(SharedInstanceUpdateDiff {
                    type_: shared_update_diff_type(change.kind()),
                    project_name: Some(
                        project_names
                            .get(&project_id)
                            .cloned()
                            .unwrap_or_else(|| project_id.clone()),
                    ),
                    current_version_name: change.before().map(|id| {
                        before_versions
                            .get(&project_id)
                            .map(|version| version.version_number.clone())
                            .unwrap_or_else(|| id.clone())
                    }),
                    new_version_name: change.after().map(|id| {
                        after_versions
                            .get(&project_id)
                            .map(|version| version.version_number.clone())
                            .unwrap_or_else(|| id.clone())
                    }),
                    project_id: Some(project_id),
                    file_name: None,
                    config_file_count: None,
                    disabled,
                });
            }
            ContentSetDiffEntry::ExternalFile { file, kind } => {
                let disabled = kind == ContentSetDiffKind::Removed
                    && removed_disabled_external_files.contains(&file);
                content_diffs.push(SharedInstanceUpdateDiff {
                    type_: shared_update_diff_type(kind),
                    project_id: None,
                    project_name: None,
                    file_name: Some(file.path),
                    current_version_name: None,
                    new_version_name: None,
                    config_file_count: None,
                    disabled,
                });
            }
        }
    }
    content_diffs.sort_by(|a, b| {
        a.project_name
            .as_deref()
            .or(a.file_name.as_deref())
            .cmp(&b.project_name.as_deref().or(b.file_name.as_deref()))
    });
    let mut diffs = shared_configuration_diffs(diff.additional, state).await;
    diffs.extend(content_diffs);
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

pub(super) fn shared_external_file_key(
    file_type: &str,
    path: &str,
) -> crate::Result<ExternalFileKey> {
    Ok(ExternalFileKey {
        content_type: file_type.parse().map_err(
            |error: modrinth_content_management::Error| {
                crate::ErrorKind::InputError(error.to_string())
            },
        )?,
        path: path.to_string(),
    })
}

fn remote_shared_content(
    version: &InstanceVersionResponse,
) -> crate::Result<(Vec<String>, BTreeSet<ExternalFileKey>)> {
    let mut version_ids = version.modrinth_ids.clone();
    if let Some(modpack_id) = version.modpack_id.as_deref() {
        version_ids.retain(|id| id != modpack_id);
    }
    dedupe_strings(&mut version_ids);
    let external_files = version
        .external_files
        .iter()
        .filter(|file| file.file_type != CONFIG_BUNDLE_FILE_TYPE)
        .map(|file| shared_external_file_key(&file.file_type, &file.file_name))
        .collect::<crate::Result<_>>()?;
    Ok((version_ids, external_files))
}
