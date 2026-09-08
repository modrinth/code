use super::super::options_file::{input_error, sha1_bytes};
use super::archive::{self, ArchiveIndex, Translations};
use crate::state::{CachedEntry, InstanceMetadata, State};
use crate::util::{fetch, io};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct ArchiveSource {
	pub hash: String,
	pub path: PathBuf,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct AssetIndexSource {
	pub hash: String,
	pub id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Snapshot {
	pub instance_id: String,
	pub game_version: String,
	pub game_jar: ArchiveSource,
	pub asset_index: Option<AssetIndexSource>,
	pub mods: Vec<ArchiveSource>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Asset {
	pub hash: String,
	pub size: usize,
}

pub(super) fn root(state: &State) -> PathBuf {
	state.directories.metadata_dir().join("game-locales").join("v1")
}

pub(super) async fn write_json<T: Serialize>(path: &std::path::Path, data: &T) -> crate::Result<()> {
	let parent = path.parent().ok_or_else(|| input_error("Missing locale cache directory"))?;
	io::create_dir_all(parent).await?;
	io::write(path, serde_json::to_vec(data)?).await?;
	Ok(())
}

pub(super) async fn snapshot_instance(metadata: &InstanceMetadata, state: &State) -> crate::Result<String> {
	let version = &metadata.applied_content_set.game_version;
	let version_id = metadata.applied_content_set.loader_version.as_ref()
		.map(|loader| format!("{version}-{loader}")).unwrap_or_else(|| version.clone());
	let mut found = None;
	for id in [&version_id, version] {
		let directory = state.directories.version_dir(id);
		let path = directory.join(format!("{id}.json"));
		let Ok(bytes) = io::read(&path).await else { continue; };
		let value: serde_json::Value = serde_json::from_slice(&bytes)?;
		let Some(hash) = value.pointer("/downloads/client/sha1").and_then(|v| v.as_str()) else { continue; };
		if !archive::valid_hash(hash) { continue; }
		let jar_id = value.get("id").and_then(|v| v.as_str()).unwrap_or(id);
		let jar = state.directories.version_dir(jar_id).join(format!("{jar_id}.jar"));
		let asset_index = value.get("assetIndex").and_then(|v| Some(AssetIndexSource {
			hash: v.get("sha1")?.as_str()?.to_owned(),
			id: v.get("id")?.as_str()?.to_owned(),
		})).filter(|a| archive::valid_hash(&a.hash) && !a.id.contains(['/', '\\']));
		found = Some((ArchiveSource { hash: hash.to_owned(), path: jar }, asset_index));
		break;
	}
	let (game_jar, asset_index) = found.ok_or_else(|| input_error("Minecraft locale source is not installed yet"))?;
	let instance_dir = state.directories.instances_dir().join(&metadata.instance.path);
	let scanned = crate::state::instances::adapters::filesystem::scan_content_files(
		&state.directories.instances_dir(), &metadata.instance.path,
	)?;
	let scanned: Vec<_> = scanned.into_iter().filter(|f| f.enabled && f.relative_path.starts_with("mods/")).collect();
	let keys: Vec<_> = scanned.iter().map(|f| f.hash_cache_key.as_str()).collect();
	let hashes = CachedEntry::get_file_hash_many(&keys, None, &state.pool, &state.api_semaphore).await?;
	let by_path: BTreeMap<_, _> = hashes.into_iter().map(|h| (h.path, h.hash)).collect();
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
	let snapshot = Snapshot { instance_id: metadata.instance.id.clone(), game_version: version.clone(), game_jar, asset_index, mods };
	let bytes = serde_json::to_vec(&snapshot)?;
	let id = sha1_bytes(&bytes);
	let path = root(state).join("snapshots").join(format!("{id}.json"));
	if !path.exists() { write_json(&path, &snapshot).await?; }
	Ok(id)
}

pub(super) async fn load_snapshot(state: &State, id: &str) -> crate::Result<Snapshot> {
	if !archive::valid_hash(id) { return Err(input_error("Invalid locale snapshot hash")); }
	let bytes = io::read(root(state).join("snapshots").join(format!("{id}.json"))).await?;
	archive::checked_bytes(bytes.clone(), id)?;
	Ok(serde_json::from_slice(&bytes)?)
}

/// Old synced values retain a Minecraft version but no archive hashes. Prefer
/// that version's installed resources before considering another instance.
pub(super) async fn historical_snapshot(
	state: &State, instance_id: &str, version: &str,
) -> crate::Result<Option<String>> {
	let mut directories = tokio::fs::read_dir(state.directories.versions_dir())
		.await.map_err(io::IOError::from)?;
	let mut ids = Vec::new();
	while let Some(entry) = directories.next_entry().await.map_err(io::IOError::from)? {
		let id = entry.file_name().to_string_lossy().into_owned();
		if id == version || id.starts_with(&format!("{version}-")) { ids.push(id); }
	}
	ids.sort();
	for id in ids {
		let directory = state.directories.version_dir(&id);
		let Ok(bytes) = io::read(directory.join(format!("{id}.json"))).await else { continue; };
		let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else { continue; };
		let Some(hash) = value.pointer("/downloads/client/sha1").and_then(|v| v.as_str()) else { continue; };
		if !archive::valid_hash(hash) { continue; }
		let jar_id = value.get("id").and_then(|v| v.as_str()).unwrap_or(&id);
		let asset_index = value.get("assetIndex").and_then(|v| Some(AssetIndexSource {
			hash: v.get("sha1")?.as_str()?.to_owned(), id: v.get("id")?.as_str()?.to_owned(),
		})).filter(|a| archive::valid_hash(&a.hash) && !a.id.contains(['/', '\\']));
		let snapshot = Snapshot {
			instance_id: instance_id.to_owned(), game_version: version.to_owned(),
			game_jar: ArchiveSource { hash: hash.to_owned(), path: state.directories.version_dir(jar_id).join(format!("{jar_id}.jar")) },
			asset_index, mods: Vec::new(),
		};
		if archive_index(state, &snapshot.game_jar).await.is_err() { continue; }
		let id = sha1_bytes(&serde_json::to_vec(&snapshot)?);
		write_json(&root(state).join("snapshots").join(format!("{id}.json")), &snapshot).await?;
		return Ok(Some(id));
	}
	Ok(None)
}

pub(super) async fn archive_index(state: &State, source: &ArchiveSource) -> crate::Result<ArchiveIndex> {
	if !archive::valid_hash(&source.hash) { return Err(input_error("Invalid locale archive hash")); }
	let path = root(state).join("archives").join(format!("{}.json", source.hash));
	if let Ok(bytes) = io::read(&path).await
		&& let Ok(index) = serde_json::from_slice::<ArchiveIndex>(&bytes)
		&& (index.version >= 2 || !source.path.exists()) { return Ok(index); }
	let source = source.clone();
	let index = tokio::task::spawn_blocking(move || archive::inspect(&source.path, &source.hash))
		.await.map_err(|e| input_error(e.to_string()))??;
	write_json(&path, &index).await?;
	Ok(index)
}

pub(super) async fn language_assets(state: &State, source: &AssetIndexSource) -> crate::Result<BTreeMap<String, Asset>> {
	if !archive::valid_hash(&source.hash) { return Err(input_error("Invalid locale asset index hash")); }
	let path = root(state).join("indexes").join(format!("{}.json", source.hash));
	if let Ok(bytes) = io::read(&path).await { return Ok(serde_json::from_slice(&bytes)?); }
	let bytes = io::read(state.directories.assets_index_dir().join(format!("{}.json", source.id))).await?;
	let bytes = archive::checked_bytes(bytes, &source.hash)?;
	let value: serde_json::Value = serde_json::from_slice(&bytes)?;
	let mut assets = BTreeMap::new();
	for (name, asset) in value.get("objects").and_then(|v| v.as_object()).into_iter().flatten() {
		let Some(name) = name.strip_prefix("minecraft/lang/") else { continue; };
		let Some(locale) = name.strip_suffix(".json").or_else(|| name.strip_suffix(".lang")) else { continue; };
		let asset: Asset = serde_json::from_value(asset.clone())?;
		if archive::valid_locale(locale) && archive::valid_hash(&asset.hash) && asset.size <= archive::MAX_LANGUAGE_BYTES {
			assets.insert(name.to_ascii_lowercase(), asset);
		}
	}
	write_json(&path, &assets).await?;
	Ok(assets)
}

pub(super) async fn asset_language(state: &State, asset: &Asset, legacy: bool) -> crate::Result<Translations> {
	if !archive::valid_hash(&asset.hash) || asset.size > archive::MAX_LANGUAGE_BYTES {
		return Err(input_error("Invalid Minecraft locale asset"));
	}
	let path = root(state).join("assets").join(&asset.hash);
	let bytes = match io::read(&path).await {
		Ok(bytes) => bytes,
		Err(_) => {
			let bytes = match io::read(state.directories.object_dir(&asset.hash)).await {
				Ok(bytes) => bytes,
				Err(_) => fetch::fetch(
					&format!("https://resources.download.minecraft.net/{}/{}", &asset.hash[..2], asset.hash),
					Some(&asset.hash), None, None, &state.fetch_semaphore, &state.pool,
				).await?.to_vec(),
			};
			if bytes.len() != asset.size { return Err(input_error("Minecraft locale asset has unexpected size")); }
			archive::checked_bytes(bytes.clone(), &asset.hash)?;
			io::create_dir_all(path.parent().unwrap()).await?;
			io::write(&path, &bytes).await?;
			bytes
		}
	};
	archive::checked_bytes(bytes.clone(), &asset.hash)?;
	archive::parse_language(&bytes, legacy)
}
