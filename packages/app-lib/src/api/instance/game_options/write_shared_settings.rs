//! Writes shared settings into an instance without replacing the rest of `options.txt`.

use super::super::synced_options::{self, CheckpointStatus};
use super::CATALOG_REVISION;
use super::api_types::{
    GameOptionMappingKind, SaveGameSettingsResult, SyncOutcome, SyncReason,
};
use super::options_file::{
    GameOptionsDocument, options_path, read_document, sha1_bytes,
};
use super::read_instance_changes::{
    discover_custom_settings, previous_applied_settings,
    read_instance_changes_into_shared_settings,
};
use super::supported_settings::*;
use crate::state::{
    CanonicalValue, GameOptionsProjection, InstanceMetadata, ProjectedField,
    ProjectionOrigin, State, SyncedOption, game_options_sync_is_enabled,
    load_game_option_preferences, load_game_options_sync_state,
    load_shared_game_options,
};
use crate::util::io;
use std::collections::HashSet;

pub(super) async fn sync_is_active_for_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    Ok(metadata.synced_options.game_options
        && game_options_sync_is_enabled(&state.pool).await?)
}

/// Updates the enabled settings in memory and records the exact keys and values
/// Modrinth wrote, so changes made later in Minecraft can be recognized.
pub(super) async fn apply_shared_settings_to_document(
    metadata: &InstanceMetadata,
    document: &mut GameOptionsDocument,
    state: &State,
) -> crate::Result<Vec<ProjectedField>> {
    let values = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let settings = crate::state::Settings::get(&state.pool).await?;
    let mut fields = Vec::new();
    for (option_id, preference) in preferences {
        if !preference.enabled {
            continue;
        }
        if option_id == "fullscreen"
            && (metadata.launch_overrides.force_fullscreen.is_some()
                || settings.force_fullscreen)
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
        let Some(key) = target_physical_key(
            definition,
            stored,
            document,
            &metadata.applied_content_set.game_version,
        ) else {
            continue;
        };
        let migrated = definition.is_some_and(|definition| {
            alias_migration_needed(definition, document, &key)
                || document.value(&key).is_some_and(|raw| {
                    !physical_representation_supported_for_target(
                        definition,
                        &key,
                        raw,
                        &metadata.applied_content_set.game_version,
                    )
                })
        });
        let raw_value = if let Some(definition) = definition {
            let Some(raw_value) = encode_value(
                definition,
                &key,
                value,
                &metadata.applied_content_set.game_version,
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
        document.set(&key, &raw_value, definition.is_none() || migrated)?;
        fields.push(ProjectedField {
            option_id,
            physical_key: key,
            raw_value,
            origin: ProjectionOrigin::Shared,
            migrated,
        });
    }
    Ok(fields)
}

/// Reads local changes, applies shared values, and stores the keys and values it
/// wrote so the next sync can tell what changed.
pub(super) async fn sync_instance_options(
    metadata: &InstanceMetadata,
    state: &State,
    reason: SyncReason,
    allow_publish: bool,
) -> crate::Result<SyncOutcome> {
    if synced_options::instance_is_running(metadata, state).await? {
        return Ok(SyncOutcome::Deferred);
    }
    if reason != SyncReason::PackExtracted
        && reason != SyncReason::BeforePackUpdate
        && synced_options::sync_files_are_protected(metadata)
    {
        return Ok(SyncOutcome::Deferred);
    }
    let path = options_path(metadata, state);
    if !path.exists() {
        return Ok(SyncOutcome::WaitingForFile);
    }
    let (mut document, input_bytes) = read_document(&path).await?;
    let input_sha1 = sha1_bytes(&input_bytes);
    let checkpoint = synced_options::checkpoint(
        &metadata.instance.id,
        SyncedOption::GameOptions,
        "default",
        state,
    )
    .await?;
    let prior_projection = checkpoint.as_ref().and_then(|checkpoint| {
        previous_applied_settings(checkpoint.merge_base.as_deref())
    });
    let should_publish = allow_publish
        && reason != SyncReason::PackExtracted
        && checkpoint.as_ref().is_none_or(|checkpoint| {
            match checkpoint.status {
                CheckpointStatus::Ready => {
                    checkpoint.expected_sha1 != input_sha1
                }
                CheckpointStatus::Pending => {
                    checkpoint.expected_sha1 != input_sha1
                        && prior_projection.as_ref().is_none_or(|projection| {
                            projection.input_sha1 != input_sha1
                        })
                }
            }
        });
    let canonical_changed = if should_publish {
        read_instance_changes_into_shared_settings(
            metadata,
            &document,
            prior_projection.as_ref(),
            state,
        )
        .await?
    } else if allow_publish && reason != SyncReason::PackExtracted {
        let launcher_keys = prior_projection
            .as_ref()
            .into_iter()
            .flat_map(|projection| projection.fields.iter())
            .filter(|field| field.origin == ProjectionOrigin::LauncherOverride)
            .map(|field| field.physical_key.clone())
            .collect::<HashSet<_>>();
        discover_custom_settings(metadata, &document, &launcher_keys, state)
            .await?
    } else {
        false
    };
    if reason == SyncReason::BeforePackUpdate {
        if canonical_changed {
            let _ = Box::pin(sync_other_participating_instances(
                state,
                Some(&metadata.instance.id),
            ))
            .await;
        }
        return Ok(SyncOutcome::Unchanged);
    }

    let fields =
        apply_shared_settings_to_document(metadata, &mut document, state)
            .await?;
    let used_migration = fields.iter().any(|field| {
        field.migrated
            || setting_by_id(&field.option_id).is_some_and(|definition| {
                (matches!(definition.encoding, ValueEncoding::KeyBinding)
                    && split_key_binding(&field.raw_value)
                        .0
                        .parse::<i32>()
                        .is_ok())
                    || physical_variant_for_present_key(
                        definition,
                        &field.physical_key,
                        &metadata.applied_content_set.game_version,
                    )
                    .map(|variant| {
                        variant.mapping != GameOptionMappingKind::Direct
                    })
                    .unwrap_or_else(|| {
                        definition.keys.first().copied()
                            != Some(field.physical_key.as_str())
                    })
            })
    });
    let output_bytes = document.serialize()?;
    let output_sha1 = sha1_bytes(&output_bytes);
    let (canonical_revision, _) =
        load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
    let projection = GameOptionsProjection {
        schema_version: 1,
        target_game_version: metadata.applied_content_set.game_version.clone(),
        input_sha1,
        canonical_revision,
        fields,
    };
    let projection_bytes = serde_json::to_vec(&projection)?;
    synced_options::begin_checkpoint(
        &metadata.instance.id,
        SyncedOption::GameOptions,
        "default",
        &output_sha1,
        Some(&projection_bytes),
        canonical_revision as i64,
        state,
    )
    .await?;
    let changed = output_bytes != input_bytes;
    if changed {
        io::write(&path, output_bytes).await?;
    }
    synced_options::finish_plain_checkpoint(
        &metadata.instance.id,
        SyncedOption::GameOptions,
        "default",
        state,
    )
    .await?;
    if canonical_changed {
        let _ = Box::pin(sync_other_participating_instances(
            state,
            Some(&metadata.instance.id),
        ))
        .await;
    }
    Ok(if changed && used_migration {
        SyncOutcome::Migrated
    } else if changed {
        SyncOutcome::Applied
    } else {
        SyncOutcome::Unchanged
    })
}

pub(in crate::api::instance) async fn sync_instance_with_state(
    metadata: &InstanceMetadata,
    state: &State,
    reason: SyncReason,
) -> crate::Result<()> {
    sync_instance_options(metadata, state, reason, true).await?;
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
    for metadata in instances {
        if excluded_instance_id == Some(metadata.instance.id.as_str())
            || !metadata.synced_options.game_options
        {
            continue;
        }
        match sync_instance_options(&metadata, state, SyncReason::Normal, true)
            .await
        {
            Ok(SyncOutcome::Applied | SyncOutcome::Unchanged) => {
                result.applied += 1
            }
            Ok(SyncOutcome::Migrated) => result.migrated += 1,
            Ok(SyncOutcome::Deferred | SyncOutcome::WaitingForFile) => {
                result.deferred += 1
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to project synced game options into {}: {error}",
                    metadata.instance.id
                );
                result.failed += 1;
            }
        }
    }
    result
}

pub(super) async fn sync_all_participating_instances(
    state: &State,
) -> SaveGameSettingsResult {
    sync_other_participating_instances(state, None).await
}
