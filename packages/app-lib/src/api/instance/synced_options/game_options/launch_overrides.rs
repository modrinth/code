//! Updates shared settings, then applies launcher settings before Minecraft starts.

use super::super as synced_options;
use super::CATALOG_REVISION;
use super::api_types::SyncReason;
use super::options_file::{
    GameOptionsDocument, input_error, options_path, read_document, sha1_bytes,
    validate_raw_key_value,
};
use super::read_instance_changes::previous_applied_settings;
use super::catalog::setting_by_file_key;
use super::write_shared_settings::{
    sync_instance_options, sync_is_active_for_instance,
};
use crate::state::{
    GameOptionsProjection, InstanceMetadata, ProjectedField, ProjectionOrigin,
    State, SyncedOption, load_game_options_sync_state,
};
use crate::util::io;
use std::collections::HashSet;

pub(super) fn currently_launcher_owned_keys(
    metadata: &InstanceMetadata,
) -> HashSet<String> {
    let mut keys = HashSet::new();
    if metadata.launch_overrides.force_fullscreen.is_some() {
        keys.insert("fullscreen".to_string());
    }
    keys
}

/// Reads changes made in Minecraft and writes the latest shared values before launch.
pub async fn sync_before_launch(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| input_error("Unknown instance"))?;
    if sync_is_active_for_instance(&metadata, &state).await? {
        let _ = sync_instance_options(
            &metadata,
            &state,
            SyncReason::BeforeLaunch,
            true,
        )
        .await?;
    }
    Ok(())
}

/// Applies launcher settings, such as forced fullscreen, after shared settings.
///
/// We remember these separately so the next sync does not mistake them for changes
/// made by the player.
pub async fn apply_launcher_overrides(
    instance_id: &str,
    overrides: &[(String, String)],
) -> crate::Result<()> {
    if overrides.is_empty() {
        return Ok(());
    }
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| input_error("Unknown instance"))?;
    let path = options_path(&metadata, &state);
    let (mut document, input_bytes) = if path.exists() {
        read_document(&path).await?
    } else {
        (GameOptionsDocument::empty(), Vec::new())
    };
    let input_sha1 = sha1_bytes(&input_bytes);
    let checkpoint = synced_options::checkpoint(
        instance_id,
        SyncedOption::GameOptions,
        "default",
        &state,
    )
    .await?;
    let mut projection = checkpoint
        .as_ref()
        .filter(|checkpoint| checkpoint.expected_sha1 == input_sha1)
        .and_then(|checkpoint| {
            previous_applied_settings(checkpoint.merge_base.as_deref())
        })
        .unwrap_or(GameOptionsProjection {
            schema_version: 1,
            target_game_version: metadata
                .applied_content_set
                .game_version
                .clone(),
            input_sha1: input_sha1.clone(),
            canonical_revision: load_game_options_sync_state(
                &state.pool,
                CATALOG_REVISION,
            )
            .await?
            .0,
            fields: Vec::new(),
        });

    for (key, value) in overrides {
        validate_raw_key_value(key, value)?;
        document.set(key, value, true)?;
        projection.fields.retain(|field| field.physical_key != *key);
        let option_id = setting_by_file_key(key)
            .map(|definition| definition.id.to_string())
            .unwrap_or_else(|| format!("launcher:{key}"));
        projection.fields.push(ProjectedField {
            option_id,
            physical_key: key.clone(),
            raw_value: value.clone(),
            origin: ProjectionOrigin::LauncherOverride,
            migrated: false,
        });
    }
    let output_bytes = document.serialize()?;
    let output_sha1 = sha1_bytes(&output_bytes);
    projection.input_sha1 = input_sha1;
    let projection_bytes = serde_json::to_vec(&projection)?;
    synced_options::begin_checkpoint(
        instance_id,
        SyncedOption::GameOptions,
        "default",
        &output_sha1,
        Some(&projection_bytes),
        projection.canonical_revision as i64,
        &state,
    )
    .await?;
    io::write(&path, output_bytes).await?;
    synced_options::finish_plain_checkpoint(
        instance_id,
        SyncedOption::GameOptions,
        "default",
        &state,
    )
    .await?;
    Ok(())
}
