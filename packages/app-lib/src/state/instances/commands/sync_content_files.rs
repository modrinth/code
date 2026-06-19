use crate::State;
use crate::state::instances::adapters::{filesystem, sqlite};
use crate::state::instances::{Instance, InstanceFile};
use crate::state::{CachedEntry, ProjectType};
use chrono::Utc;
use std::collections::HashMap;

pub(crate) async fn sync_content_files(
	instance_id: &str,
	state: &State,
) -> crate::Result<Vec<InstanceFile>> {
	let instance = sqlite::instance_rows::get_instance_by_id(instance_id, &state.pool)
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
	let hashes_by_path = hashes
		.into_iter()
		.map(|hash| (hash.path.clone(), hash))
		.collect::<HashMap<_, _>>();

	let now = Utc::now();
	let mut files = Vec::new();

	for file in scanned {
		let cache_path = format!("{}/{}", instance.path, file.relative_path);
		let Some(hash) = hashes_by_path.get(&cache_path) else {
			continue;
		};

		files.push(InstanceFile {
			id: instance_file_id(&instance.id, &file.relative_path),
			instance_id: instance.id.clone(),
			relative_path: file.relative_path,
			file_name: file.file_name,
			enabled: file.enabled,
			sha1: hash.hash.clone(),
			size: file.size,
			missing: false,
			added_at: now,
			modified_at: now,
		});
	}

	let mut tx = state.pool.begin().await?;
	sqlite::content_rows::mark_instance_files_missing(&instance.id, &mut tx)
		.await?;

	for file in &files {
		sqlite::content_rows::upsert_instance_file(file, &mut tx).await?;
	}

	tx.commit().await?;

	Ok(files)
}

pub(crate) fn project_type_for_file(
	file: &InstanceFile,
) -> Option<ProjectType> {
	filesystem::project_type_from_relative_path(&file.relative_path)
}

fn instance_file_id(instance_id: &str, relative_path: &str) -> String {
	format!("instance-file:{instance_id}:{relative_path}")
}
