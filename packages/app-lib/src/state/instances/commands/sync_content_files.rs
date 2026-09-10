use crate::State;
use crate::state::content_store::{
	ContentProjectionStatus, FileContent, content_file_path,
};
use crate::state::instances::adapters::{filesystem, sqlite};
use crate::state::instances::{Instance, InstanceFile};
use crate::state::{CachedEntry, ProjectType, file_hash_cache_key};
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
	let executor = ContentMutationExecutor::lock(&instance.id, state).await?;
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
        .map(|file| {
			(
				file.relative_path.trim_end_matches(".disabled").to_string(),
				file,
			)
		})
        .collect::<HashMap<_, _>>();
    let mut scanned = filesystem::scan_content_files(
        &state.directories.instances_dir(),
        &instance.path,
    )?;
    scanned.retain(|file| {
        let registered = existing.iter().any(|existing| {
			bindings.contains_key(&existing.id)
				&& content_file_path(existing) == file.relative_path
		});
        registered || !file.is_symlink
    });
	let running =
		crate::state::instance_has_running_process(&instance.id, state).await?;
    let cache_keys = scanned
        .iter()
		.filter(|scanned| running && !scanned.is_symlink)
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
		file.missing = !stored
			|| projection != ContentProjectionStatus::Healthy;
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
		let canonical_path = scanned.relative_path.trim_end_matches(".disabled");
		let previous = existing_by_path.get(canonical_path).copied();
		if let Some(file) = previous
			&& bindings.contains_key(&file.id)
		{
			if content_file_path(file) != scanned.relative_path {
				tracing::warn!(
					instance_id = %instance.id,
					path = %scanned.relative_path,
					"Ignoring content at the inactive form of a managed path"
				);
			}
			continue;
		}
		let hash = hashes_by_key
			.get(scanned.hash_cache_key.trim_end_matches(".disabled"));
		if running && hash.is_none() {
			continue;
		}
        let mut file = InstanceFile {
            id: previous
                .map(|file| file.id.clone())
                .unwrap_or_else(instance_file_id),
            instance_id: instance.id.clone(),
            relative_path: scanned.relative_path.clone(),
            file_name: scanned.file_name.clone(),
            enabled: scanned.enabled,
			sha1: hash.map(|hash| hash.hash.clone()).unwrap_or_default(),
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
			file = match executor
				.execute(ContentMutation::Adopt { file: &file })
				.await?
			{
				ContentMutationResult::File(file) => file,
				_ => unreachable!("adopt mutations return a content file"),
			};
			stored_by_executor.insert(file.id.clone());
		} else {
			file.relative_path = canonical_path.to_string();
			file.file_name = scanned
				.file_name
				.trim_end_matches(".disabled")
				.to_string();
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
		if stored_by_executor.contains(&file.id) {
			stored.push(file);
		} else {
			stored.push(
				sqlite::content_rows::upsert_instance_file(&file, &mut tx).await?,
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

pub(crate) fn project_type_for_file(
    file: &InstanceFile,
) -> Option<ProjectType> {
    filesystem::project_type_from_relative_path(&file.relative_path)
}

fn instance_file_id() -> String {
    format!("instance-file:{}", Uuid::new_v4())
}
