use super::storage::{read_bytes, read_library, write_library};
use super::{
    PackLibrary, PackPlacement, SyncedPack, pack_option, pack_path, same_path,
    version_compatible,
};
use crate::event::{InstancePayloadType, emit::emit_instance};
use crate::state::instances::commands;
use crate::state::{
    CacheBehaviour, CachedEntry, ContentItem, ContentSourceKind,
    InstanceMetadata, ProjectType, State, SyncedOption, SyncedPackInfo,
};
use crate::util::fetch;

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

pub(super) async fn capture(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<bool> {
    let Some(placements) =
        library.instances.get(&metadata.instance.id).cloned()
    else {
        return Ok(false);
    };
    if sync_files_are_protected(metadata)
        || instance_is_running(metadata, state).await?
    {
        return Ok(false);
    }
    let global = get_global_options().await?;
    let items =
        commands::list_content(&metadata.instance.id, None, None, state)
            .await?;
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

async fn apply_pack(
    metadata: &InstanceMetadata,
    id: &str,
    pack: &SyncedPack,
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<()> {
    let instance_id = &metadata.instance.id;
    let previous = library
        .instances
        .get(instance_id)
        .and_then(|items| items.get(id))
        .cloned();
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
    let items = commands::list_content(instance_id, None, None, state).await?;
    let matching = previous
        .as_ref()
        .filter(|placement| !placement.path.is_empty())
        .and_then(|placement| current_item(&items, pack, placement))
        .or_else(|| {
            items.iter().find(|item| {
                local(item)
                    && item.id == pack.sha1
                    && item.project_type == pack.item.project_type
            })
        });
    if matching.is_some_and(|item| !local(item)) {
        return Err(crate::ErrorKind::InputError(
            "This pack is managed in this instance.".to_string(),
        )
        .into());
    }
    if let Some(item) = matching {
        let compatible = if pack.item.project.is_some() {
            if let Some(version) = &item.version {
                CachedEntry::get_version(
                    &version.id,
                    None,
                    &state.pool,
                    &state.api_semaphore,
                )
                .await?
                .is_some_and(|version| {
                    version_compatible(pack, &version, metadata)
                })
            } else {
                false
            }
        } else {
            pack.game_versions
                .contains(&metadata.applied_content_set.game_version)
        };
        if compatible || item.locked {
            let changed = item.enabled != pack.item.enabled;
            let joined = previous
                .as_ref()
                .is_none_or(|placement| placement.suspended);
            let path = if changed {
                toggle_pack(metadata, item, pack.item.enabled, state).await?
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
                        path,
                        sha1: item.id.clone(),
                        enabled: pack.item.enabled,
                        pending: !compatible,
                        content_set_id: metadata.applied_content_set.id.clone(),
                        resource_pack_selection_path,
                        resource_pack_selection_pending,
                        ..Default::default()
                    },
                );
            if changed || joined {
                emit_instance(instance_id, InstancePayloadType::Synced).await?;
            }
            return Ok(());
        }
    }
    let version = if let Some(project) = &pack.item.project {
        CachedEntry::get_project_versions(
            &project.id,
            Some(CacheBehaviour::MustRevalidate),
            &state.pool,
            &state.api_semaphore,
        )
        .await?
        .unwrap_or_default()
        .into_iter()
        .filter(|version| {
            version_compatible(pack, version, metadata)
                && !version.files.is_empty()
        })
        .max_by_key(|version| version.date_published)
    } else {
        None
    };
    let compatible = if pack.item.project.is_some() {
        version.is_some()
    } else {
        pack.game_versions
            .contains(&metadata.applied_content_set.game_version)
    };
    if !compatible {
        if let Some(previous) = &previous
            && owns_file(metadata, previous, state).await?
        {
            commands::remove_project(instance_id, &previous.path, state)
                .await?;
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
                previous.path == path
                    && owns_file(metadata, previous, state).await?
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
    let bytes = if let Some(file) = file {
        fetch::fetch(
            &file.url,
            file.hashes.get("sha1").map(String::as_str),
            None,
            None,
            &state.fetch_semaphore,
            &state.pool,
        )
        .await?
    } else {
        read_bytes(pack, state).await?
    };
    super::operations::validate_pack(&bytes, pack.item.project_type)?;
    let sha1 = fetch::sha1_async(bytes.clone()).await?;
    let mut pending = previous.clone().unwrap_or_default();
    pending.pending = true;
    library
        .instances
        .entry(instance_id.clone())
        .or_default()
        .insert(id.to_string(), pending);
    write_library(library, state).await?;
    if let (Some(project), Some(version)) = (&pack.item.project, &version) {
        let plan = commands::resolve_install_plan(
            instance_id,
            commands::InstanceInstallProjectRequest {
                project_id: project.id.clone(),
                version_id: Some(version.id.clone()),
                content_type: pack.item.project_type.into(),
                selected: Default::default(),
            },
            state,
        )
        .await?;
        for dependency in &plan.dependencies {
            commands::add_project_from_version(
                instance_id,
                &dependency.version_id,
                fetch::DownloadReason::Dependency,
                dependency.dependent_on_version_id.clone(),
                ContentSourceKind::Local,
                state,
            )
            .await?;
        }
    }
    let file_name = if pack.item.enabled {
        file_name.to_string()
    } else {
        format!("{file_name}.disabled")
    };
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
        && owns_file(metadata, &previous, state).await?
    {
        commands::remove_project(instance_id, &previous.path, state).await?;
    }
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

pub(super) async fn apply_instance(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<()> {
    if sync_files_are_protected(metadata)
        || instance_is_running(metadata, state).await?
        || super::super::projects::ensure_metadata_content_unlocked(metadata)
            .is_err()
    {
        return Ok(());
    }
    let global = get_global_options().await?;
    let previous_placements = library
        .instances
        .get(&metadata.instance.id)
        .cloned()
        .unwrap_or_default();
    let packs = library.packs.clone();
    for (id, pack) in &packs {
        if !participating(metadata, pack, global) {
            continue;
        }
        if let Err(error) = apply_pack(metadata, id, pack, library, state).await
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
        }
    }
    let placements = library
        .instances
        .get(&metadata.instance.id)
        .cloned()
        .unwrap_or_default();
    for (id, placement) in placements {
        if packs.contains_key(&id) {
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
            && owns_file(metadata, &placement, state).await?
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
    if let Err(error) =
        super::selection::apply(metadata, library, &previous_placements, state)
            .await
    {
        tracing::warn!(
            "Could not apply resource-pack selection for {}: {error}",
            metadata.instance.id
        );
    }
    Ok(())
}

pub(super) async fn apply_all(
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<()> {
    let instances = crate::state::list_instances(&state.pool).await?;
    library.instances.retain(|id, _| {
        instances.iter().any(|metadata| &metadata.instance.id == id)
    });
    library.resource_pack_observations.retain(|id, _| {
        instances.iter().any(|metadata| &metadata.instance.id == id)
    });
    library
        .resource_pack_incompatible_observations
        .retain(|id, _| {
            instances.iter().any(|metadata| &metadata.instance.id == id)
        });
    library
        .resource_pack_order
        .retain(|id| library.packs.contains_key(id));
    for metadata in &instances {
        if sync_files_are_protected(metadata) {
            continue;
        }
        if let Err(error) =
            super::selection::capture(metadata, library, state).await
        {
            tracing::warn!(
                "Could not capture resource-pack selection for {} before syncing: {error}",
                metadata.instance.id
            );
        }
    }
    write_library(library, state).await?;
    for metadata in instances {
        if let Err(error) = apply_instance(&metadata, library, state).await {
            tracing::warn!(
                "Could not reconcile synced packs for {}: {error}",
                metadata.instance.id
            );
        }
    }
    write_library(library, state).await
}

pub(in crate::api::instance) async fn reconcile(
    metadata: &InstanceMetadata,
    _option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    let mut library = read_library(state).await?;
    if library.packs.is_empty() && library.instances.is_empty() {
        return Ok(());
    }
    let metadata =
        crate::state::get_instance(&metadata.instance.id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    if capture(&metadata, &mut library, state).await? {
        apply_all(&mut library, state).await
    } else {
        apply_instance(&metadata, &mut library, state).await?;
        write_library(&library, state).await
    }
}

pub(crate) async fn reconcile_after_change(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    reconcile(&metadata, SyncedOption::ResourcePacks, &state).await
}

pub(in crate::api::instance) async fn reconcile_after_content_change(
    instance_id: &str,
) {
    if let Err(error) = reconcile_after_change(instance_id).await {
        tracing::warn!(
            "Content changed in {instance_id}, but synced packs could not be reconciled: {error}"
        );
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
