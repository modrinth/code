use super::super::options_file::{input_error, sha1_bytes};
use super::archive::{self, ArchiveIndex};
use crate::state::{CachedEntry, InstanceMetadata, Project, State};
use crate::util::{fetch, io};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct ArchiveSource {
    pub hash: String,
    pub path: PathBuf,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Snapshot {
    pub instance_id: String,
    pub game_version: String,
    pub mods: Vec<ArchiveSource>,
}

pub(super) async fn mod_projects(
    state: &State,
    hashes: &[&str],
) -> crate::Result<HashMap<String, Project>> {
    let files = CachedEntry::get_file_many(
        hashes,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let project_ids: std::collections::HashSet<_> =
        files.iter().map(|file| file.project_id.as_str()).collect();
    let projects = CachedEntry::get_project_many(
        &project_ids.into_iter().collect::<Vec<_>>(),
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let projects: HashMap<_, _> = projects
        .into_iter()
        .map(|project| (project.id.clone(), project))
        .collect();
    Ok(files
        .into_iter()
        .filter_map(|file| {
            projects
                .get(&file.project_id)
                .cloned()
                .map(|project| (file.hash, project))
        })
        .collect())
}

pub(super) fn root(state: &State) -> PathBuf {
    state
        .directories
        .metadata_dir()
        .join("game-locales")
        .join("v1")
}

pub(super) async fn write_json<T: Serialize>(
    path: &std::path::Path,
    data: &T,
) -> crate::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| input_error("Missing locale cache directory"))?;
    io::create_dir_all(parent).await?;
    io::write(path, serde_json::to_vec(data)?).await?;
    Ok(())
}

pub(super) async fn snapshot_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<String> {
    let version = &metadata.applied_content_set.game_version;
    let instance_dir = state
        .directories
        .instances_dir()
        .join(&metadata.instance.path);
    let scanned =
        crate::state::instances::adapters::filesystem::scan_content_files(
            &state.directories.instances_dir(),
            &metadata.instance.path,
        )?;
    let scanned: Vec<_> = scanned
        .into_iter()
        .filter(|f| f.enabled && f.relative_path.starts_with("mods/"))
        .collect();
    let keys: Vec<_> =
        scanned.iter().map(|f| f.hash_cache_key.as_str()).collect();
    let hashes = CachedEntry::get_file_hash_many(
        &keys,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let by_path: BTreeMap<_, _> =
        hashes.into_iter().map(|h| (h.path, h.hash)).collect();
    let mut mods = Vec::new();
    for file in scanned {
        let path = instance_dir.join(&file.relative_path);
        let key = format!("{}/{}", metadata.instance.path, file.relative_path);
        let hash = match by_path.get(&key) {
            Some(hash) => hash.clone(),
            None => fetch::sha1_file_async(&path).await?.1,
        };
        mods.push(ArchiveSource { hash, path });
    }
    mods.sort_by(|a, b| a.path.cmp(&b.path));
    let snapshot = Snapshot {
        instance_id: metadata.instance.id.clone(),
        game_version: version.clone(),
        mods,
    };
    let bytes = serde_json::to_vec(&snapshot)?;
    let id = sha1_bytes(&bytes);
    let path = root(state).join("snapshots").join(format!("{id}.json"));
    if !io::read(&path).await.is_ok_and(|cached| cached == bytes) {
        tracing::info!(
            snapshot_id = id,
            instance_id = metadata.instance.id,
            "Game setting locales: writing missing or invalid snapshot cache"
        );
        write_json(&path, &snapshot).await?;
    }
    Ok(id)
}

#[tracing::instrument(skip_all, fields(snapshot_id = id), err)]
pub(super) async fn load_snapshot(
    state: &State,
    id: &str,
) -> crate::Result<Snapshot> {
    if !archive::valid_hash(id) {
        return Err(input_error("Invalid locale snapshot hash"));
    }
    let bytes =
        io::read(root(state).join("snapshots").join(format!("{id}.json")))
            .await?;
    archive::checked_bytes(bytes.clone(), id).inspect_err(|error| {
		tracing::warn!(%error, expected_hash = id, actual_hash = sha1_bytes(&bytes),
			"Game setting locales: snapshot checksum mismatch");
	})?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(super) async fn archive_index(
    state: &State,
    source: &ArchiveSource,
) -> crate::Result<ArchiveIndex> {
    if !archive::valid_hash(&source.hash) {
        return Err(input_error("Invalid locale archive hash"));
    }
    let path = root(state)
        .join("archives")
        .join(format!("{}.json", source.hash));
    if let Ok(bytes) = io::read(&path).await
        && let Ok(index) = serde_json::from_slice::<ArchiveIndex>(&bytes)
        && (index.version >= 2 || !source.path.exists())
    {
        tracing::debug!(
            hash = source.hash,
            version = index.version,
            bundles = index.bundles.len(),
            "Game setting locales: archive cache hit"
        );
        return Ok(index);
    }
    tracing::info!(
        hash = source.hash,
        "Game setting locales: rebuilding archive cache"
    );
    let source = source.clone();
    let index = tokio::task::spawn_blocking(move || {
        archive::inspect(&source.path, &source.hash)
    })
    .await
    .map_err(|e| input_error(e.to_string()))??;
    write_json(&path, &index).await?;
    Ok(index)
}
