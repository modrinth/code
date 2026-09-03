use super::super::synced_options::game_options::{
    ResourcePackOptionsUpdate, merge_resource_pack_entries,
    merge_resource_pack_order, read_resource_pack_entries,
};
use super::super::synced_options::{get_global_options, instance_dir};
use super::reconciliation::participating;
use super::selection_compatibility;
use super::{PackLibrary, PackPlacement, SyncedPack};
use crate::state::instances::commands;
use crate::state::{
    ContentSourceKind, InstanceMetadata, ProjectType, State, SyncedOption,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn option_entry(path: &str) -> Option<String> {
    let path = path.trim_end_matches(".disabled");
    let file_name = Path::new(path).file_name()?.to_str()?;
    Some(format!("file/{file_name}"))
}

fn selected_entry(entries: &[String], entry: &str) -> bool {
    let legacy = entry.strip_prefix("file/").unwrap_or(entry);
    entries
        .iter()
        .any(|candidate| candidate == entry || candidate == legacy)
}

fn target_entry(metadata: &InstanceMetadata, entry: String) -> String {
    let legacy = metadata
        .applied_content_set
        .game_version
        .strip_prefix("1.")
        .and_then(|version| version.split('.').next()?.parse::<u32>().ok())
        .is_some_and(|minor| minor < 13);
    if legacy {
        entry.strip_prefix("file/").unwrap_or(&entry).to_string()
    } else {
        entry
    }
}

async fn local_file(
    metadata: &InstanceMetadata,
    path: &str,
    state: &State,
) -> crate::Result<bool> {
    if path.is_empty() || !instance_dir(metadata, state).join(path).exists() {
        return Ok(false);
    }
    let kind = commands::content_source_kind_for_project_path(
        &metadata.instance.id,
        path,
        state,
    )
    .await?;
    Ok(kind.is_none_or(|kind| kind == ContentSourceKind::Local))
}

async fn can_capture(
    metadata: &InstanceMetadata,
    pack: &SyncedPack,
    placement: &PackPlacement,
    state: &State,
) -> crate::Result<bool> {
    if pack.item.project_type != ProjectType::ResourcePack
        || !pack.item.enabled
        || !placement.enabled
        || placement.excluded
        || placement.suspended
        || placement.pending
        || placement.error.is_some()
        || placement.path.ends_with(".disabled")
        || placement.content_set_id != metadata.applied_content_set.id
    {
        return Ok(false);
    }
    local_file(metadata, &placement.path, state).await
}

pub(super) async fn selected_in_instance(
    metadata: &InstanceMetadata,
    project_path: &str,
    state: &State,
) -> crate::Result<Option<bool>> {
    let Some(entry) = option_entry(project_path) else {
        return Ok(None);
    };
    let Some(options) = read_resource_pack_entries(metadata, state).await?
    else {
        return Ok(None);
    };
    Ok(Some(selected_entry(&options.entries, &entry)))
}

pub(super) async fn capture_source_order(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<()> {
    let Some(options) = read_resource_pack_entries(metadata, state).await?
    else {
        return Ok(());
    };
    let mut known_entries = BTreeMap::new();
    for (id, placement) in library
        .instances
        .get(&metadata.instance.id)
        .into_iter()
        .flat_map(|placements| placements.iter())
    {
        let Some(pack) = library.packs.get(id) else {
            continue;
        };
        if pack.selected != Some(true)
            || placement.resource_pack_selection_pending
            || !can_capture(metadata, pack, placement, state).await?
        {
            continue;
        }
        let Some(entry) = option_entry(&placement.path) else {
            continue;
        };
        let legacy = entry.strip_prefix("file/").unwrap_or(&entry).to_string();
        known_entries.insert(entry, id.clone());
        known_entries.insert(legacy, id.clone());
    }
    let mut seen = BTreeSet::new();
    let selected = options
        .entries
        .iter()
        .filter_map(|entry| known_entries.get(entry))
        .filter(|id| seen.insert((*id).clone()))
        .cloned()
        .collect::<Vec<_>>();
    let managed = selected.iter().cloned().collect();
    library.resource_pack_order = merge_resource_pack_order(
        &library.resource_pack_order,
        &managed,
        &selected,
    );
    library
        .resource_pack_observations
        .insert(metadata.instance.id.clone(), options.entries);
    library
        .resource_pack_incompatible_observations
        .insert(metadata.instance.id.clone(), options.incompatible);
    Ok(())
}

pub(super) async fn capture(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    state: &State,
) -> crate::Result<Option<bool>> {
    let Some(placements) =
        library.instances.get(&metadata.instance.id).cloned()
    else {
        return Ok(None);
    };
    let global = get_global_options().await?;
    if !global.get(SyncedOption::ResourcePacks)
        || !metadata.synced_options.resource_packs
    {
        return Ok(None);
    }
    let Some(options) = read_resource_pack_entries(metadata, state).await?
    else {
        return Ok(None);
    };
    let entries = options.entries;
    let mut game_format = None;
    let previous = library
        .resource_pack_observations
        .get(&metadata.instance.id)
        .cloned();
    let previous_incompatible = library
        .resource_pack_incompatible_observations
        .get(&metadata.instance.id)
        .cloned()
        .unwrap_or_default();
    let mut shared_changed = false;
    let mut initialized = false;
    let mut known_entries = BTreeMap::new();
    let mut deselected = BTreeSet::new();
    for (id, placement) in placements {
        let Some(pack) = library.packs.get(&id) else {
            continue;
        };
        if !participating(metadata, pack, global)
            || !can_capture(metadata, pack, &placement, state).await?
        {
            continue;
        }
        let Some(entry) = option_entry(&placement.path) else {
            continue;
        };
        let selected = selected_entry(&entries, &entry);
        let changed_in_instance = previous.as_ref().is_some_and(|previous| {
            selected_entry(previous, &entry) != selected
        });
        let initialize = pack.selected.is_none();
        let path_changed = placement
            .resource_pack_selection_path
            .as_ref()
            .is_some_and(|path| option_entry(path) != Some(entry.clone()));
        if path_changed
            && !selected
            && !changed_in_instance
            && pack.selected.is_some()
        {
            continue;
        }
        if placement.resource_pack_selection_pending && !changed_in_instance {
            continue;
        }
        if !selected && !changed_in_instance && !initialize {
            continue;
        }
        let target_entry = target_entry(metadata, entry.clone());
        if !selected
            && !previous_incompatible.contains(&target_entry)
            && !options.incompatible.contains(&target_entry)
        {
            let format = if let Some(format) = game_format {
                format
            } else {
                let format =
                    selection_compatibility::game_format(metadata, state)
                        .await?;
                game_format = Some(format);
                format
            };
            if !selection_compatibility::compatible(
                &instance_dir(metadata, state).join(&placement.path),
                format,
            )
            .await
            {
                continue;
            }
        }
        let legacy = entry.strip_prefix("file/").unwrap_or(&entry).to_string();
        known_entries.insert(entry, id.clone());
        known_entries.insert(legacy, id.clone());
        initialized |= initialize;
        if (changed_in_instance || initialize)
            && pack.selected != Some(selected)
        {
            if let Some(pack) = library.packs.get_mut(&id) {
                pack.selected = Some(selected);
            }
            shared_changed = true;
        }
        if (changed_in_instance || initialize) && !selected {
            deselected.insert(id.clone());
        }
        if changed_in_instance
            && let Some(placement) = library
                .instances
                .get_mut(&metadata.instance.id)
                .and_then(|placements| placements.get_mut(&id))
        {
            placement.resource_pack_selection_pending = false;
        }
    }
    let ordered_ids = |entries: &[String]| {
        let mut seen = BTreeSet::new();
        entries
            .iter()
            .filter_map(|entry| known_entries.get(entry))
            .filter(|id| seen.insert((*id).clone()))
            .cloned()
            .collect::<Vec<_>>()
    };
    let current_order = ordered_ids(&entries);
    let order_changed = previous
        .as_ref()
        .is_some_and(|previous| ordered_ids(previous) != current_order);
    if initialized || previous.is_none() || order_changed {
        let selected = current_order
            .into_iter()
            .filter(|id| {
                library
                    .packs
                    .get(id)
                    .is_some_and(|pack| pack.selected == Some(true))
            })
            .collect::<Vec<_>>();
        let managed = selected.iter().cloned().chain(deselected).collect();
        let order = merge_resource_pack_order(
            &library.resource_pack_order,
            &managed,
            &selected,
        );
        if order != library.resource_pack_order {
            library.resource_pack_order = order;
            shared_changed = true;
        }
    }
    let observation_changed = previous.as_ref() != Some(&entries)
        || previous_incompatible != options.incompatible;
    if observation_changed {
        library
            .resource_pack_observations
            .insert(metadata.instance.id.clone(), entries);
        library
            .resource_pack_incompatible_observations
            .insert(metadata.instance.id.clone(), options.incompatible);
    }
    Ok((shared_changed || observation_changed).then_some(shared_changed))
}

pub(super) async fn apply(
    metadata: &InstanceMetadata,
    library: &mut PackLibrary,
    previous_placements: &BTreeMap<String, PackPlacement>,
    state: &State,
) -> crate::Result<()> {
    let directory = instance_dir(metadata, state);
    let global = get_global_options().await?;
    if !global.get(SyncedOption::ResourcePacks)
        || !metadata.synced_options.resource_packs
    {
        return Ok(());
    }
    let mut managed = BTreeSet::new();
    for placement in previous_placements.values() {
        if placement.excluded
            || placement.suspended
            || placement.error.is_some()
        {
            continue;
        }
        for path in std::iter::once(&placement.path)
            .chain(placement.resource_pack_selection_path.iter())
        {
            if !path.is_empty()
                && ProjectType::get_from_parent_folder(path)
                    == Some(ProjectType::ResourcePack)
                && !directory.join(path).exists()
                && let Some(entry) = option_entry(path)
            {
                managed.insert(
                    entry.strip_prefix("file/").unwrap_or(&entry).to_string(),
                );
                managed.insert(entry);
            }
        }
    }

    let placements = library
        .instances
        .get(&metadata.instance.id)
        .cloned()
        .unwrap_or_default();
    let mut tracked = Vec::new();
    let mut active_entries = BTreeMap::new();
    let game_format =
        selection_compatibility::game_format(metadata, state).await?;
    let incompatible = read_resource_pack_entries(metadata, state)
        .await?
        .map(|options| options.incompatible)
        .unwrap_or_default();
    for (id, placement) in &placements {
        let Some(pack) = library.packs.get(id) else {
            continue;
        };
        if pack.item.project_type != ProjectType::ResourcePack
            || !participating(metadata, pack, global)
            || placement.excluded
            || placement.suspended
            || placement.pending
            || placement.error.is_some()
            || placement.content_set_id != metadata.applied_content_set.id
            || !local_file(metadata, &placement.path, state).await?
        {
            continue;
        }
        let Some(selected) = pack.selected else {
            continue;
        };
        let Some(entry) = option_entry(&placement.path) else {
            continue;
        };
        let target_entry = target_entry(metadata, entry.clone());
        if selected
            && pack.item.enabled
            && placement.enabled
            && !incompatible.contains(&target_entry)
            && !selection_compatibility::compatible(
                &directory.join(&placement.path),
                game_format,
            )
            .await
        {
            if let Some(placement) = library
                .instances
                .get_mut(&metadata.instance.id)
                .and_then(|placements| placements.get_mut(id))
            {
                placement.resource_pack_selection_pending = true;
            }
            continue;
        }
        managed
            .insert(entry.strip_prefix("file/").unwrap_or(&entry).to_string());
        managed.insert(entry.clone());
        if selected && pack.item.enabled && placement.enabled {
            active_entries.insert(id.clone(), target_entry);
        }
        tracked.push(id.clone());
    }
    if managed.is_empty() {
        return Ok(());
    }
    let mut selected = Vec::new();
    for id in &library.resource_pack_order {
        if let Some(entry) = active_entries.remove(id) {
            selected.push(entry);
        }
    }
    selected.extend(active_entries.into_values());
    library
        .resource_pack_observations
        .remove(&metadata.instance.id);
    library
        .resource_pack_incompatible_observations
        .remove(&metadata.instance.id);
    if let Some(placements) = library.instances.get_mut(&metadata.instance.id) {
        for id in &tracked {
            if let Some(placement) = placements.get_mut(id) {
                placement.resource_pack_selection_pending = true;
            }
        }
    }
    let update =
        merge_resource_pack_entries(metadata, &managed, &selected, state)
            .await?;
    let pending = match update {
        ResourcePackOptionsUpdate::Deferred => true,
        ResourcePackOptionsUpdate::Applied(options) => {
            library
                .resource_pack_observations
                .insert(metadata.instance.id.clone(), options.entries);
            library
                .resource_pack_incompatible_observations
                .insert(metadata.instance.id.clone(), options.incompatible);
            false
        }
    };
    if let Some(placements) = library.instances.get_mut(&metadata.instance.id) {
        for id in tracked {
            if let Some(placement) = placements.get_mut(&id) {
                placement.resource_pack_selection_pending = pending;
                if !pending {
                    placement.resource_pack_selection_path =
                        Some(placement.path.clone());
                }
            }
        }
    }
    Ok(())
}
