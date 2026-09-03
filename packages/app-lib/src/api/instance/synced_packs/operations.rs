use super::super::DesyncServerMode;
use super::super::synced_options::{
    get_global_options, instance_dir, instance_is_running,
    instance_option_enabled, sync_files_are_protected,
};
use super::reconciliation::{apply_all, participating, synced_instance_ids};
use super::storage::{cache_bytes, read_library, write_library};
use super::{
    PackLibrary, PackPlacement, PackSyncPreview, PackSyncTarget, SyncedPack,
    pack_option, version_compatible,
};
use crate::event::{InstancePayloadType, emit::emit_instance};
use crate::state::instances::commands;
use crate::state::{
    CacheBehaviour, CachedEntry, ContentItem, ContentSourceKind,
    InstanceMetadata, ProjectType, State, SyncedOption, SyncedPackInfo,
};
use crate::util::io;
use bytes::Bytes;
use std::io::{Cursor, Read};
use std::path::Path;
use uuid::Uuid;

async fn source(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<(InstanceMetadata, ContentItem)> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    super::super::projects::ensure_metadata_content_unlocked(&metadata)?;
    if sync_files_are_protected(&metadata)
        || instance_is_running(&metadata, state).await?
    {
        return Err(crate::ErrorKind::InputError(
            "Close the instance before syncing its packs.".to_string(),
        )
        .into());
    }
    let item = commands::list_content(instance_id, None, None, state)
        .await?
        .into_iter()
        .find(|item| item.file_path == project_path)
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "This pack could not be found in the instance.".to_string(),
            )
        })?;
    pack_option(item.project_type)?;
    if item
        .source_kind
        .is_some_and(|kind| kind != ContentSourceKind::Local)
    {
        return Err(crate::ErrorKind::InputError(
            "Managed content cannot be added to pack syncing.".to_string(),
        )
        .into());
    }
    Ok((metadata, item))
}

pub(super) async fn pack_from_item(
    mut item: ContentItem,
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<SyncedPack> {
    pack_option(item.project_type)?;
    if !path_util::is_safe_file_name(&item.file_name)
        || item.file_path
            != format!("{}/{}", item.project_type.get_folder(), item.file_name)
    {
        return Err(crate::ErrorKind::InputError(
            "Invalid pack file path.".to_string(),
        )
        .into());
    }
    let bytes = Bytes::from(
        io::read(instance_dir(metadata, state).join(&item.file_path)).await?,
    );
    validate_pack(&bytes, item.project_type)?;
    let game_versions = if let Some(version) = &item.version {
        CachedEntry::get_version(
            &version.id,
            None,
            &state.pool,
            &state.api_semaphore,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "The pack's compatibility could not be verified.".to_string(),
            )
        })?
        .game_versions
    } else {
        vec![metadata.applied_content_set.game_version.clone()]
    };
    let sha1 = cache_bytes(bytes, state).await?;
    item.file_name = item.file_name.trim_end_matches(".disabled").to_string();
    item.file_path.clear();
    item.synced_pack = None;
    item.locked = false;
    item.has_update = false;
    item.update_version_id = None;
    Ok(SyncedPack {
        item,
        sha1,
        game_versions,
    })
}

pub(super) fn validate_pack(
    bytes: &[u8],
    project_type: ProjectType,
) -> crate::Result<()> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| {
            crate::ErrorKind::InputError(
                "Choose a valid pack ZIP file.".to_string(),
            )
        })?;
    let mut metadata = String::new();
    {
        let file = archive.by_name("pack.mcmeta").map_err(|_| {
            crate::ErrorKind::InputError(
                "Choose a pack ZIP containing pack.mcmeta.".to_string(),
            )
        })?;
        if file.size() > 1024 * 1024 {
            return Err(crate::ErrorKind::InputError(
                "The pack metadata is too large.".to_string(),
            )
            .into());
        }
        file.take(1024 * 1024 + 1).read_to_string(&mut metadata)?;
    }
    let value: serde_json::Value = serde_json::from_str(&metadata)?;
    let (prefix, other_prefix) = match project_type {
        ProjectType::ResourcePack => ("assets/", "data/"),
        ProjectType::DataPack => ("data/", "assets/"),
        _ => {
            return Err(crate::ErrorKind::InputError(
                "Unsupported synced content type.".to_string(),
            )
            .into());
        }
    };
    if !value.get("pack").is_some_and(serde_json::Value::is_object)
        || (!archive.file_names().any(|name| name.starts_with(prefix))
            && archive
                .file_names()
                .any(|name| name.starts_with(other_prefix)))
    {
        return Err(crate::ErrorKind::InputError(
            "This file is not a pack of the selected type.".to_string(),
        )
        .into());
    }
    Ok(())
}

fn existing_pack<'a>(
    library: &'a PackLibrary,
    candidate: &SyncedPack,
) -> Option<(&'a String, &'a SyncedPack)> {
    library.packs.iter().find(|(_, pack)| {
        pack.item.project_type == candidate.item.project_type
            && (pack.sha1 == candidate.sha1
                || candidate.item.project.as_ref().is_some_and(|project| {
                    pack.item
                        .project
                        .as_ref()
                        .is_some_and(|existing| existing.id == project.id)
                }))
    })
}

pub async fn get_pack_sync_preview(
    instance_id: &str,
    project_path: &str,
) -> crate::Result<PackSyncPreview> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let (metadata, item) = source(instance_id, project_path, &state).await?;
    let candidate = pack_from_item(item, &metadata, &state).await?;
    let library = read_library(&state).await?;
    let existing = existing_pack(&library, &candidate);
    let pack = existing.map_or(&candidate, |(_, pack)| pack);
    let versions = if let Some(project) = &pack.item.project {
        Some(
            CachedEntry::get_project_versions(
                &project.id,
                Some(CacheBehaviour::MustRevalidate),
                &state.pool,
                &state.api_semaphore,
            )
            .await?
            .unwrap_or_default(),
        )
    } else {
        None
    };
    let global = get_global_options().await?;
    let instances = crate::state::list_instances(&state.pool)
        .await?
        .iter()
        .map(|instance| {
            let excluded = existing.is_some_and(|(id, _)| {
                library
                    .instances
                    .get(&instance.instance.id)
                    .and_then(|items| items.get(id))
                    .is_some_and(|placement| placement.excluded)
            });
            PackSyncTarget {
                instance_id: instance.instance.id.clone(),
                name: instance.instance.name.clone(),
                game_version: instance.applied_content_set.game_version.clone(),
                compatible: versions.as_ref().map_or_else(
                    || {
                        pack.game_versions.contains(
                            &instance.applied_content_set.game_version,
                        )
                    },
                    |versions| {
                        versions.iter().any(|version| {
                            version_compatible(pack, version, instance)
                        })
                    },
                ),
                participating: participating(instance, pack, global)
                    && (instance.instance.id == instance_id || !excluded),
            }
        })
        .collect();
    Ok(PackSyncPreview {
        pack: pack.item.clone(),
        instances,
    })
}

pub async fn sync_pack(
    instance_id: &str,
    project_path: &str,
) -> crate::Result<()> {
    sync_pack_inner(instance_id, project_path, false).await
}

pub(in crate::api::instance) async fn seed_from_instance(
    metadata: &InstanceMetadata,
    option: SyncedOption,
    state: &State,
) -> crate::Result<()> {
    super::super::projects::ensure_metadata_content_unlocked(metadata)?;
    let instance_id = &metadata.instance.id;
    let items = commands::list_content(instance_id, None, None, state).await?;
    let mut candidates = Vec::new();
    for item in items {
        if pack_option(item.project_type).ok() != Some(option)
            || item
                .source_kind
                .is_some_and(|kind| kind != ContentSourceKind::Local)
        {
            continue;
        }
        let candidate = pack_from_item(item.clone(), metadata, state).await?;
        if !candidate
            .game_versions
            .contains(&metadata.applied_content_set.game_version)
        {
            return Err(crate::ErrorKind::InputError(
				"A pack in the selected sync source is incompatible with its Minecraft version.".to_string(),
			).into());
        }
        candidates.push((item, candidate));
    }

    let mut library = read_library(state).await?;
    for (item, mut pack) in candidates {
        let id = existing_pack(&library, &pack)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        pack.item.id = id.clone();
        library.packs.insert(id.clone(), pack);
        library
            .instances
            .entry(instance_id.clone())
            .or_default()
            .insert(
                id,
                PackPlacement {
                    path: item.file_path,
                    sha1: item.id,
                    enabled: item.enabled,
                    content_set_id: metadata.applied_content_set.id.clone(),
                    ..Default::default()
                },
            );
    }
    write_library(&library, state).await
}

pub(in crate::api::instance) async fn sync_new_pack(
    instance_id: &str,
    project_path: &str,
) {
    let Some(_) = ProjectType::get_from_parent_folder(project_path)
        .and_then(|project_type| pack_option(project_type).ok())
    else {
        return;
    };
    if let Err(error) = sync_pack_inner(instance_id, project_path, true).await {
        tracing::warn!(
            "Installed {project_path} in {instance_id}, but could not sync it: {error}"
        );
    }
}

async fn sync_pack_inner(
    instance_id: &str,
    project_path: &str,
    automatic: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let global = get_global_options().await?;
    if automatic {
        let metadata = crate::state::get_instance(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
        let option = ProjectType::get_from_parent_folder(project_path)
            .and_then(|project_type| pack_option(project_type).ok());
        if !option.is_some_and(|option| {
            global.get(option) && instance_option_enabled(&metadata, option)
        }) {
            return Ok(());
        }
    }
    let (metadata, item) = source(instance_id, project_path, &state).await?;
    let candidate = pack_from_item(item.clone(), &metadata, &state).await?;
    if !candidate
        .game_versions
        .contains(&metadata.applied_content_set.game_version)
    {
        return Err(crate::ErrorKind::InputError(
            "This pack version is incompatible with this instance.".to_string(),
        )
        .into());
    }
    let mut library = read_library(&state).await?;
    let (id, mut pack) = existing_pack(&library, &candidate)
        .map(|(id, pack)| (id.clone(), pack.clone()))
        .unwrap_or_else(|| (Uuid::new_v4().to_string(), candidate));
    if automatic
        && library
            .instances
            .get(instance_id)
            .and_then(|placements| placements.get(&id))
            .is_some_and(|placement| placement.excluded)
    {
        return Ok(());
    }
    if !participating(&metadata, &pack, global) {
        return Err(crate::ErrorKind::InputError("Enable pack syncing in app settings and turn off this instance's override first.".to_string()).into());
    }
    pack.item.id = id.clone();
    library.packs.insert(id.clone(), pack);
    library
        .instances
        .entry(instance_id.to_string())
        .or_default()
        .insert(
            id,
            PackPlacement {
                path: item.file_path,
                sha1: item.id,
                enabled: item.enabled,
                content_set_id: metadata.applied_content_set.id,
                ..Default::default()
            },
        );
    apply_all(&mut library, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Synced).await?;
    Ok(())
}

pub async fn list_synced_packs(
    project_type: ProjectType,
) -> crate::Result<Vec<ContentItem>> {
    pack_option(project_type)?;
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let library = read_library(&state).await?;
    let global = get_global_options().await?;
    let instances = crate::state::list_instances(&state.pool).await?;
    Ok(library
        .packs
        .iter()
        .filter(|(_, pack)| pack.item.project_type == project_type)
        .map(|(id, pack)| {
            let mut item = pack.item.clone();
            item.synced_pack = Some(SyncedPackInfo {
                id: id.clone(),
                instance_ids: synced_instance_ids(
                    id, &library, &instances, global,
                ),
                update_pending: library
                    .instances
                    .values()
                    .filter_map(|placements| placements.get(id))
                    .any(|placement| {
                        !placement.excluded
                            && !placement.suspended
                            && (placement.pending
                                || placement.error.is_some()
                                || placement.enabled != pack.item.enabled)
                    }),
            });
            item
        })
        .collect())
}

pub async fn upload_synced_pack(
    path: &Path,
    project_type: ProjectType,
    game_versions: Vec<String>,
) -> crate::Result<()> {
    let option = pack_option(project_type)?;
    if game_versions.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "Select the Minecraft versions this pack supports.".to_string(),
        )
        .into());
    }
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    if !get_global_options().await?.get(option) {
        return Err(crate::ErrorKind::InputError(
            "Enable pack syncing before adding packs.".to_string(),
        )
        .into());
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| path_util::is_safe_file_name(name))
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Invalid pack filename.".to_string())
        })?
        .to_string();
    let bytes = Bytes::from(io::read(path).await?);
    validate_pack(&bytes, project_type)?;
    let size = bytes.len() as u64;
    let sha1 = cache_bytes(bytes, &state).await?;
    let mut library = read_library(&state).await?;
    if library
        .packs
        .values()
        .any(|pack| pack.sha1 == sha1 && pack.item.project_type == project_type)
    {
        return Ok(());
    }
    let id = Uuid::new_v4().to_string();
    library.packs.insert(
        id.clone(),
        SyncedPack {
            sha1,
            game_versions,
            item: ContentItem {
                id,
                file_name,
                file_path: String::new(),
                size,
                enabled: true,
                locked: false,
                project_type,
                project: None,
                version: None,
                environment: None,
                owner: None,
                has_update: false,
                update_version_id: None,
                date_added: Some(chrono::Utc::now().to_rfc3339()),
                source_kind: Some(ContentSourceKind::Local),
                embedded_metadata: None,
                synced_pack: None,
            },
        },
    );
    apply_all(&mut library, &state).await
}

pub async fn set_synced_pack_enabled(
    pack_id: &str,
    enabled: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let mut library = read_library(&state).await?;
    let pack = library.packs.get_mut(pack_id).ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown synced pack.".to_string())
    })?;
	pack_option(pack.item.project_type)?;
    pack.item.enabled = enabled;
    apply_all(&mut library, &state).await
}

pub async fn remove_synced_pack(pack_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let mut library = read_library(&state).await?;
	let pack = library.packs.get(pack_id).ok_or_else(|| {
		crate::ErrorKind::InputError("Unknown synced pack.".to_string())
	})?;
	pack_option(pack.item.project_type)?;
	library.packs.remove(pack_id);
    apply_all(&mut library, &state).await
}

pub async fn desync_pack(
    instance_id: &str,
    pack_id: &str,
    mode: DesyncServerMode,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    super::super::projects::ensure_metadata_content_unlocked(&metadata)?;
    if sync_files_are_protected(&metadata) {
        return Err(crate::ErrorKind::InputError(
			"Wait for the instance to finish installing before changing pack syncing.".to_string(),
		).into());
    }
    let mut library = read_library(&state).await?;
	let pack = library.packs.get(pack_id).ok_or_else(|| {
		crate::ErrorKind::InputError("Unknown synced pack.".to_string())
	})?;
	pack_option(pack.item.project_type)?;
    let placement = library
        .instances
        .get_mut(instance_id)
        .and_then(|items| items.get_mut(pack_id))
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "This pack is not synced in this instance.".to_string(),
            )
        })?;
    placement.excluded = true;
    if mode == DesyncServerMode::RemoveFromOtherInstances {
        library.packs.remove(pack_id);
        apply_all(&mut library, &state).await?;
    } else {
        write_library(&library, &state).await?;
    }
    emit_instance(instance_id, InstancePayloadType::Synced).await?;
    Ok(())
}
