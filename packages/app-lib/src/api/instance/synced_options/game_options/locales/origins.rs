use super::sources::{self, ArchiveSource, Snapshot};
use super::{Observation, Origin, is_plain_label, mod_translation_key};
use crate::state::State;
use crate::util::io;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

type ArchiveKeys = BTreeMap<String, Arc<str>>;

struct SnapshotIndex {
    snapshot: Snapshot,
    keys: BTreeMap<String, (usize, Arc<str>)>,
}

#[derive(Default)]
pub(super) struct OriginResolver {
    archives: HashMap<String, Arc<ArchiveKeys>>,
    snapshots: HashMap<String, SnapshotIndex>,
}

impl OriginResolver {
    pub(super) async fn index_snapshot(
        &mut self,
        state: &State,
        id: &str,
        snapshot: Snapshot,
    ) -> crate::Result<()> {
        if self.snapshots.contains_key(id) {
            return Ok(());
        }
        let mut archives = Vec::new();
        for (source_index, source) in snapshot.mods.iter().enumerate() {
            let archive = if let Some(keys) = self.archives.get(&source.hash) {
                Arc::clone(keys)
            } else {
                match load_archive_keys(state, source).await {
                    Ok(keys) => {
                        let keys = Arc::new(keys);
                        self.archives
                            .insert(source.hash.clone(), Arc::clone(&keys));
                        keys
                    }
                    Err(error) => {
                        tracing::warn!(hash = source.hash, %error,
							"Game setting locales: origin archive unavailable");
                        continue;
                    }
                }
            };
            archives.push((source_index, archive));
        }
        let index = tokio::task::spawn_blocking(move || {
            let mut keys = BTreeMap::new();
            for (source_index, archive) in archives {
                for (key, nested_path) in archive.iter() {
                    keys.entry(key.clone()).or_insert_with(|| {
                        (source_index, Arc::clone(nested_path))
                    });
                }
            }
            SnapshotIndex { snapshot, keys }
        })
        .await
        .map_err(|error| super::input_error(error.to_string()))?;
        self.snapshots.insert(id.to_owned(), index);
        Ok(())
    }

    pub(super) async fn resolve(
        &mut self,
        state: &State,
        observation: &Observation,
    ) -> crate::Result<Option<Origin>> {
        let Some(key) = mod_translation_key(&observation.raw_key) else {
            return Ok(None);
        };
        if !self.snapshots.contains_key(&observation.snapshot) {
            let snapshot =
                sources::load_snapshot(state, &observation.snapshot).await?;
            self.index_snapshot(state, &observation.snapshot, snapshot)
                .await?;
        }
        let index = &self.snapshots[&observation.snapshot];
        Ok(index
            .keys
            .get(key)
            .map(|(source_index, nested_path)| Origin {
                instance_id: index.snapshot.instance_id.clone(),
                game_version: index.snapshot.game_version.clone(),
                legacy_game_jar_hash: None,
                archive: index.snapshot.mods[*source_index].clone(),
                nested_path: nested_path.to_string(),
                translation_key: key.to_owned(),
                choices: BTreeMap::new(),
            }))
    }
}

/// A complete key index also caches misses until the archive hash changes.
async fn load_archive_keys(
    state: &State,
    source: &ArchiveSource,
) -> crate::Result<ArchiveKeys> {
    if !super::archive::valid_hash(&source.hash) {
        return Err(super::input_error("Invalid locale archive hash"));
    }
    let path = sources::root(state)
        .join("origin-keys-v1")
        .join(format!("{}.json", source.hash));
    if let Ok(bytes) = io::read(&path).await {
        let cached = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<ArchiveKeys>(&bytes)
        })
        .await
        .map_err(|error| super::input_error(error.to_string()))?;
        if let Ok(keys) = cached {
            return Ok(keys);
        }
    }
    let index = sources::archive_index(state, source).await?;
    let keys = tokio::task::spawn_blocking(move || {
        let mut keys = ArchiveKeys::new();
        for mut bundle in index.bundles {
            let Some(mut english) = bundle.locales.remove("en_us") else {
                continue;
            };
            bundle.deprecated.apply(&mut english);
            let nested_path: Arc<str> = bundle.nested_path.into();
            for (key, value) in english {
                if is_plain_label(&value) {
                    keys.entry(key).or_insert_with(|| Arc::clone(&nested_path));
                }
            }
        }
        keys
    })
    .await
    .map_err(|error| super::input_error(error.to_string()))?;
    if let Err(error) = sources::write_json(&path, &keys).await {
        tracing::warn!(hash = source.hash, %error,
			"Game setting locales: could not cache origin keys");
    }
    Ok(keys)
}
