//! Caches mod translations and remembers which JAR supplied each setting.

mod archive;
mod sources;
mod storage;

use super::catalog::{is_never_sync_key, setting_by_file_key};
use super::options_file::{
    GameOptionsDocument, input_error, options_path, read_document,
};
use crate::state::{InstanceMetadata, State};
use archive::{ArchiveIndex, Translations};
use serde::{Deserialize, Serialize};
use sources::{ArchiveSource, Snapshot};
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
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
    /// Rejects origins that older versions resolved from Minecraft instead of a mod.
    #[serde(default, rename = "game_jar_hash", skip_serializing)]
    legacy_game_jar_hash: Option<String>,
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
    pub source: Option<GameSettingSource>,
}

#[derive(Serialize)]
pub struct GameSettingSource {
    pub instance_id: String,
    pub file_name: String,
    pub file_path: String,
    pub project: Option<GameSettingSourceProject>,
}

#[derive(Serialize)]
pub struct GameSettingSourceProject {
    pub id: String,
    pub title: String,
    pub icon_url: Option<String>,
}

struct Candidate {
    snapshot_id: String,
    snapshot: Snapshot,
    keys: BTreeMap<String, String>,
}

pub(crate) fn queue_game_locale_index() {
    if let Some(state) = State::get_if_initialized() {
        tracing::info!(
            started = state.game_locale_indexer.started.load(Ordering::Acquire),
            "Game setting locales: indexing queued"
        );
        state.game_locale_indexer.notify.notify_one();
    } else {
        tracing::warn!(
            "Game setting locales: cannot queue indexing before state initialization"
        );
    }
}

pub(crate) fn start_game_locale_indexer(state: Arc<State>) {
    if state
        .game_locale_indexer
        .started
        .swap(true, Ordering::AcqRel)
    {
        return;
    }
    tracing::info!("Game setting locales: indexer started");
    state.game_locale_indexer.notify.notify_one();
    tokio::spawn(async move {
        loop {
            state.game_locale_indexer.notify.notified().await;
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let started = std::time::Instant::now();
            if let Err(error) = index_installed_sources(&state).await {
                tracing::warn!(%error, "Game setting locales: indexing failed");
            }
            tracing::info!(
                elapsed_ms = started.elapsed().as_millis(),
                "Game setting locales: indexing pass finished"
            );
            #[cfg(feature = "tauri")]
            {
                use tauri::Emitter;
                match crate::EventState::get()
                    .app
                    .emit("game-option-locales-updated", ())
                {
                    Ok(()) => tracing::info!(
                        "Game setting locales: emitted game-option-locales-updated"
                    ),
                    Err(error) => {
                        tracing::warn!(%error, "Game setting locales: update event failed")
                    }
                }
            }
        }
    });
}

fn mod_translation_key(raw_key: &str) -> Option<&str> {
    if setting_by_file_key(raw_key).is_some() || is_never_sync_key(raw_key) {
        return None;
    }
    raw_key.strip_prefix("key_")
}

fn document_keys(document: &GameOptionsDocument) -> BTreeMap<String, String> {
    document
        .effective_entries()
        .keys()
        .filter(|key| mod_translation_key(key).is_some())
        .map(|key| (format!("external:{key}"), (*key).to_owned()))
        .collect()
}

pub(super) fn has_localizable_mod_settings(
    document: &GameOptionsDocument,
) -> bool {
    document
        .effective_entries()
        .keys()
        .any(|key| mod_translation_key(key).is_some())
}

/// Records mod JAR hashes before saving new options.
pub(super) async fn capture_observation(
    metadata: &InstanceMetadata,
    state: &State,
) -> Option<String> {
    match sources::snapshot_instance(metadata, state).await {
        Ok(id) => Some(id),
        Err(error) => {
            tracing::warn!(%error, instance_id = metadata.instance.id,
				"Game setting locales: snapshot capture failed");
            None
        }
    }
}

pub(super) async fn record_observations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    snapshot: Option<&str>,
) {
    let Some(snapshot) = snapshot else {
        return;
    };
    for (id, raw_key) in document_keys(document) {
        let observation = Observation {
            snapshot: snapshot.to_owned(),
            raw_key,
        };
        if let Err(error) = storage::observe(
            tx,
            "",
            &id,
            &metadata.instance.id,
            &metadata.applied_content_set.game_version,
            &observation,
        )
        .await
        {
            tracing::warn!(%error, "Could not record game-setting translation provenance");
        }
    }
}

async fn cached_archive(
    state: &State,
    source: &ArchiveSource,
    cache: &mut HashMap<String, Arc<ArchiveIndex>>,
) -> crate::Result<Arc<ArchiveIndex>> {
    if let Some(index) = cache.get(&source.hash) {
        return Ok(index.clone());
    }
    let index = Arc::new(sources::archive_index(state, source).await?);
    if cache.len() >= 8 {
        cache.clear();
    }
    cache.insert(source.hash.clone(), index.clone());
    Ok(index)
}

#[tracing::instrument(skip_all, err)]
async fn index_installed_sources(state: &State) -> crate::Result<()> {
    let mut instances = crate::state::list_instances(&state.pool).await?;
    tracing::info!(
        instances = instances.len(),
        "Game setting locales: indexing installed instances"
    );
    instances.sort_by(|a, b| a.instance.id.cmp(&b.instance.id));
    let mut candidates = Vec::new();
    let mut archives = HashMap::new();
    for metadata in instances {
        if super::super::sync_files_are_protected(&metadata) {
            tracing::info!(
                instance_id = metadata.instance.id,
                "Game setting locales: skipping protected instance"
            );
            continue;
        }
        let document = match read_document(&options_path(&metadata, state))
            .await
        {
            Ok((document, _)) => document,
            Err(error) => {
                tracing::warn!(%error, instance_id = metadata.instance.id, "Game setting locales: options file read failed");
                continue;
            }
        };
        let keys = document_keys(&document);
        if keys.is_empty() {
            continue;
        }
        let Some(snapshot_id) = capture_observation(&metadata, state).await
        else {
            continue;
        };
        let snapshot = match sources::load_snapshot(state, &snapshot_id).await {
            Ok(snapshot) => snapshot,
            Err(error) => {
                tracing::warn!(%error, instance_id = metadata.instance.id, %snapshot_id,
					"Could not load game-setting locale snapshot");
                continue;
            }
        };
        tracing::info!(instance_id = metadata.instance.id, %snapshot_id, game_version = snapshot.game_version,
			mods = snapshot.mods.len(),
			"Game setting locales: snapshot loaded");
        for source in &snapshot.mods {
            if let Err(error) =
                cached_archive(state, source, &mut archives).await
            {
                tracing::warn!(hash = source.hash, %error, "Game setting locales: archive indexing failed");
            }
        }
        let mut tx = state.pool.begin().await?;
        for (id, key) in &keys {
            let observation = Observation {
                snapshot: snapshot_id.clone(),
                raw_key: key.clone(),
            };
            storage::observe(
                &mut tx,
                &metadata.instance.id,
                id,
                &metadata.instance.id,
                &snapshot.game_version,
                &observation,
            )
            .await?;
        }
        tx.commit().await?;
        tracing::info!(
            instance_id = metadata.instance.id,
            options = keys.len(),
            "Game setting locales: observations recorded"
        );
        candidates.push(Candidate {
            snapshot_id,
            snapshot,
            keys,
        });
    }
    let rows = storage::load(&state.pool, None).await?;
    tracing::info!(
        candidates = candidates.len(),
        rows = rows.len(),
        resolved = rows.iter().filter(|row| row.origin.is_some()).count(),
        "Game setting locales: resolving translation origins"
    );
    let mut pinned = 0;
    let mut unresolved = Vec::new();
    for row in rows {
        if row.origin.is_some() {
            continue;
        }
        let mut observations = Vec::new();
        if let Some(observation) = &row.observation {
            observations.push(observation.clone());
        } else {
            let mut matching: Vec<_> = candidates
                .iter()
                .filter(|c| {
                    (row.scope.is_empty()
                        || row.scope == c.snapshot.instance_id)
                        && c.keys.contains_key(&row.option_id)
                        && (row.backfilled
                            || row
                                .source_instance_id
                                .as_ref()
                                .is_none_or(|id| *id == c.snapshot.instance_id)
                                && row.source_game_version.as_ref().is_none_or(
                                    |v| *v == c.snapshot.game_version,
                                ))
                })
                .collect();
            matching.sort_by_key(|c| {
                (
                    row.source_instance_id.as_deref()
                        != Some(c.snapshot.instance_id.as_str()),
                    row.source_game_version.as_deref()
                        != Some(c.snapshot.game_version.as_str()),
                    c.snapshot.instance_id.clone(),
                )
            });
            observations.extend(matching.into_iter().map(|c| Observation {
                snapshot: c.snapshot_id.clone(),
                raw_key: c.keys[&row.option_id].clone(),
            }));
        }
        let mut resolved = false;
        let observation_count = observations.len();
        for observation in observations {
            match resolve_origin(
                state,
                &row.option_id,
                &observation,
                &mut archives,
            )
            .await
            {
                Ok(Some(origin)) => {
                    storage::pin(&state.pool, &row, &origin).await?;
                    resolved = true;
                    pinned += 1;
                    break;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(%error, scope = row.scope, option_id = row.option_id,
					snapshot_id = observation.snapshot, "Game setting locales: origin resolution failed")
                }
            }
        }
        if !resolved {
            tracing::debug!(
                scope = row.scope,
                option_id = row.option_id,
                observation_count,
                backfilled = row.backfilled,
                "Game setting locales: no matching translation origin"
            );
            unresolved.push((row.scope, row.option_id));
        }
    }
    tracing::info!(
        pinned,
        ?unresolved,
        "Game setting locales: origin resolution finished"
    );
    Ok(())
}

async fn resolve_origin(
    state: &State,
    option_id: &str,
    observation: &Observation,
    archives: &mut HashMap<String, Arc<ArchiveIndex>>,
) -> crate::Result<Option<Origin>> {
    let snapshot = sources::load_snapshot(state, &observation.snapshot).await?;
    if !option_id.starts_with("external:") {
        return Ok(None);
    }
    let Some(key) = mod_translation_key(&observation.raw_key) else {
        return Ok(None);
    };
    for source in &snapshot.mods {
        let index = match cached_archive(state, source, archives).await {
            Ok(index) => index,
            Err(error) => {
                tracing::warn!(%error, option_id, hash = source.hash, "Game setting locales: origin archive unavailable");
                continue;
            }
        };
        for bundle in &index.bundles {
            let Some(english) = bundle.locales.get("en_us") else {
                continue;
            };
            let mut english = english.clone();
            bundle.deprecated.apply(&mut english);
            if !english.get(key).is_some_and(|s| plain_label(s).is_some()) {
                continue;
            }
            return Ok(Some(Origin {
                instance_id: snapshot.instance_id.clone(),
                game_version: snapshot.game_version.clone(),
                legacy_game_jar_hash: None,
                archive: source.clone(),
                nested_path: bundle.nested_path.clone(),
                translation_key: key.to_owned(),
                choices: BTreeMap::new(),
            }));
        }
    }
    tracing::debug!(
        option_id,
        raw_key = observation.raw_key,
        key,
        "Game setting locales: translation keys not found in source archives"
    );
    Ok(None)
}

fn plain_label(value: &str) -> Option<String> {
    if value.trim().is_empty() || value.contains('%') || value.contains('§') {
        return None;
    }
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
#[tracing::instrument(skip_all, fields(instance_id, locale, requested = option_ids.len(), refresh_sources), err)]
pub async fn get_game_setting_locale_labels(
    instance_id: Option<&str>,
    locale: &str,
    option_ids: Vec<String>,
    refresh_sources: bool,
) -> crate::Result<GameSettingLocaleLabels> {
    tracing::info!("Game setting locales: label request received");
    if option_ids.len() > 16_384 {
        return Err(input_error("Too many requested game-setting labels"));
    }
    let requested: std::collections::HashSet<_> = option_ids
        .into_iter()
        .filter(|id| {
            id.strip_prefix("external:")
                .and_then(mod_translation_key)
                .is_some()
        })
        .collect();
    if requested.is_empty() {
        return Ok(GameSettingLocaleLabels::default());
    }
    let locale = minecraft_locale(locale);
    if !archive::valid_locale(&locale) {
        return Err(input_error("Invalid Minecraft locale"));
    }
    let state = State::get().await?;
    if refresh_sources {
        queue_game_locale_index();
    }
    let scope = instance_id.unwrap_or("");
    let mut rows = storage::load(&state.pool, Some(scope)).await?;
    tracing::info!(
        scope,
        rows = rows.len(),
        resolved = rows.iter().filter(|row| row.origin.is_some()).count(),
        "Game setting locales: scoped origins loaded"
    );
    if scope.is_empty() {
        let mut sourced: std::collections::HashSet<_> =
            rows.iter().map(|row| row.option_id.clone()).collect();
        // Unsaved global options have no provenance row; use an indexed local label until saved.
        for row in storage::load(&state.pool, None).await? {
            if !row.scope.is_empty()
                && requested.contains(&row.option_id)
                && row.origin.is_some()
                && sourced.insert(row.option_id.clone())
            {
                rows.push(row);
            }
        }
    }
    tracing::info!(
        rows = rows.len(),
        resolved = rows.iter().filter(|row| row.origin.is_some()).count(),
        "Game setting locales: origins selected including fallbacks"
    );
    let hashes: std::collections::HashSet<_> = rows
        .iter()
        .filter(|row| requested.contains(&row.option_id))
        .filter_map(|row| row.origin.as_ref())
        .filter(|origin| {
            origin.legacy_game_jar_hash.as_deref()
                != Some(origin.archive.hash.as_str())
        })
        .map(|origin| origin.archive.hash.as_str())
        .collect();
    let projects = sources::mod_projects(&state, &hashes.into_iter().collect::<Vec<_>>()).await
		.unwrap_or_else(|error| {
			tracing::warn!(%error, "Game setting locales: mod project metadata unavailable");
			HashMap::new()
		});
    let mut result = GameSettingLocaleLabels::default();
    let mut archives = HashMap::new();
    let mut dictionaries: HashMap<String, Translations> = HashMap::new();
    let mut missing_origins = Vec::new();
    for row in rows {
        if !requested.contains(&row.option_id) {
            continue;
        }
        let Some(origin) = row.origin else {
            missing_origins.push(row.option_id);
            continue;
        };
        if origin.legacy_game_jar_hash.as_deref()
            == Some(origin.archive.hash.as_str())
        {
            continue;
        }
        let dictionary_id =
            format!("{}:{}", origin.archive.hash, origin.nested_path);
        if !dictionaries.contains_key(&dictionary_id) {
            let index = match cached_archive(
                &state,
                &origin.archive,
                &mut archives,
            )
            .await
            {
                Ok(index) => index,
                Err(error) => {
                    tracing::warn!(%error, option_id = row.option_id, hash = origin.archive.hash,
						"Game setting locales: label archive unavailable");
                    continue;
                }
            };
            let Some(bundle) = index
                .bundles
                .iter()
                .find(|b| b.nested_path == origin.nested_path)
            else {
                tracing::warn!(
                    option_id = row.option_id,
                    nested_path = origin.nested_path,
                    "Game setting locales: source bundle missing"
                );
                continue;
            };
            let mut translations =
                bundle.locales.get("en_us").cloned().unwrap_or_default();
            if let Some(selected) = bundle.locales.get(&locale) {
                translations.extend(selected.clone());
            }
            bundle.deprecated.apply(&mut translations);
            tracing::info!(%dictionary_id, %locale, translations = translations.len(),
				"Game setting locales: dictionary loaded");
            dictionaries.insert(dictionary_id.clone(), translations);
        }
        let translations = &dictionaries[&dictionary_id];
        let label = translations
            .get(&origin.translation_key)
            .and_then(|s| plain_label(s));
        if let Some(label) = label {
            let choices = origin
                .choices
                .iter()
                .filter_map(|(value, key)| {
                    translations
                        .get(key)
                        .and_then(|s| plain_label(s))
                        .map(|label| (value.clone(), label))
                })
                .collect();
            let source = origin
                .archive
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|file_name| GameSettingSource {
                    instance_id: origin.instance_id.clone(),
                    file_name: file_name.to_owned(),
                    file_path: format!("mods/{file_name}"),
                    project: projects.get(&origin.archive.hash).map(
                        |project| GameSettingSourceProject {
                            id: project.id.clone(),
                            title: project.title.clone(),
                            icon_url: project.icon_url.clone(),
                        },
                    ),
                });
            result.settings.insert(
                row.option_id,
                GameSettingLocaleLabel {
                    label,
                    choices,
                    source,
                },
            );
        } else {
            tracing::warn!(
                option_id = row.option_id,
                key = origin.translation_key,
                key_present =
                    translations.contains_key(&origin.translation_key),
                "Game setting locales: translation missing or rejected as a label"
            );
        }
    }
    let mut missing: Vec<_> = requested
        .iter()
        .filter(|id| !result.settings.contains_key(*id))
        .collect();
    missing.sort();
    tracing::info!(
        returned = result.settings.len(),
        ?missing,
        ?missing_origins,
        "Game setting locales: label request completed"
    );
    Ok(result)
}
