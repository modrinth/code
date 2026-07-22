use crate::State;
use crate::state::instances::adapters::{filesystem, sqlite};
use crate::state::instances::{Instance, InstanceFile};
use crate::state::{CachedEntry, ProjectType};
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
    let scanned = filesystem::scan_content_files(
        &state.directories.instances_dir(),
        &instance.path,
    )?;
    let cache_keys = scanned
        .iter()
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
                format!(
                    "{}-{}",
                    hash.size,
                    hash.path.trim_end_matches(".disabled")
                ),
                hash,
            )
        })
        .collect::<HashMap<_, _>>();
    let existing_files =
        sqlite::content_rows::get_instance_files(&instance.id, &state.pool)
            .await?;
    let scanned_paths = scanned
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<HashSet<_>>();
    let missing_file_ids = existing_files
        .iter()
        .filter(|file| {
            !file.missing && !scanned_paths.contains(&file.relative_path)
        })
        .map(|file| file.id.clone())
        .collect::<Vec<_>>();
    let existing_files_by_path = existing_files
        .into_iter()
        .map(|file| (file.relative_path.clone(), file))
        .collect::<HashMap<_, _>>();

    let now = Utc::now();
    let mut files = Vec::new();
    let mut present_without_hash_ids = Vec::new();

    for file in scanned {
        let hash_key = file.hash_cache_key.trim_end_matches(".disabled");
        let existing_file = existing_files_by_path.get(&file.relative_path);
        let Some(hash) = hashes_by_key.get(hash_key) else {
            if let Some(existing_file) = existing_file {
                present_without_hash_ids.push(existing_file.id.clone());
            }
            continue;
        };

        files.push(InstanceFile {
            id: existing_file
                .map(|file| file.id.clone())
                .unwrap_or_else(instance_file_id),
            instance_id: instance.id.clone(),
            relative_path: file.relative_path,
            file_name: file.file_name,
            enabled: file.enabled,
            sha1: hash.hash.clone(),
            size: file.size,
            missing: false,
            added_at: existing_file.map(|file| file.added_at).unwrap_or(now),
            modified_at: now,
        });
    }

    let mut tx = state.pool.begin().await?;
    for file_id in missing_file_ids {
        sqlite::content_rows::set_instance_file_missing(
            &file_id, true, &mut tx,
        )
        .await?;
    }

    let mut stored_files =
        Vec::with_capacity(files.len() + present_without_hash_ids.len());
    for file_id in present_without_hash_ids {
        if let Some(file) = sqlite::content_rows::set_instance_file_missing(
            &file_id, false, &mut tx,
        )
        .await?
        {
            stored_files.push(file);
        }
    }
    for file in &files {
        stored_files.push(
            sqlite::content_rows::upsert_instance_file(file, &mut tx).await?,
        );
    }

    tx.commit().await?;

    Ok(stored_files)
}

pub(crate) fn project_type_for_file(
    file: &InstanceFile,
) -> Option<ProjectType> {
    filesystem::project_type_from_relative_path(&file.relative_path)
}

fn instance_file_id() -> String {
    format!("instance-file:{}", Uuid::new_v4())
}
