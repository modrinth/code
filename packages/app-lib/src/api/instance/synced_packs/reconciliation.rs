use super::storage::{
    cache_bytes, read_bytes, read_cached_bytes, read_library, write_library,
};
use super::{
    PackLibrary, PackPlacement, SyncedPack, pack_option, pack_path, same_path,
    version_compatible,
};
use crate::event::{InstancePayloadType, emit::emit_instance};
use crate::state::instances::commands;
use crate::state::{
    CacheBehaviour, CachedEntry, ContentItem, ContentItemVersion,
    ContentSourceKind, InstanceMetadata, ProjectType, State, SyncedOption,
    SyncedPackInfo, Version,
};
use crate::util::fetch;
use modrinth_content_management::ResolutionPreferences;
use std::collections::BTreeMap;

use super::super::synced_options::{
    GlobalSyncedOptions, get_global_options, instance_dir, instance_is_running,
    instance_option_enabled, sync_files_are_protected,
};

pub(super) fn participating(
    metadata: &InstanceMetadata,
    pack: &SyncedPack,
    global: GlobalSyncedOptions,
) -> bool {
    pack_option(pack.item.project_type).is_ok_and(|option| {
        global.get(option) && instance_option_enabled(metadata, option)
    })
}

fn local(item: &ContentItem) -> bool {
    item.source_kind
        .is_none_or(|kind| kind == ContentSourceKind::Local)
}

fn current_item<'a>(
    items: &'a [ContentItem],
    pack: &SyncedPack,
    placement: &PackPlacement,
) -> Option<&'a ContentItem> {
    items
        .iter()
        .find(|item| {
            item.project_type == pack.item.project_type
                && same_path(&item.file_path, &placement.path)
                && pack.item.project.as_ref().is_none_or(|project| {
                    item.project
                        .as_ref()
                        .is_some_and(|candidate| candidate.id == project.id)
                })
        })
        .or_else(|| {
            items.iter().find(|item| {
                item.project_type == pack.item.project_type
                    && pack.item.project.as_ref().is_some_and(|project| {
                        item.project
                            .as_ref()
                            .is_some_and(|candidate| candidate.id == project.id)
                    })
            })
        })
}

async fn capture_items(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    state: &State,
    placements: BTreeMap<String, PackPlacement>,
    global: GlobalSyncedOptions,
    items: &[ContentItem],
) -> crate::Result<bool> {
    let mut shared_changed = false;
    for (id, mut placement) in placements {
        let Some(pack) = library.packs.get(&id).cloned() else {
            continue;
        };
        if placement.excluded
            || placement.suspended
            || placement.pending
            || placement.path.is_empty()
            || placement.content_set_id != metadata.applied_content_set.id
            || !participating(metadata, &pack, global)
            || pack.item.enabled != placement.enabled
        {
            continue;
        }
        if let Some(item) = current_item(&items, &pack, &placement) {
            if !local(item) {
                continue;
            }
            if item.enabled != placement.enabled {
                if let Some(pack) = library.packs.get_mut(&id) {
                    pack.item.enabled = item.enabled;
                }
                shared_changed = true;
            }
            if pack.item.project_type == ProjectType::ResourcePack
                && placement.resource_pack_selection_path.is_none()
            {
                placement.resource_pack_selection_path =
                    Some(placement.path.clone());
            }
            placement.path = item.file_path.clone();
            placement.sha1 = item.id.clone();
            placement.enabled = item.enabled;
        } else if !instance_dir(metadata, state).join(&placement.path).exists()
        {
            placement.excluded = true;
        }
        library
            .instances
            .entry(metadata.instance.id.clone())
            .or_default()
            .insert(id, placement);
    }
    match super::selection::capture(metadata, library, state).await {
        Ok(changed) => shared_changed |= changed.unwrap_or(false),
        Err(error) => tracing::warn!(
            "Could not capture resource-pack selection for {}: {error}",
            metadata.instance.id
        ),
    }
    Ok(shared_changed)
}

async fn owns_file(
    metadata: &InstanceMetadata,
    placement: &PackPlacement,
    state: &State,
) -> crate::Result<bool> {
    if placement.path.is_empty() {
        return Ok(false);
    }
    let path = instance_dir(metadata, state).join(&placement.path);
    if !path.exists() {
        return Ok(false);
    }
    let kind = commands::content_source_kind_for_project_path(
        &metadata.instance.id,
        &placement.path,
        state,
    )
    .await?;
    if kind.is_some_and(|kind| kind != ContentSourceKind::Local) {
        return Ok(false);
    }
    let (_, hash) = fetch::sha1_file_async(&path).await?;
    Ok(hash == placement.sha1)
}

async fn toggle_pack(
    metadata: &InstanceMetadata,
    item: &ContentItem,
    enabled: bool,
    state: &State,
) -> crate::Result<String> {
    let path = format!(
        "{}{}",
        item.file_path.trim_end_matches(".disabled"),
        if enabled { "" } else { ".disabled" },
    );
    if path != item.file_path
        && instance_dir(metadata, state).join(&path).exists()
    {
        return Err(crate::ErrorKind::InputError(
            "Another pack already uses this file name in the instance."
                .to_string(),
        )
        .into());
    }
    commands::toggle_disable_project(
        &metadata.instance.id,
        &item.file_path,
        Some(enabled),
        state,
    )
    .await
}

#[derive(Default)]
struct PreparedPack {
    compatible: bool,
    version: Option<Version>,
    contents: Option<(bytes::Bytes, String)>,
    dependencies: Vec<commands::DownloadedProjectVersion>,
    conflict: bool,
    deferred: bool,
    owned_previous: bool,
}

fn matching_item<'a>(
    items: &'a [ContentItem],
    pack: &SyncedPack,
    previous: Option<&PackPlacement>,
) -> Option<&'a ContentItem> {
    previous
        .filter(|placement| !placement.path.is_empty())
        .and_then(|placement| current_item(items, pack, placement))
        .or_else(|| {
            items.iter().find(|item| {
                local(item)
                    && item.id == pack.sha1
                    && item.project_type == pack.item.project_type
            })
        })
}

fn local_conflict(
    items: &[ContentItem],
    pack: &SyncedPack,
    previous: Option<&PackPlacement>,
    target_path: &str,
) -> bool {
    previous.is_none_or(|placement| placement.path.is_empty())
        && matching_item(items, pack, previous).is_none()
        && items.iter().any(|item| {
            same_path(&item.file_path, target_path)
                || pack.item.project.as_ref().is_some_and(|project| {
                    item.project
                        .as_ref()
                        .is_some_and(|candidate| candidate.id == project.id)
                })
        })
}

async fn prepare_pack(
    metadata: &InstanceMetadata,
    id: &str,
    pack: &SyncedPack,
    library: &PackLibrary,
    items: &[ContentItem],
    state: &State,
) -> crate::Result<PreparedPack> {
    let mut prepared = PreparedPack::default();
    let previous = library
        .instances
        .get(&metadata.instance.id)
        .and_then(|placements| placements.get(id));
    if local_conflict(
        items,
        pack,
        previous,
        &pack_path(pack, &pack.item.file_name),
    ) {
        prepared.conflict = true;
        return Ok(prepared);
    }
    prepared.owned_previous = if let Some(previous) = previous {
        owns_file(metadata, previous, state).await?
    } else {
        false
    };
    prepared.version = if let Some(project) = &pack.item.project {
        let versions = CachedEntry::get_project_versions(
			&project.id,
			Some(CacheBehaviour::MustRevalidate),
			&state.pool,
			&state.api_semaphore,
		).await?.ok_or_else(|| {
			crate::ErrorKind::InputError(
				"Pack versions are temporarily unavailable; keeping the installed pack.".to_owned(),
			)
		})?;
        versions
            .into_iter()
            .filter(|version| {
                version_compatible(pack, version, metadata)
                    && !version.files.is_empty()
            })
            .max_by_key(|version| version.date_published)
    } else {
        None
    };
    if (pack.item.project.is_some() && prepared.version.is_none())
        || (pack.item.project.is_none()
            && !pack
                .game_versions
                .contains(&metadata.applied_content_set.game_version))
    {
        return Ok(prepared);
    }
    let file = prepared.version.as_ref().and_then(|version| {
        version
            .files
            .iter()
            .find(|file| file.primary)
            .or_else(|| version.files.first())
    });
    let file_name = file
        .map_or(pack.item.file_name.as_str(), |file| file.filename.as_str());
    if !path_util::is_safe_file_name(file_name) {
        return Err(crate::ErrorKind::InputError(
            "Invalid pack filename.".to_owned(),
        )
        .into());
    }
    if local_conflict(items, pack, previous, &pack_path(pack, file_name)) {
        prepared.conflict = true;
        return Ok(prepared);
    }
    let contents = if let Some(file) = file {
        let cached = if let Some(sha1) = file.hashes.get("sha1") {
            read_cached_bytes(sha1, state)
                .await?
                .map(|bytes| (bytes, sha1.clone()))
        } else {
            None
        };
        if let Some(cached) = cached {
            cached
        } else {
            let bytes = fetch::fetch(
                &file.url,
                file.hashes.get("sha1").map(String::as_str),
                None,
                None,
                &state.fetch_semaphore,
                &state.pool,
            )
            .await?;
            let sha1 = cache_bytes(bytes.clone(), state).await?;
            (bytes, sha1)
        }
    } else {
        (read_bytes(pack, state).await?, pack.sha1.clone())
    };
    let bytes = contents.0.clone();
    let project_type = pack.item.project_type;
    tokio::task::spawn_blocking(move || {
        super::operations::validate_pack(&bytes, project_type)
    })
    .await??;
    prepared.contents = Some(contents);
    if let (Some(project), Some(version)) =
        (&pack.item.project, &prepared.version)
        && !version.dependencies.is_empty()
    {
        let plan = commands::resolve_install_plan(
            &metadata.instance.id,
            commands::InstanceInstallProjectRequest {
                project_id: project.id.clone(),
                version_id: Some(version.id.clone()),
                content_type: pack.item.project_type.into(),
                selected: ResolutionPreferences::default(),
            },
            state,
        )
        .await?;
        if !plan.dependencies.is_empty()
            && instance_is_running(metadata, state).await?
        {
            prepared.deferred = true;
            return Ok(prepared);
        }
        for dependency in plan.dependencies {
            prepared.dependencies.push(
                commands::download_project_version(
                    &metadata.instance.id,
                    &dependency.version_id,
                    fetch::DownloadReason::Dependency,
                    dependency.dependent_on_version_id,
                    state,
                )
                .await?,
            );
        }
    }
    Ok(prepared)
}

async fn apply_pack(
    metadata: &InstanceMetadata,
    id: &str,
    pack: &SyncedPack,
    library: &mut PackLibrary,
    items: &mut Vec<ContentItem>,
    state: &State,
    mut prepared: PreparedPack,
) -> crate::Result<()> {
    let instance_id = &metadata.instance.id;
    let previous = library
        .instances
        .get(instance_id)
        .and_then(|items| items.get(id))
        .cloned();
    if prepared.conflict {
        let message = "A local copy of this pack already exists. Sync it from the content tab to include it.";
        let placement = library
            .instances
            .entry(instance_id.clone())
            .or_default()
            .entry(id.to_owned())
            .or_default();
        if placement.error.as_deref() != Some(message) {
            tracing::warn!(
                "Could not sync pack {id} to {instance_id}: {message}"
            );
            placement.error = Some(message.to_owned());
        }
        return Ok(());
    }
    let resource_pack_selection_path =
        previous.as_ref().and_then(|placement| {
            placement.resource_pack_selection_path.clone().or_else(|| {
                (!placement.path.is_empty()).then(|| placement.path.clone())
            })
        });
    let resource_pack_selection_pending = previous
        .as_ref()
        .is_some_and(|placement| placement.resource_pack_selection_pending);
    if previous
        .as_ref()
        .is_some_and(|placement| placement.excluded)
    {
        return Ok(());
    }
    let matching = previous
        .as_ref()
        .filter(|placement| !placement.path.is_empty())
        .and_then(|placement| current_item(items, pack, placement))
        .or_else(|| {
            items.iter().find(|item| {
                local(item)
                    && item.id == pack.sha1
                    && item.project_type == pack.item.project_type
            })
        })
        .cloned();
    if matching.as_ref().is_some_and(|item| !local(item)) {
        return Err(crate::ErrorKind::InputError(
            "This pack is managed in this instance.".to_string(),
        )
        .into());
    }
    if let Some(item) = matching {
        let is_source = previous
            .as_ref()
            .is_some_and(|placement| placement.is_source);
        let compatible = is_source || prepared.compatible;
        if compatible || item.locked {
            let changed = item.enabled != pack.item.enabled;
            let joined = previous
                .as_ref()
                .is_none_or(|placement| placement.suspended);
            let path = if changed {
                toggle_pack(metadata, &item, pack.item.enabled, state).await?
            } else {
                item.file_path.clone()
            };
            library
                .instances
                .entry(instance_id.clone())
                .or_default()
                .insert(
                    id.to_string(),
                    PackPlacement {
                        is_source,
                        path: path.clone(),
                        sha1: item.id.clone(),
                        enabled: pack.item.enabled,
                        pending: !compatible,
                        content_set_id: metadata.applied_content_set.id.clone(),
                        resource_pack_selection_path,
                        resource_pack_selection_pending,
                        ..Default::default()
                    },
                );
            if let Some(cached) = items
                .iter_mut()
                .find(|cached| cached.file_path == item.file_path)
            {
                cached.file_path = path;
                cached.enabled = pack.item.enabled;
            }
            if changed || joined {
                emit_instance(instance_id, InstancePayloadType::Synced).await?;
            }
            return Ok(());
        }
    }
    let version = prepared.version.take();
    let compatible = if pack.item.project.is_some() {
        version.is_some()
    } else {
        pack.game_versions
            .contains(&metadata.applied_content_set.game_version)
    };
    if !compatible {
        if let Some(previous) = &previous
            && prepared.owned_previous
        {
            commands::remove_project(instance_id, &previous.path, state)
                .await?;
            items.retain(|item| item.file_path != previous.path);
            emit_instance(instance_id, InstancePayloadType::Synced).await?;
        }
        library
            .instances
            .entry(instance_id.clone())
            .or_default()
            .insert(
                id.to_string(),
                PackPlacement {
                    enabled: pack.item.enabled,
                    content_set_id: metadata.applied_content_set.id.clone(),
                    resource_pack_selection_path,
                    resource_pack_selection_pending,
                    ..Default::default()
                },
            );
        return Ok(());
    }
    let file = version.as_ref().and_then(|version| {
        version
            .files
            .iter()
            .find(|file| file.primary)
            .or_else(|| version.files.first())
    });
    let file_name = file
        .map_or(pack.item.file_name.as_str(), |file| file.filename.as_str());
    if !path_util::is_safe_file_name(file_name) {
        return Err(crate::ErrorKind::InputError(
            "Invalid pack filename.".to_string(),
        )
        .into());
    }
    let target_path = pack_path(pack, file_name);
    if previous
        .as_ref()
        .is_none_or(|placement| placement.path.is_empty())
        && items.iter().any(|item| {
            same_path(&item.file_path, &target_path)
                || pack.item.project.as_ref().is_some_and(|project| {
                    item.project
                        .as_ref()
                        .is_some_and(|candidate| candidate.id == project.id)
                })
        })
    {
        return Err(crate::ErrorKind::InputError(
			"A local copy of this pack already exists. Sync it from the content tab to include it.".to_string(),
		).into());
    }
    let target_base = target_path.trim_end_matches(".disabled");
    for path in [target_base.to_string(), format!("{target_base}.disabled")] {
        if instance_dir(metadata, state).join(&path).exists() {
            let owned = if let Some(previous) = &previous {
                previous.path == path && prepared.owned_previous
            } else {
                false
            };
            if !owned {
                return Err(crate::ErrorKind::InputError(
                    "An existing pack would be replaced.".to_string(),
                )
                .into());
            }
        }
    }
    let (bytes, sha1) = prepared.contents.take().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Pack contents were not prepared.".to_owned(),
        )
    })?;
    let mut pending = previous.clone().unwrap_or_default();
    pending.pending = true;
    library
        .instances
        .entry(instance_id.clone())
        .or_default()
        .insert(id.to_string(), pending);
    write_library(library, state).await?;
    if prepared.deferred {
        return Ok(());
    }
    for dependency in prepared.dependencies {
        let dependency_path = commands::add_downloaded_project_version(
            instance_id,
            dependency,
            ContentSourceKind::Local,
            state,
        )
        .await?;
        if previous
            .as_ref()
            .is_some_and(|placement| placement.path == dependency_path)
        {
            prepared.owned_previous = false;
        }
    }
    let file_name = if pack.item.enabled {
        file_name.to_string()
    } else {
        format!("{file_name}.disabled")
    };
    let size = bytes.len() as u64;
    let path = commands::add_project_bytes(
        instance_id,
        &file_name,
        bytes,
        Some(&sha1),
        Some(pack.item.project_type),
        ContentSourceKind::Local,
        pack.item
            .project
            .as_ref()
            .map(|project| project.id.as_str()),
        version.as_ref().map(|version| version.id.as_str()),
        state,
    )
    .await?;
    if let Some(previous) = previous
        && previous.path != path
        && prepared.owned_previous
    {
        commands::remove_project(instance_id, &previous.path, state).await?;
        items.retain(|item| item.file_path != previous.path);
    }
    let mut installed = pack.item.clone();
    installed.id = sha1.clone();
    installed.file_path = path.clone();
    installed.file_name = file_name;
    installed.size = size;
    installed.source_kind = Some(ContentSourceKind::Local);
    installed.version = version.as_ref().map(|version| ContentItemVersion {
        id: version.id.clone(),
        version_number: version.version_number.clone(),
        file_name: installed.file_name.clone(),
        date_published: Some(version.date_published.to_rfc3339()),
    });
    items.retain(|item| item.file_path != path);
    items.push(installed);
    library
        .instances
        .entry(instance_id.clone())
        .or_default()
        .insert(
            id.to_string(),
            PackPlacement {
                path,
                sha1,
                enabled: pack.item.enabled,
                content_set_id: metadata.applied_content_set.id.clone(),
                resource_pack_selection_path,
                resource_pack_selection_pending,
                ..Default::default()
            },
        );
    emit_instance(instance_id, InstancePayloadType::Synced).await?;
    Ok(())
}

pub(super) async fn run_queued(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let Some(metadata) =
        crate::state::get_instance(instance_id, &state.pool).await?
    else {
        return Ok(());
    };
    let mut preparation =
        super::worker::Preparation::new(state, &metadata).await;
    let mut library = preparation.library().await?;
    if library.packs.is_empty() && library.instances.is_empty() {
        return Ok(());
    }
    apply_instance_inner(
        &metadata,
        &mut library,
        state,
        None,
        &mut preparation,
    )
    .await?;
    write_library(&library, state).await?;
    emit_instance(instance_id, InstancePayloadType::Synced).await?;
    Ok(())
}

async fn apply_instance_inner(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    state: &State,
    removed_pack_id: Option<&str>,
    preparation: &mut super::worker::Preparation<'_>,
) -> crate::Result<()> {
    if super::super::synced_options::pending::contains(
        &metadata.instance.id,
        SyncedOption::ResourcePacks,
        state,
    )
    .await?
    {
        return Ok(());
    }
    if sync_files_are_protected(metadata)
        || super::super::projects::ensure_metadata_content_unlocked(metadata)
            .is_err()
    {
        return Ok(());
    }
    let running = instance_is_running(metadata, state).await?;
    let global = get_global_options().await?;
    let has_packs = removed_pack_id.is_none()
        && library
            .packs
            .values()
            .any(|pack| participating(metadata, pack, global));
    let (mut items, versions) = if has_packs {
        preparation
            .run(library, async {
                let items =
                    commands::list_pack_content(&metadata.instance.id, state)
                        .await?;
                let ids = items
                    .iter()
                    .filter_map(|item| {
                        item.version.as_ref().map(|version| version.id.as_str())
                    })
                    .collect::<Vec<_>>();
                let versions = CachedEntry::get_version_many(
                    &ids,
                    None,
                    &state.pool,
                    &state.api_semaphore,
                )
                .await?;
                super::selection_compatibility::warm(metadata, &items, state)
                    .await?;
                Ok((items, versions))
            })
            .await?
    } else {
        (Vec::new(), Vec::new())
    };
    if has_packs && !running {
        let placements = library
            .instances
            .get(&metadata.instance.id)
            .cloned()
            .unwrap_or_default();
        if capture_items(metadata, library, state, placements, global, &items)
            .await?
        {
            super::worker::queue_all();
        }
    }
    let previous_placements = library
        .instances
        .get(&metadata.instance.id)
        .cloned()
        .unwrap_or_default();
    let packs = if removed_pack_id.is_none() {
        library.packs.clone()
    } else {
        BTreeMap::default()
    };
    for (id, pack) in &packs {
        if !participating(metadata, pack, global) {
            continue;
        }
        if running
            && (pack.item.project_type != ProjectType::ResourcePack
                || previous_placements.get(id).is_some_and(|placement| {
                    !placement.path.is_empty()
                        && instance_dir(metadata, state)
                            .join(&placement.path)
                            .exists()
                })
                || items.iter().any(|item| {
                    item.project_type == pack.item.project_type
                        && (item.id == pack.sha1
                            || same_path(
                                &item.file_path,
                                &pack_path(pack, &pack.item.file_name),
                            )
                            || pack.item.project.as_ref().is_some_and(
                                |project| {
                                    item.project.as_ref().is_some_and(
                                        |candidate| candidate.id == project.id,
                                    )
                                },
                            ))
                }))
        {
            continue;
        }
        let previous = library
            .instances
            .get(&metadata.instance.id)
            .and_then(|placements| placements.get(id));
        if previous.is_some_and(|placement| placement.excluded) {
            continue;
        }
        let matching = matching_item(&items, pack, previous);
        let compatible = matching.is_some_and(|item| {
            previous.is_some_and(|placement| placement.is_source)
                || if pack.item.project.is_some() {
                    item.version.as_ref().is_some_and(|installed| {
                        versions.iter().any(|version| {
                            version.id == installed.id
                                && version_compatible(pack, version, metadata)
                        })
                    })
                } else {
                    pack.game_versions
                        .contains(&metadata.applied_content_set.game_version)
                }
        });
        let prepared = if local_conflict(
            &items,
            pack,
            previous,
            &pack_path(pack, &pack.item.file_name),
        ) {
            PreparedPack {
                conflict: true,
                ..Default::default()
            }
        } else if compatible
            || matching.is_some_and(|item| item.locked || !local(item))
        {
            PreparedPack {
                compatible,
                ..Default::default()
            }
        } else {
            match preparation
                .run(
                    library,
                    prepare_pack(metadata, id, pack, library, &items, state),
                )
                .await
            {
                Ok(prepared) => prepared,
                Err(error)
                    if matches!(
                        error.raw.as_ref(),
                        crate::ErrorKind::PackSyncChanged
                            | crate::ErrorKind::Ratelimited { .. }
                    ) =>
                {
                    return Err(error);
                }
                Err(error) => {
                    let placement = library
                        .instances
                        .entry(metadata.instance.id.clone())
                        .or_default()
                        .entry(id.clone())
                        .or_default();
                    let message = error.to_string();
                    if placement.error.as_ref() != Some(&message) {
                        tracing::warn!(
                            "Could not prepare synced pack {id}: {error}"
                        );
                        placement.error = Some(message);
                    }
                    continue;
                }
            }
        };
        let mut refresh = !prepared.dependencies.is_empty();
        if let Err(error) =
            apply_pack(metadata, id, pack, library, &mut items, state, prepared)
                .await
        {
            tracing::warn!(
                "Could not sync pack {id} to {}: {error}",
                metadata.instance.id
            );
            let placement = library
                .instances
                .entry(metadata.instance.id.clone())
                .or_default()
                .entry(id.clone())
                .or_default();
            placement.error = Some(error.to_string());
            refresh = true;
        }
        if refresh {
            items = preparation
                .run(
                    library,
                    commands::list_pack_content(&metadata.instance.id, state),
                )
                .await?;
        }
    }
    let placements = library
        .instances
        .get(&metadata.instance.id)
        .cloned()
        .unwrap_or_default();
    for (id, placement) in placements {
        if running
            || library.packs.contains_key(&id)
            || removed_pack_id.is_some_and(|removed| removed != id)
        {
            continue;
        }
        let included = ProjectType::get_from_parent_folder(&placement.path)
            .and_then(|project_type| pack_option(project_type).ok())
            .is_some_and(|option| {
                global.get(option) && instance_option_enabled(metadata, option)
            });
        if included
            && !placement.excluded
            && !placement.suspended
            && preparation
                .run(library, owns_file(metadata, &placement, state))
                .await?
        {
            commands::remove_project(
                &metadata.instance.id,
                &placement.path,
                state,
            )
            .await?;
            emit_instance(&metadata.instance.id, InstancePayloadType::Synced)
                .await?;
        }
        if let Some(placements) =
            library.instances.get_mut(&metadata.instance.id)
        {
            placements.remove(&id);
        }
    }
    if has_packs {
        preparation
            .run(
                library,
                super::selection_compatibility::warm(metadata, &items, state),
            )
            .await?;
    }
    let selection_result = if let Some(pack_id) = removed_pack_id {
        super::selection::apply_removal(
            metadata,
            library,
            previous_placements.get(pack_id),
            state,
        )
        .await
    } else {
        super::selection::apply(metadata, library, &previous_placements, state)
            .await
    };
    if let Err(error) = selection_result {
        tracing::warn!(
            "Could not apply resource-pack selection for {}: {error}",
            metadata.instance.id
        );
    }
    Ok(())
}

pub(super) async fn apply_removal(
    pack_id: &str,
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<()> {
    library.resource_pack_order.retain(|id| id != pack_id);
    apply_all(library, state).await
}

pub(super) async fn apply_all(
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<()> {
    let instances = crate::state::list_instances(&state.pool).await?;
    let ids = instances
        .iter()
        .map(|metadata| &metadata.instance.id)
        .collect::<std::collections::BTreeSet<_>>();
    library.instances.retain(|id, _| ids.contains(id));
    library
        .resource_pack_observations
        .retain(|id, _| ids.contains(id));
    library
        .resource_pack_incompatible_observations
        .retain(|id, _| ids.contains(id));
    library
        .resource_pack_order
        .retain(|id| library.packs.contains_key(id));
    write_library(library, state).await?;
    super::worker::queue_all();
    Ok(())
}

pub(in crate::api::instance) async fn reconcile(
    metadata: &InstanceMetadata,
    _option: SyncedOption,
    _state: &State,
) -> crate::Result<()> {
    super::worker::queue_reconciliation(&metadata.instance.id);
    Ok(())
}

async fn capture_after_change(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_owned())
        })?;
    let mut preparation =
        super::worker::Preparation::new(&state, &metadata).await;
    let mut library = preparation.library().await?;
    let Some(placements) = library.instances.get(instance_id).cloned() else {
        super::worker::queue_reconciliation(instance_id);
        return Ok(());
    };
    if sync_files_are_protected(&metadata)
        || instance_is_running(&metadata, &state).await?
        || super::super::synced_options::pending::contains(
            instance_id,
            SyncedOption::ResourcePacks,
            &state,
        )
        .await?
    {
        super::worker::queue_reconciliation(instance_id);
        return Ok(());
    }
    let items = preparation
        .run(&library, async {
            let items =
                commands::list_pack_content(instance_id, &state).await?;
            super::selection_compatibility::warm(&metadata, &items, &state)
                .await?;
            Ok(items)
        })
        .await?;
    let global = get_global_options().await?;
    if capture_items(
        &metadata,
        &mut library,
        &state,
        placements,
        global,
        &items,
    )
    .await?
    {
        apply_all(&mut library, &state).await?;
    } else {
        write_library(&library, &state).await?;
        super::worker::queue_reconciliation(instance_id);
    }
    Ok(())
}

pub(in crate::api::instance) fn schedule_reconciliation() {
    super::worker::queue_all();
}

pub(in crate::api::instance) async fn reconcile_after_content_change(
    instance_id: &str,
) {
    loop {
        match capture_after_change(instance_id).await {
            Err(error)
                if matches!(
                    error.raw.as_ref(),
                    crate::ErrorKind::PackSyncChanged
                ) =>
            {
                tokio::task::yield_now().await;
            }
            Err(error) => {
                tracing::warn!(
                    "Could not capture pack changes for {instance_id}: {error}"
                );
                super::worker::queue_reconciliation(instance_id);
                return;
            }
            Ok(()) => return,
        }
    }
}

pub(in crate::api::instance) async fn capture_resource_pack_selection_change(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let mut library = read_library(state).await?;
    match super::selection::capture(metadata, &mut library, state).await? {
        Some(true) => apply_all(&mut library, state).await?,
        Some(false) => write_library(&library, state).await?,
        None => {}
    }
    Ok(())
}

pub(in crate::api::instance) async fn prepare_instance_update(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let mut library = read_library(state).await?;
    match super::selection::capture(metadata, &mut library, state).await {
        Ok(Some(true)) => apply_all(&mut library, state).await?,
        Ok(_) => {}
        Err(error) => tracing::warn!(
            "Could not capture resource-pack selection before updating {}: {error}",
            metadata.instance.id
        ),
    }
    let global = get_global_options().await?;
    let pending_ids = library
        .instances
        .get(&metadata.instance.id)
        .into_iter()
        .flat_map(|placements| placements.iter())
        .filter_map(|(id, placement)| {
            let pack = library.packs.get(id)?;
            (pack.item.project_type == ProjectType::ResourcePack
                && pack.selected.is_some()
                && participating(metadata, pack, global)
                && !placement.excluded
                && !placement.suspended)
                .then(|| id.clone())
        })
        .collect::<Vec<_>>();
    let mut changed = library
        .resource_pack_observations
        .remove(&metadata.instance.id)
        .is_some();
    changed |= library
        .resource_pack_incompatible_observations
        .remove(&metadata.instance.id)
        .is_some();
    if let Some(placements) = library.instances.get_mut(&metadata.instance.id) {
        for id in pending_ids {
            if let Some(placement) = placements.get_mut(&id)
                && !placement.resource_pack_selection_pending
            {
                placement.resource_pack_selection_pending = true;
                changed = true;
            }
        }
    }
    if changed {
        write_library(&library, state).await?;
    }
    Ok(())
}

pub(in crate::api::instance) async fn detach(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    let mut library = read_library(state).await?;
    let mut changed = false;
    if let Some(placements) = library.instances.get_mut(&metadata.instance.id) {
        for (id, placement) in placements {
            if library.packs.get(id).is_some_and(|pack| {
                pack_option(pack.item.project_type).ok() == Some(option)
            }) && !placement.suspended
            {
                placement.suspended = true;
                changed = true;
            }
        }
    }
    if option == SyncedOption::ResourcePacks {
        changed |= library
            .resource_pack_observations
            .remove(&metadata.instance.id)
            .is_some();
        changed |= library
            .resource_pack_incompatible_observations
            .remove(&metadata.instance.id)
            .is_some();
    }
    if changed {
        write_library(&library, state).await?;
    }
    Ok(())
}

pub(in crate::api::instance) async fn decorate_content(
    instance_id: &str,
    items: &mut [ContentItem],
    state: &State,
) -> crate::Result<()> {
    let library = read_library(state).await?;
    let Some(placements) = library.instances.get(instance_id) else {
        return Ok(());
    };
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let global = get_global_options().await?;
    let instances = crate::state::list_instances(&state.pool).await?;
    for item in items {
        if !local(item) {
            continue;
        }
        for (id, placement) in placements {
            let Some(pack) = library.packs.get(id) else {
                continue;
            };
            if !placement.excluded
                && !placement.suspended
                && same_path(&placement.path, &item.file_path)
                && item.project_type == pack.item.project_type
                && pack.item.project.as_ref().map_or_else(
                    || item.id == placement.sha1,
                    |project| {
                        item.project
                            .as_ref()
                            .is_some_and(|candidate| candidate.id == project.id)
                    },
                )
                && participating(&metadata, pack, global)
            {
                item.synced_pack = Some(SyncedPackInfo {
                    id: id.clone(),
                    instance_ids: synced_instance_ids(
                        id, &library, &instances, global,
                    ),
                    update_pending: placement.pending
                        || item.enabled != pack.item.enabled
                        || placement.error.is_some(),
                });
                break;
            }
        }
    }
    Ok(())
}

pub(super) fn synced_instance_ids(
    id: &str,
    library: &PackLibrary,
    instances: &[InstanceMetadata],
    global: GlobalSyncedOptions,
) -> Vec<String> {
    let Some(pack) = library.packs.get(id) else {
        return Vec::new();
    };
    instances
        .iter()
        .filter(|metadata| {
            participating(metadata, pack, global)
                && library
                    .instances
                    .get(&metadata.instance.id)
                    .and_then(|items| items.get(id))
                    .is_some_and(|placement| {
                        !placement.excluded
                            && !placement.suspended
                            && !placement.path.is_empty()
                    })
        })
        .map(|metadata| metadata.instance.id.clone())
        .collect()
}
