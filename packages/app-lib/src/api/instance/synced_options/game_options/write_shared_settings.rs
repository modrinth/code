//! Imports `options.txt` saves and copies canonical values to synced instances.

use super::super as synced_options;
use super::CATALOG_REVISION;
use super::api_types::{
    GameOptionMappingKind, SaveGameSettingsResult, SyncOutcome,
};
use super::catalog::*;
use super::options_file::{
    GameOptionsDocument, options_path, read_document, sha1_bytes,
};
use super::pack_updates::materialize_pack_options_if_missing;
use super::read_instance_changes::{
    InstanceEdits, read_instance_changes_into_shared_settings,
};
use super::source_selection::initialize_from_source_instance;
use crate::state::{
    CanonicalValue, InstanceMetadata, State, SyncedOption,
    game_options_sync_is_enabled, load_game_option_preferences,
    load_game_options_sync_state, load_shared_game_options,
};
use crate::util::io;

struct AppliedSettings {
    count: usize,
    used_migration: bool,
}

pub(super) async fn sync_is_active_for_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    Ok(metadata.synced_options.game_options
        && !synced_options::pending::contains(
            &metadata.instance.id,
            SyncedOption::GameOptions,
            state,
        )
        .await?
        && game_options_sync_is_enabled(&state.pool).await?)
}

pub(super) async fn remember_synced_file(
    instance_id: &str,
    contents: &[u8],
    state: &State,
) -> crate::Result<()> {
    let (canonical_revision, _) =
        load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
    synced_options::begin_checkpoint(
        instance_id,
        SyncedOption::GameOptions,
        "default",
        &sha1_bytes(contents),
        Some(contents),
        canonical_revision as i64,
        state,
    )
    .await?;
    synced_options::finish_plain_checkpoint(
        instance_id,
        SyncedOption::GameOptions,
        "default",
        state,
    )
    .await
}

async fn apply_shared_settings_to_document(
    metadata: &InstanceMetadata,
    document: &mut GameOptionsDocument,
    state: &State,
) -> crate::Result<AppliedSettings> {
    let values = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let mut count = 0;
    let mut used_migration = false;

    for (option_id, preference) in preferences {
        if !preference.enabled {
            continue;
        }
        if option_id == "fullscreen"
            && metadata.launch_overrides.force_fullscreen.is_some()
        {
            continue;
        }
        let Some(stored) = values.get(&option_id) else {
            continue;
        };
        let Some(value) = stored.value.as_ref().filter(|_| stored.seeded)
        else {
            continue;
        };
        let definition = setting_by_id(&option_id);
        if definition.is_some()
            && !supported_settings_cover_game_version(
                &metadata.applied_content_set.game_version,
            )
        {
            continue;
        }
        if definition.is_none()
            && stored
                .raw_key
                .as_deref()
                .is_some_and(|key| setting_by_file_key(key).is_some())
        {
            continue;
        }

        let target_version = &metadata.applied_content_set.game_version;
        let key =
            target_physical_key(definition, stored, document, target_version)
                .or_else(|| {
                    definition.and_then(|definition| {
                        target_key_for_version(definition, target_version)
                            .map(str::to_string)
                    })
                })
                .or_else(|| stored.raw_key.clone());
        let Some(key) = key else {
            continue;
        };

        let migrated = definition.is_some_and(|definition| {
            alias_migration_needed(definition, document, &key)
                || document.value(&key).is_some_and(|raw| {
                    !physical_representation_supported_for_target(
                        definition,
                        &key,
                        raw,
                        target_version,
                    )
                })
        });
        let raw_value = if let Some(definition) = definition {
            let Some(raw_value) = encode_value(
                definition,
                &key,
                value,
                target_version,
                document.value(&key),
            ) else {
                continue;
            };
            raw_value
        } else if let CanonicalValue::ExternalRaw(raw_value) = value {
            raw_value.clone()
        } else {
            continue;
        };

        document.set(&key, &raw_value, true)?;
        count += 1;
        used_migration |= migrated
            || definition.is_some_and(|definition| {
                (matches!(definition.encoding, ValueEncoding::KeyBinding)
                    && split_key_binding(&raw_value).0.parse::<i32>().is_ok())
                    || physical_variant_for_present_key(
                        definition,
                        &key,
                        target_version,
                    )
                    .map(|variant| {
                        variant.mapping != GameOptionMappingKind::Direct
                    })
                    .unwrap_or_else(|| {
                        definition.keys.first().copied() != Some(key.as_str())
                    })
            });
    }

    Ok(AppliedSettings {
        count,
        used_migration,
    })
}

async fn record_unsynced_changes(
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    contents: &[u8],
    state: &State,
) -> crate::Result<bool> {
    let Some(checkpoint) = synced_options::checkpoint(
        &metadata.instance.id,
        SyncedOption::GameOptions,
        "default",
        state,
    )
    .await?
    else {
        return Ok(false);
    };
    if checkpoint.expected_sha1 == sha1_bytes(contents)
        || !crate::state::shared_game_options_exist(&state.pool).await?
    {
        return Ok(false);
    }
    let Some(last_synced) =
        last_synced_document(checkpoint.merge_base.as_deref())
    else {
        return Ok(false);
    };
    read_instance_changes_into_shared_settings(
        metadata,
        document,
        InstanceEdits::Offline(&last_synced),
        state,
    )
    .await
}

async fn import_unsynced_instance_changes(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    if synced_options::sync_files_are_protected(metadata)
        || !sync_is_active_for_instance(metadata, state).await?
    {
        return Ok(false);
    }
    let path = options_path(metadata, state);
    if !path.exists() {
        return Ok(false);
    }
    let (document, contents) = read_document(&path).await?;
    if document.value("version").is_none()
        && !supported_settings_cover_game_version(
            &metadata.applied_content_set.game_version,
        )
    {
        return Ok(false);
    }
    record_unsynced_changes(metadata, &document, &contents, state).await
}

pub(super) async fn apply_shared_settings_to_instance(
    metadata: &InstanceMetadata,
    state: &State,
    allow_protected: bool,
) -> crate::Result<SyncOutcome> {
    if !allow_protected && synced_options::sync_files_are_protected(metadata) {
        return Ok(SyncOutcome::Deferred);
    }
    if !sync_is_active_for_instance(metadata, state).await? {
        return Ok(SyncOutcome::Unchanged);
    }

    let path = options_path(metadata, state);
    if !path.exists() {
        materialize_pack_options_if_missing(metadata, state).await?;
    }
    let file_was_missing = !path.exists();
    let (mut document, input_bytes) = if path.exists() {
        read_document(&path).await?
    } else {
        let Some(document) =
            GameOptionsDocument::for_instance(metadata, state).await?
        else {
            return Ok(SyncOutcome::WaitingForFile);
        };
        (document, Vec::new())
    };
    if document.value("version").is_none()
        && !supported_settings_cover_game_version(
            &metadata.applied_content_set.game_version,
        )
    {
        return Ok(SyncOutcome::Deferred);
    }
    if !file_was_missing {
        record_unsynced_changes(metadata, &document, &input_bytes, state)
            .await?;
    }
    let applied =
        apply_shared_settings_to_document(metadata, &mut document, state)
            .await?;
    if file_was_missing && applied.count == 0 {
        return Ok(SyncOutcome::WaitingForFile);
    }

    let output_bytes = document.serialize()?;
    let changed = file_was_missing || output_bytes != input_bytes;
    if changed {
        io::write(&path, &output_bytes).await?;
    }
    remember_synced_file(&metadata.instance.id, &output_bytes, state).await?;

    Ok(if changed && applied.used_migration {
        SyncOutcome::Migrated
    } else if changed {
        SyncOutcome::Applied
    } else {
        SyncOutcome::Unchanged
    })
}

pub(super) async fn capture_instance_options(
    metadata: &InstanceMetadata,
    state: &State,
    allow_protected: bool,
) -> crate::Result<SyncOutcome> {
    if !allow_protected && synced_options::sync_files_are_protected(metadata) {
        return Ok(SyncOutcome::Deferred);
    }
    if !sync_is_active_for_instance(metadata, state).await? {
        return Ok(SyncOutcome::Unchanged);
    }

    let path = options_path(metadata, state);
    if !path.exists() {
        return Ok(SyncOutcome::WaitingForFile);
    }
    let (document, input_bytes) = read_document(&path).await?;
    let input_sha1 = sha1_bytes(&input_bytes);
    let checkpoint = synced_options::checkpoint(
        &metadata.instance.id,
        SyncedOption::GameOptions,
        "default",
        state,
    )
    .await?;
    if checkpoint
        .as_ref()
        .is_some_and(|checkpoint| checkpoint.expected_sha1 == input_sha1)
    {
        return Ok(SyncOutcome::Unchanged);
    }

    let canonical_changed =
        if crate::state::shared_game_options_exist(&state.pool).await? {
            read_instance_changes_into_shared_settings(
                metadata,
                &document,
                InstanceEdits::Live(
                    last_synced_document(checkpoint.as_ref().and_then(
                        |checkpoint| checkpoint.merge_base.as_deref(),
                    ))
                    .as_ref(),
                ),
                state,
            )
            .await?
        } else {
            initialize_from_source_instance(metadata, state).await?;
            true
        };
    remember_synced_file(&metadata.instance.id, &input_bytes, state).await?;

    if canonical_changed {
        let _ = sync_other_participating_instances(
            state,
            Some(&metadata.instance.id),
        )
        .await;
    }
    Ok(SyncOutcome::Unchanged)
}

fn last_synced_document(
    contents: Option<&[u8]>,
) -> Option<GameOptionsDocument> {
    contents.and_then(|contents| GameOptionsDocument::parse(contents).ok())
}

pub(in crate::api::instance) async fn capture_instance_file_change(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    capture_instance_options(metadata, state, false).await?;
    Ok(())
}

pub(in crate::api::instance) async fn apply_instance_with_state(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    apply_shared_settings_to_instance(metadata, state, false).await?;
    Ok(())
}

pub(super) async fn sync_other_participating_instances(
    state: &State,
    excluded_instance_id: Option<&str>,
) -> SaveGameSettingsResult {
    let mut result = SaveGameSettingsResult {
        state: None,
        applied: 0,
        migrated: 0,
        deferred: 0,
        unsupported: 0,
        failed: 0,
        conflicts: Vec::new(),
    };
    match game_options_sync_is_enabled(&state.pool).await {
        Ok(true) => {}
        Ok(false) => return result,
        Err(error) => {
            tracing::warn!(
                "Failed to inspect global game-options state before fan-out: {error}"
            );
            result.failed += 1;
            return result;
        }
    }
    let instances = match crate::state::list_instances(&state.pool).await {
        Ok(instances) => instances,
        Err(error) => {
            tracing::warn!(
                "Failed to list instances for game-options fan-out: {error}"
            );
            result.failed += 1;
            return result;
        }
    };
    let instances = instances
        .into_iter()
        .filter(|metadata| metadata.synced_options.game_options)
        .collect::<Vec<_>>();
    let mut imported = false;
    for metadata in &instances {
        if excluded_instance_id == Some(metadata.instance.id.as_str()) {
            continue;
        }
        match import_unsynced_instance_changes(metadata, state).await {
            Ok(changed) => imported |= changed,
            Err(error) => tracing::warn!(
                "Failed to read unsynced game options from {}: {error}",
                metadata.instance.id
            ),
        }
    }
    for metadata in &instances {
        if !imported
            && excluded_instance_id == Some(metadata.instance.id.as_str())
        {
            continue;
        }
        match apply_shared_settings_to_instance(metadata, state, false).await {
            Ok(SyncOutcome::Applied | SyncOutcome::Unchanged) => {
                result.applied += 1
            }
            Ok(SyncOutcome::Migrated) => result.migrated += 1,
            Ok(SyncOutcome::Deferred | SyncOutcome::WaitingForFile) => {
                result.deferred += 1
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to write synced game options into {}: {error}",
                    metadata.instance.id
                );
                result.failed += 1;
            }
        }
    }
    result
}

pub(crate) async fn sync_all_participating_instances(
    state: &State,
) -> SaveGameSettingsResult {
    sync_other_participating_instances(state, None).await
}
