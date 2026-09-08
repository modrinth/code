//! Caches game translations and remembers which JAR supplied each setting.

mod archive;
mod catalog;
mod sources;
mod storage;

use super::catalog::setting_by_file_key;
use super::options_file::{GameOptionsDocument, input_error, options_path, read_document};
use crate::state::{InstanceMetadata, State};
use archive::{ArchiveIndex, Translations};
use serde::{Deserialize, Serialize};
use sources::{ArchiveSource, AssetIndexSource, Snapshot};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::Notify;

#[derive(Default)]
pub(crate) struct GameLocaleIndexer {
	notify: Notify,
	started: AtomicBool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Observation {
	snapshot: String,
	raw_key: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Origin {
	instance_id: String,
	game_version: String,
	game_jar_hash: String,
	asset_index: Option<AssetIndexSource>,
	archive: ArchiveSource,
	nested_path: String,
	translation_key: String,
	choices: BTreeMap<String, String>,
}

#[derive(Default, Serialize)]
pub struct GameSettingLocaleLabels {
	pub settings: BTreeMap<String, GameSettingLocaleLabel>,
}

#[derive(Serialize)]
pub struct GameSettingLocaleLabel {
	pub label: String,
	pub choices: BTreeMap<String, String>,
}

struct Candidate {
	snapshot_id: String,
	snapshot: Snapshot,
	keys: BTreeMap<String, String>,
}

pub(crate) fn queue_game_locale_index() {
	if let Some(state) = State::get_if_initialized() {
		state.game_locale_indexer.notify.notify_one();
	}
}

pub(crate) fn start_game_locale_indexer(state: Arc<State>) {
	if state.game_locale_indexer.started.swap(true, Ordering::AcqRel) { return; }
	state.game_locale_indexer.notify.notify_one();
	tokio::spawn(async move {
		loop {
			state.game_locale_indexer.notify.notified().await;
			tokio::time::sleep(std::time::Duration::from_millis(300)).await;
			if let Err(error) = index_installed_sources(&state).await {
				tracing::warn!(%error, "Could not index game-setting translations");
			}
			#[cfg(feature = "tauri")]
			{
				use tauri::Emitter;
				let _ = crate::EventState::get().app.emit("game-option-locales-updated", ());
			}
		}
	});
}

fn document_keys(document: &GameOptionsDocument) -> BTreeMap<String, String> {
	document.effective_entries().keys().map(|key| {
		let id = setting_by_file_key(key).map(|s| s.id.to_owned())
			.unwrap_or_else(|| format!("external:{key}"));
		(id, (*key).to_owned())
	}).filter(|(id, key)| !catalog::translation_keys(key, !id.starts_with("external:")).is_empty()).collect()
}

/// Records Minecraft and mod JAR hashes before saving new options.
pub(super) async fn capture_observation(metadata: &InstanceMetadata, state: &State) -> Option<String> {
	match sources::snapshot_instance(metadata, state).await {
		Ok(id) => Some(id),
		Err(error) => {
			tracing::debug!(%error, "Game-setting locale source is not available yet");
			None
		}
	}
}

pub(super) async fn record_observations(
	tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, metadata: &InstanceMetadata,
	document: &GameOptionsDocument, snapshot: Option<&str>,
) {
	let Some(snapshot) = snapshot else { return; };
	for (id, raw_key) in document_keys(document) {
		let observation = Observation { snapshot: snapshot.to_owned(), raw_key };
		if let Err(error) = storage::observe(tx, "", &id, &metadata.instance.id,
			&metadata.applied_content_set.game_version, &observation).await {
			tracing::warn!(%error, "Could not record game-setting translation provenance");
		}
	}
}

async fn cached_archive(
	state: &State, source: &ArchiveSource, cache: &mut HashMap<String, Arc<ArchiveIndex>>,
) -> crate::Result<Arc<ArchiveIndex>> {
	if let Some(index) = cache.get(&source.hash) { return Ok(index.clone()); }
	let index = Arc::new(sources::archive_index(state, source).await?);
	if cache.len() >= 8 { cache.clear(); }
	cache.insert(source.hash.clone(), index.clone());
	Ok(index)
}

async fn index_installed_sources(state: &State) -> crate::Result<()> {
	let mut instances = crate::state::list_instances(&state.pool).await?;
	instances.sort_by(|a, b| a.instance.id.cmp(&b.instance.id));
	let mut candidates = Vec::new();
	let mut archives = HashMap::new();
	for metadata in instances {
		if super::super::sync_files_are_protected(&metadata) { continue; }
		let Some(snapshot_id) = capture_observation(&metadata, state).await else { continue; };
		let snapshot = sources::load_snapshot(state, &snapshot_id).await?;
		for source in std::iter::once(&snapshot.game_jar).chain(&snapshot.mods) {
			if let Err(error) = cached_archive(state, source, &mut archives).await {
				tracing::debug!(hash = source.hash, %error, "Could not index locale archive");
			}
		}
		if let Some(index) = &snapshot.asset_index {
			let _ = sources::language_assets(state, index).await;
		}
		let Ok((document, _)) = read_document(&options_path(&metadata, state)).await else { continue; };
		let keys = document_keys(&document);
		let mut tx = state.pool.begin().await?;
		for (id, key) in &keys {
			let observation = Observation { snapshot: snapshot_id.clone(), raw_key: key.clone() };
			storage::observe(&mut tx, &metadata.instance.id, id, &metadata.instance.id,
				&snapshot.game_version, &observation).await?;
		}
		tx.commit().await?;
		candidates.push(Candidate { snapshot_id, snapshot, keys });
	}
	for row in storage::load(&state.pool, None).await? {
		if row.origin.is_some() { continue; }
		let mut observations = Vec::new();
		if let Some(observation) = &row.observation {
			observations.push(observation.clone());
		} else {
			if row.backfilled
				&& let Some(version) = &row.source_game_version
				&& let Some(definition) = super::catalog::setting_by_id(&row.option_id)
				&& let Ok(Some(snapshot)) = sources::historical_snapshot(
					state, row.source_instance_id.as_deref().unwrap_or(""), version,
				).await {
				observations.extend(definition.keys.iter().map(|key| Observation {
					snapshot: snapshot.clone(), raw_key: (*key).to_owned(),
				}));
			}
			let mut matching: Vec<_> = candidates.iter().filter(|c| {
				(row.scope.is_empty() || row.scope == c.snapshot.instance_id)
					&& c.keys.contains_key(&row.option_id)
					&& (row.backfilled || row.source_instance_id.as_ref().is_none_or(|id| *id == c.snapshot.instance_id)
						&& row.source_game_version.as_ref().is_none_or(|v| *v == c.snapshot.game_version))
			}).collect();
			matching.sort_by_key(|c| (
				row.source_instance_id.as_deref() != Some(c.snapshot.instance_id.as_str()),
				row.source_game_version.as_deref() != Some(c.snapshot.game_version.as_str()),
				c.snapshot.instance_id.clone(),
			));
			observations.extend(matching.into_iter().map(|c| Observation {
				snapshot: c.snapshot_id.clone(), raw_key: c.keys[&row.option_id].clone(),
			}));
		}
		for observation in observations {
			match resolve_origin(state, &row.option_id, &observation, &mut archives).await {
				Ok(Some(origin)) => { storage::pin(&state.pool, &row, &origin).await?; break; }
				Ok(None) => {}
				Err(error) => tracing::debug!(%error, option_id = row.option_id, "Translation origin remains unresolved"),
			}
		}
	}
	Ok(())
}

async fn resolve_origin(
	state: &State, option_id: &str, observation: &Observation,
	archives: &mut HashMap<String, Arc<ArchiveIndex>>,
) -> crate::Result<Option<Origin>> {
	let snapshot = sources::load_snapshot(state, &observation.snapshot).await?;
	let vanilla = !option_id.starts_with("external:");
	let keys = catalog::translation_keys(&observation.raw_key, vanilla);
	let sources: Vec<_> = if vanilla { vec![&snapshot.game_jar] }
		else { snapshot.mods.iter().chain(std::iter::once(&snapshot.game_jar)).collect() };
	for source in sources {
		let Ok(index) = cached_archive(state, source, archives).await else { continue; };
		for bundle in &index.bundles {
			let Some(english) = bundle.locales.get("en_us") else { continue; };
			let mut english = english.clone();
			bundle.deprecated.apply(&mut english);
			for key in &keys {
				if !english.get(key).is_some_and(|s| plain_label(s).is_some()) { continue; }
				let asset_index = if source.hash == snapshot.game_jar.hash { snapshot.asset_index.clone() } else { None };
				if let Some(index) = &asset_index {
					sources::language_assets(state, index).await?;
				}
				return Ok(Some(Origin {
					instance_id: snapshot.instance_id.clone(), game_version: snapshot.game_version.clone(),
					game_jar_hash: snapshot.game_jar.hash.clone(), asset_index,
					archive: source.clone(), nested_path: bundle.nested_path.clone(), translation_key: key.clone(),
					choices: catalog::choice_keys(option_id).iter().filter(|(_, key)| english.contains_key(*key))
						.map(|(value, key)| ((*value).to_owned(), (*key).to_owned())).collect(),
				}));
			}
		}
	}
	Ok(None)
}

fn plain_label(value: &str) -> Option<String> {
	if value.trim().is_empty() || value.contains('%') || value.contains('§') { return None; }
	Some(value.to_owned())
}

fn minecraft_locale(locale: &str) -> String {
	match locale {
		"es-419" => "es_mx".to_owned(),
		"fil-PH" => "tl_ph".to_owned(),
		"ms-MY" => "ms_my".to_owned(),
		"zh-Hans" => "zh_cn".to_owned(),
		"zh-Hant" => "zh_tw".to_owned(),
		_ => locale.replace('-', "_").to_ascii_lowercase(),
	}
}

/// Loads translations for the settings shown in the modal.
pub async fn get_game_setting_locale_labels(
	instance_id: Option<&str>, locale: &str, option_ids: Vec<String>, refresh_sources: bool,
) -> crate::Result<GameSettingLocaleLabels> {
	if option_ids.len() > 16_384 { return Err(input_error("Too many requested game-setting labels")); }
	let locale = minecraft_locale(locale);
	if !archive::valid_locale(&locale) { return Err(input_error("Invalid Minecraft locale")); }
	let state = State::get().await?;
	if refresh_sources { queue_game_locale_index(); }
	let rows = storage::load(&state.pool, Some(instance_id.unwrap_or(""))).await?;
	let requested: std::collections::HashSet<_> = option_ids.into_iter().collect();
	let mut result = GameSettingLocaleLabels::default();
	let mut archives = HashMap::new();
	let mut assets: HashMap<String, Translations> = HashMap::new();
	let mut dictionaries: HashMap<String, Translations> = HashMap::new();
	for row in rows {
		if !requested.contains(&row.option_id) { continue; }
		let Some(mut origin) = row.origin else { continue; };
		for (value, key) in catalog::choice_keys(&row.option_id) {
			origin.choices.entry((*value).to_owned()).or_insert_with(|| (*key).to_owned());
		}
		let dictionary_id = format!("{}:{}:{}", origin.archive.hash, origin.nested_path,
			origin.asset_index.as_ref().map(|a| a.hash.as_str()).unwrap_or(""));
		if !dictionaries.contains_key(&dictionary_id) {
			let Ok(index) = cached_archive(&state, &origin.archive, &mut archives).await else { continue; };
			let Some(bundle) = index.bundles.iter().find(|b| b.nested_path == origin.nested_path) else { continue; };
			let mut translations = bundle.locales.get("en_us").cloned().unwrap_or_default();
			if let Some(selected) = bundle.locales.get(&locale) { translations.extend(selected.clone()); }
			if locale != "en_us" && let Some(index) = &origin.asset_index
				&& let Ok(available) = sources::language_assets(&state, index).await {
				for name in [format!("{locale}.json"), format!("{locale}.lang")] {
					if let Some(asset) = available.get(&name) {
						if !assets.contains_key(&asset.hash) {
							let selected = sources::asset_language(&state, asset, name.ends_with(".lang")).await.unwrap_or_default();
							assets.insert(asset.hash.clone(), selected);
						}
						translations.extend(assets[&asset.hash].clone());
						break;
					}
				}
			}
			bundle.deprecated.apply(&mut translations);
			dictionaries.insert(dictionary_id.clone(), translations);
		}
		let translations = &dictionaries[&dictionary_id];
		let label = translations.get(&origin.translation_key).and_then(|s| plain_label(s));
		if let Some(label) = label {
			let choices = origin.choices.iter().filter_map(|(value, key)| {
				translations.get(key).and_then(|s| plain_label(s)).map(|label| (value.clone(), label))
			}).collect();
			result.settings.insert(row.option_id, GameSettingLocaleLabel { label, choices });
		}
	}
	Ok(result)
}
