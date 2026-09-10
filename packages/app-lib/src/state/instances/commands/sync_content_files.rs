use crate::State;
use crate::state::instances::adapters::{filesystem, sqlite};
use crate::state::instances::{Instance, InstanceFile};
use crate::state::{CachedEntry, ProjectType, file_hash_cache_key};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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
    let _content_lock = state.lock_instance_content(&instance.id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    state.content_store.recover(Some(&instance.id)).await?;
    let existing =
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
    let existing_by_path = existing
        .iter()
        .map(|file| (file.relative_path.clone(), file))
        .collect::<HashMap<_, _>>();
    let mut scanned = filesystem::scan_content_files(
        &state.directories.instances_dir(),
        &instance.path,
    )?;
    scanned.retain(|file| {
        let registered = existing_by_path
            .get(&file.relative_path)
            .is_some_and(|file| bindings.contains_key(&file.id));
        registered || !file.is_symlink
    });
    let cache_keys = scanned
        .iter()
        .filter(|file| !file.is_symlink)
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
    let scanned_by_path = scanned
        .iter()
        .map(|file| (file.relative_path.as_str(), file))
        .collect::<HashMap<_, _>>();
    let running =
        crate::state::instance_has_running_process(&instance.id, state).await?;
    let mut files = Vec::new();
    for previous in &existing {
        let Some(binding) = bindings.get(&previous.id) else {
            continue;
        };
        let mut file = previous.clone();
        let blob = state.content_store.file_blob(previous).await?;
        let observed = scanned_by_path.get(file.relative_path.as_str());
        let matches = if let Some(observed) = observed {
            if observed.is_symlink {
                let path = state
                    .content_store
                    .instance_path(&instance.path, &file.relative_path)
                    .await?;
                state
                    .content_store
                    .matches(&path, &binding.blob_sha512)
                    .await?
            } else {
                hashes_by_key
                    .get(observed.hash_cache_key.trim_end_matches(".disabled"))
                    .is_some_and(|hash| hash.hash == file.sha1)
            }
        } else {
            false
        };
        file.missing = file.enabled && (blob.is_none() || !matches);
        if !file.enabled && observed.is_some() {
            tracing::warn!(instance_id = %instance.id, path = %file.relative_path, "Disabled content has an external file at its reserved path");
        }
        if file.missing != previous.missing {
            file.modified_at = Utc::now();
        }
        files.push(file);
    }
    for scanned in &scanned {
        let previous = existing_by_path.get(&scanned.relative_path).copied();
        if previous.is_some_and(|file| bindings.contains_key(&file.id)) {
            continue;
        }
        let Some(hash) = hashes_by_key
            .get(scanned.hash_cache_key.trim_end_matches(".disabled"))
        else {
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
        if !running
            && crate::state::content_store::eligible(&file.relative_path)
        {
            file = crate::state::content_store::migration::adopt_file(
                instance, &file, state,
            )
            .await?;
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
    let changed = !missing.is_empty()
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
        stored.push(
            sqlite::content_rows::upsert_instance_file(&file, &mut tx).await?,
        );
    }
    tx.commit().await?;
    if changed {
        super::mark_shared_instance_stale(&instance.id, &state.pool).await?;
        crate::api::instance::queue_game_locale_index();
    }
    Ok(stored)
}

pub(crate) fn project_type_for_file(
    file: &InstanceFile,
) -> Option<ProjectType> {
    filesystem::project_type_from_relative_path(&file.relative_path)
}

fn instance_file_id() -> String {
    format!("instance-file:{}", Uuid::new_v4())
}
