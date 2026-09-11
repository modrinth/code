use crate::State;
use crate::state::content_store::{
    ContentProjectionStatus, FileContent, content_file_path,
    materialized_content_path,
};
use crate::state::instances::adapters::{filesystem, sqlite};
use crate::state::instances::{Instance, InstanceFile};
use crate::state::{
    CachedEntry, InstanceInstallStage, ProjectType, file_hash_cache_key,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::content_mutation::{
    ContentMutation, ContentMutationExecutor, ContentMutationResult,
};

pub(crate) async fn sync_content_files(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<InstanceFile>> {
    let instance =
        sqlite::instance_rows::get_instance_by_id(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;

    sync_instance_content_files(&instance, state).await
}

pub(crate) async fn sync_instance_content_files(
    instance: &Instance,
    state: &State,
) -> crate::Result<Vec<InstanceFile>> {
    let (snapshot, known_files, known_bindings) = {
        let executor =
            ContentMutationExecutor::lock(&instance.id, state).await?;
        let instance = executor.instance();
        let snapshot = filesystem::scan_content_files(
            &state.directories.instances_dir(),
            &instance.path,
        )?
        .into_iter()
        .map(|file| (file.relative_path.clone(), file))
        .collect::<HashMap<_, _>>();
        let known_files =
            sqlite::content_rows::get_instance_files(&instance.id, &state.pool)
                .await?;
        let known_bindings = crate::state::content_store::catalog::bindings(
            &state.pool,
            &instance.id,
        )
        .await?
        .into_iter()
        .map(|binding| binding.file_id)
        .collect::<HashSet<_>>();
        (snapshot, known_files, known_bindings)
    };
    let managed_paths = known_files
        .iter()
        .filter(|file| known_bindings.contains(&file.id))
        .map(|file| file.relative_path.trim_end_matches(".disabled"))
        .collect::<HashSet<_>>();
    let cache_keys = snapshot
        .values()
        .filter(|file| {
            !managed_paths
                .contains(file.relative_path.trim_end_matches(".disabled"))
        })
        .map(|file| file.hash_cache_key.as_str())
        .collect::<Vec<_>>();
    let hashes = CachedEntry::get_file_hash_many(
        &cache_keys,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let hashes_by_key = hashes
        .into_iter()
        .map(|hash| {
            (
                file_hash_cache_key(
                    hash.size,
                    hash.modified_at_ns,
                    hash.path.trim_end_matches(".disabled"),
                ),
                hash,
            )
        })
        .collect::<HashMap<_, _>>();

    let executor = ContentMutationExecutor::lock(&instance.id, state).await?;
    let instance = executor.instance();
    let mut existing =
        sqlite::content_rows::get_instance_files(&instance.id, &state.pool)
            .await?;
    let bindings = crate::state::content_store::catalog::bindings(
        &state.pool,
        &instance.id,
    )
    .await?
    .into_iter()
    .map(|binding| (binding.file_id.clone(), binding))
    .collect::<HashMap<_, _>>();
    let scanned = filesystem::scan_content_files(
        &state.directories.instances_dir(),
        &instance.path,
    )?;
    let normalized = normalize_legacy_content_files(
        instance, &existing, &bindings, &scanned, state,
    )
    .await?;
    if normalized {
        existing =
            sqlite::content_rows::get_instance_files(&instance.id, &state.pool)
                .await?;
    }
    let existing_by_path = existing
        .iter()
        .map(|file| (file.relative_path.as_str(), file))
        .collect::<HashMap<_, _>>();
    let managed_by_path = existing
        .iter()
        .filter(|file| bindings.contains_key(&file.id))
        .map(|file| (file.relative_path.trim_end_matches(".disabled"), file))
        .collect::<HashMap<_, _>>();
    let mut canonical_paths = HashSet::new();
    let mut duplicate_paths = HashSet::new();
    for file in &scanned {
        let canonical = file.relative_path.trim_end_matches(".disabled");
        if !canonical_paths.insert(canonical) {
            duplicate_paths.insert(canonical);
        }
    }
    let running =
        crate::state::instance_has_running_process(&instance.id, state).await?;
    let mut files = Vec::new();
    let mut stored_by_executor = HashSet::new();
    for previous in &existing {
        let Some(binding) = bindings.get(&previous.id) else {
            continue;
        };
        let mut file = previous.clone();
        let content = state.content_store.file_content(previous).await?;
        let stored = matches!(content, FileContent::Stored(_));
        let projection = state
            .content_store
            .inspect_projection(instance, previous, binding)
            .await?;
        file.missing =
            !stored || projection != ContentProjectionStatus::Healthy;
        if !file.enabled
            && file.missing
            && stored
            && projection == ContentProjectionStatus::Missing
            && !running
        {
            file = match executor
                .execute(ContentMutation::Toggle {
                    project_path: &file.relative_path,
                    desired_enabled: Some(false),
                })
                .await?
            {
                ContentMutationResult::File(file) => file,
                _ => unreachable!("repair mutations return a content file"),
            };
            stored_by_executor.insert(file.id.clone());
        }
        if file.missing != previous.missing {
            file.modified_at = Utc::now();
        }
        files.push(file);
    }
    for scanned in &scanned {
        let canonical_path =
            scanned.relative_path.trim_end_matches(".disabled");
        let previous = existing_by_path
            .get(scanned.relative_path.as_str())
            .or_else(|| {
                (!duplicate_paths.contains(canonical_path))
                    .then(|| existing_by_path.get(canonical_path))
                    .flatten()
            })
            .copied();
        if let Some(file) = managed_by_path.get(canonical_path) {
            if content_file_path(file) != scanned.relative_path {
                tracing::warn!(
                    instance_id = %instance.id,
                    path = %scanned.relative_path,
                    "Ignoring content at the inactive form of a managed path"
                );
            }
            continue;
        }
        let Some(hash) = hashes_by_key
            .get(scanned.hash_cache_key.trim_end_matches(".disabled"))
            .filter(|_| {
                snapshot
                    .get(&scanned.relative_path)
                    .is_some_and(|previous| {
                        previous.hash_cache_key == scanned.hash_cache_key
                            && previous.is_symlink == scanned.is_symlink
                            && previous.has_linked_parent
                                == scanned.has_linked_parent
                    })
            })
        else {
            if let Some(previous) = previous {
                let mut file = previous.clone();
                file.missing = false;
                files.push(file);
            }
            continue;
        };
        let mut file = InstanceFile {
            id: previous
                .map(|file| file.id.clone())
                .unwrap_or_else(instance_file_id),
            instance_id: instance.id.clone(),
            relative_path: scanned.relative_path.clone(),
            file_name: scanned.file_name.clone(),
            enabled: scanned.enabled,
            sha1: hash.hash.clone(),
            size: scanned.size,
            missing: false,
            added_at: previous
                .map(|file| file.added_at)
                .unwrap_or_else(Utc::now),
            modified_at: Utc::now(),
        };
        if let Some(previous) = previous {
            file.relative_path.clone_from(&previous.relative_path);
            file.file_name.clone_from(&previous.file_name);
        } else if !duplicate_paths.contains(canonical_path) {
            file.relative_path = canonical_path.to_string();
            file.file_name =
                scanned.file_name.trim_end_matches(".disabled").to_string();
        }
        files.push(file);
    }
    let present_ids = files
        .iter()
        .map(|file| file.id.as_str())
        .collect::<HashSet<_>>();
    let missing = existing
        .iter()
        .filter(|file| !present_ids.contains(file.id.as_str()) && !file.missing)
        .collect::<Vec<_>>();
    let changed = normalized
        || !missing.is_empty()
        || files.iter().any(|file| {
            existing
                .iter()
                .find(|previous| previous.id == file.id)
                .is_none_or(|previous| {
                    previous.missing != file.missing
                        || previous.enabled != file.enabled
                        || previous.sha1 != file.sha1
                        || previous.relative_path != file.relative_path
                })
        });
    let mut tx = state.pool.begin().await?;
    for file in missing {
        sqlite::content_rows::set_instance_file_missing(
            &file.id, true, &mut tx,
        )
        .await?;
    }
    let mut stored = Vec::new();
    for file in files {
        if stored_by_executor.contains(&file.id) {
            stored.push(file);
        } else {
            stored.push(
                sqlite::content_rows::upsert_instance_file(&file, &mut tx)
                    .await?,
            );
        }
    }
    tx.commit().await?;
    if changed {
        super::mark_shared_instance_stale(&instance.id, &state.pool).await?;
        crate::api::instance::queue_game_locale_index();
    }
    Ok(stored)
}

/// Background migration releases its priority permit and content locks between files.
pub(crate) async fn migrate_legacy_content(
    instance_id: &str,
    state: &State,
    background: bool,
) -> crate::Result<()> {
    let (scanned, mut changed) = {
        let _turn = if background {
            Some(state.content_store.legacy_migration_priority.read().await)
        } else {
            None
        };
        let executor =
            ContentMutationExecutor::lock(instance_id, state).await?;
        let instance = executor.instance();
        if instance.install_stage != InstanceInstallStage::Installed
            || crate::state::instance_has_running_process(instance_id, state)
                .await?
            || sqlite::instance_rows::is_instance_quarantined(
                instance_id,
                &state.pool,
            )
            .await?
        {
            return Ok(());
        }
        let existing =
            sqlite::content_rows::get_instance_files(instance_id, &state.pool)
                .await?;
        let bindings = crate::state::content_store::catalog::bindings(
            &state.pool,
            instance_id,
        )
        .await?
        .into_iter()
        .map(|binding| (binding.file_id.clone(), binding))
        .collect::<HashMap<_, _>>();
        let mut scanned = filesystem::scan_content_files(
            &state.directories.instances_dir(),
            &instance.path,
        )?;
        let normalized = normalize_legacy_content_files(
            instance, &existing, &bindings, &scanned, state,
        )
        .await?;
        let managed_paths = existing
            .iter()
            .filter(|file| bindings.contains_key(&file.id))
            .map(|file| file.relative_path.trim_end_matches(".disabled"))
            .collect::<HashSet<_>>();
        let mut canonical_paths = HashSet::new();
        let mut duplicates = HashSet::new();
        for file in &scanned {
            let canonical = file.relative_path.trim_end_matches(".disabled");
            if !canonical_paths.insert(canonical.to_string()) {
                duplicates.insert(canonical.to_string());
            }
        }
        scanned.retain(|file| {
            let canonical = file.relative_path.trim_end_matches(".disabled");
            !file.is_symlink
                && !file.has_linked_parent
                && crate::state::content_store::eligible(&file.relative_path)
                && !duplicates.contains(canonical)
                && !managed_paths.contains(canonical)
        });
        (scanned, normalized)
    };
    for scanned in scanned {
        let _turn = if background {
            Some(state.content_store.legacy_migration_priority.read().await)
        } else {
            None
        };
        let executor =
            ContentMutationExecutor::lock(instance_id, state).await?;
        let instance = executor.instance();
        if instance.install_stage != InstanceInstallStage::Installed
            || crate::state::instance_has_running_process(instance_id, state)
                .await?
            || sqlite::instance_rows::is_instance_quarantined(
                instance_id,
                &state.pool,
            )
            .await?
        {
            break;
        }
        let canonical = scanned.relative_path.trim_end_matches(".disabled");
        let previous =
            sqlite::content_rows::get_instance_file_by_relative_path(
                instance_id,
                canonical,
                &state.pool,
            )
            .await?;
        if let Some(previous) = &previous
            && crate::state::content_store::catalog::binding(
                &state.pool,
                &previous.id,
            )
            .await?
            .is_some()
        {
            continue;
        }
        let inactive = materialized_content_path(canonical, !scanned.enabled);
        let inactive = state
            .content_store
            .instance_path(&instance.path, &inactive)
            .await?;
        match tokio::fs::symlink_metadata(inactive).await {
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let source = state
            .content_store
            .instance_path(&instance.path, &scanned.relative_path)
            .await?;
        let metadata = match tokio::fs::symlink_metadata(source).await {
            Ok(metadata)
                if metadata.is_file() && !metadata.file_type().is_symlink() =>
            {
                metadata
            }
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        let file = InstanceFile {
            id: previous
                .as_ref()
                .map(|file| file.id.clone())
                .unwrap_or_else(instance_file_id),
            instance_id: instance_id.to_string(),
            relative_path: scanned.relative_path,
            file_name: scanned.file_name,
            enabled: scanned.enabled,
            sha1: String::new(),
            size: metadata.len(),
            missing: false,
            added_at: previous
                .as_ref()
                .map(|file| file.added_at)
                .unwrap_or_else(Utc::now),
            modified_at: Utc::now(),
        };
        match executor
            .execute(ContentMutation::Adopt { file: &file })
            .await?
        {
            ContentMutationResult::File(_) => changed = true,
            ContentMutationResult::Deferred { reason } => tracing::warn!(
                instance_id, path = %file.relative_path, reason,
                "Legacy file adoption deferred",
            ),
            ContentMutationResult::Removed => {
                unreachable!("adoption cannot remove content")
            }
        }
    }
    if changed {
        super::mark_shared_instance_stale(instance_id, &state.pool).await?;
        crate::api::instance::queue_game_locale_index();
        crate::event::emit::emit_instance(
            instance_id,
            crate::event::InstancePayloadType::Synced,
        )
        .await?;
    }
    Ok(())
}

async fn normalize_legacy_content_files(
    instance: &Instance,
    existing: &[InstanceFile],
    bindings: &HashMap<String, crate::state::content_store::Binding>,
    scanned: &[filesystem::ScannedContentFile],
    state: &State,
) -> crate::Result<bool> {
    let mut renames = Vec::new();
    for scanned in scanned {
        let canonical = scanned.relative_path.trim_end_matches(".disabled");
        let disabled = format!("{canonical}.disabled");
        let source =
            existing.iter().find(|file| file.relative_path == disabled);
        let target =
            existing.iter().find(|file| file.relative_path == canonical);
        let Some(source) = source.or(target) else {
            continue;
        };
        if bindings.contains_key(&source.id)
            || target.is_some_and(|file| bindings.contains_key(&file.id))
            || source.relative_path == canonical
                && source.enabled == scanned.enabled
        {
            continue;
        }
        let opposite = materialized_content_path(canonical, !scanned.enabled);
        let opposite = state
            .directories
            .instances_dir()
            .join(&instance.path)
            .join(opposite);
        match tokio::fs::symlink_metadata(opposite).await {
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        renames.push((
            source,
            canonical,
            scanned.file_name.trim_end_matches(".disabled"),
            scanned.enabled,
        ));
    }
    if renames.is_empty() {
        return Ok(false);
    }
    let mut tx = state.pool.begin().await?;
    for (source, canonical, file_name, enabled) in renames {
        if let Some(file) = sqlite::content_rows::rename_instance_file(
            &instance.id,
            &source.relative_path,
            canonical,
            file_name,
            enabled,
            &mut tx,
        )
        .await?
            && let Some(content_set_id) = &instance.applied_content_set_id
        {
            sqlite::content_rows::set_content_entry_enabled_for_file(
                content_set_id,
                &file.id,
                enabled,
                &mut tx,
            )
            .await?;
        }
    }
    tx.commit().await?;
    Ok(true)
}

pub(crate) fn project_type_for_file(
    file: &InstanceFile,
) -> Option<ProjectType> {
    filesystem::project_type_from_relative_path(&file.relative_path)
}

fn instance_file_id() -> String {
    format!("instance-file:{}", Uuid::new_v4())
}
