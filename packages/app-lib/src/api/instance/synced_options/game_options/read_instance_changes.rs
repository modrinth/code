//! Imports saved `options.txt` settings into the canonical values.

use super::CATALOG_REVISION;
use super::api_types::canonical_values_equal;
use super::catalog::*;
use super::fullscreen::update_app_fullscreen_setting;
use super::launch_overrides::currently_launcher_owned_keys;
use super::options_file::{GameOptionsDocument, validate_raw_key_value};
use crate::state::{
    CanonicalValue, GameOptionKind, InstanceMetadata, State,
    load_game_option_preferences, load_game_options_sync_state,
    load_shared_game_options,
};
use chrono::Utc;
use std::collections::HashSet;

pub(super) fn custom_setting_id(key: &str) -> String {
    format!("external:{key}")
}

/// Imports every enabled setting present in `options.txt`. The caller skips file
/// hashes produced by app writes.
pub(super) async fn read_instance_changes_into_shared_settings(
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    state: &State,
) -> crate::Result<bool> {
    let values = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let launcher_keys = currently_launcher_owned_keys(metadata);
    let now = Utc::now().timestamp();
    let mut updates = Vec::new();
    for (option_id, stored) in &values {
        if !preferences
            .get(option_id)
            .is_some_and(|preference| preference.enabled)
        {
            continue;
        }
        if matches!(stored.kind, GameOptionKind::External)
            && stored.raw_key.is_none()
        {
            continue;
        }
        if stored
            .raw_key
            .as_deref()
            .is_some_and(|key| setting_by_file_key(key).is_some())
        {
            continue;
        }
        let definition = setting_by_id(option_id);
        if definition.is_some()
            && !supported_settings_cover_game_version(
                &metadata.applied_content_set.game_version,
            )
        {
            continue;
        }
        let current_key = if let Some(definition) = definition {
            observed_physical_key(
                definition,
                document,
                &metadata.applied_content_set.game_version,
            )
        } else {
            stored.raw_key.clone()
        };
        let Some(current_key) = current_key else {
            continue;
        };
        if launcher_keys.contains(&current_key) {
            continue;
        }
        let Some(current_raw) = document.value(&current_key) else {
            continue;
        };
        let candidate = if let Some(definition) = definition {
            decode_value_for_version(
                definition,
                &current_key,
                current_raw,
                &metadata.applied_content_set.game_version,
            )
        } else if matches!(stored.kind, GameOptionKind::External) {
            if validate_raw_key_value(&current_key, current_raw).is_err() {
                continue;
            }
            Some(CanonicalValue::ExternalRaw(current_raw.to_string()))
        } else {
            None
        };
        let Some(candidate) = candidate else {
            continue;
        };
        if stored
            .value
            .as_ref()
            .is_some_and(|current| canonical_values_equal(current, &candidate))
        {
            continue;
        }
        updates.push((option_id.to_string(), candidate, stored.revision));
    }

    let mut changed = false;
    let mut fullscreen_value = None;
    if !updates.is_empty() {
        let (canonical_revision, _) =
            load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
        let mut tx = state.pool.begin().await?;
        for (option_id, candidate, revision) in updates {
            let next_revision = revision.saturating_add(1) as i64;
            let current_revision = revision as i64;
            let value_json = serde_json::to_string(&candidate)?;
            let canonical_type = candidate.type_name();
            let result = sqlx::query!(
                "
				UPDATE synced_game_option_values
				SET canonical_type = ?, canonical_value_json = ?, seeded = 1,
					revision = ?, origin = 'instance', source_game_version = ?,
					source_instance_id = ?, updated_at = ?
				WHERE option_id = ? AND revision = ?
				",
                canonical_type,
                value_json,
                next_revision,
                metadata.applied_content_set.game_version,
                metadata.instance.id,
                now,
                option_id,
                current_revision,
            )
            .execute(&mut *tx)
            .await?;
            if result.rows_affected() == 0 {
                continue;
            }
            if option_id == "fullscreen" {
                fullscreen_value = Some(candidate.clone());
            }
            changed = true;
        }
        if changed {
            let next_revision = canonical_revision.saturating_add(1) as i64;
            sqlx::query!(
                "
				UPDATE synced_game_option_state
				SET revision = ?, catalog_revision = ?
				WHERE singleton = 1
				",
                next_revision,
                CATALOG_REVISION as i64,
            )
            .execute(&mut *tx)
            .await?;
        }
        if let Some(value) = fullscreen_value.as_ref() {
            update_app_fullscreen_setting(&mut tx, value, true).await?;
        }
        tx.commit().await?;
    }
    let discovered =
        discover_custom_settings(metadata, document, &launcher_keys, state)
            .await?;
    Ok(discovered || changed)
}

/// Finds settings added by mods and adds them to the shared settings list.
pub(super) async fn discover_custom_settings(
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    excluded_keys: &HashSet<String>,
    state: &State,
) -> crate::Result<bool> {
    let existing = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let (canonical_revision, _) =
        load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
    let entries = document.effective_entries();
    let now = Utc::now().timestamp();
    let mut discovered = false;
    let mut tx = state.pool.begin().await?;
    let game_version = metadata.applied_content_set.game_version.as_str();

    for definition in all_supported_settings() {
        for key in definition.keys {
            let custom_id = custom_setting_id(key);
            let Some(custom) = existing.get(&custom_id) else {
                continue;
            };
            if !custom.seeded {
                continue;
            }
            let revision = custom.revision.saturating_add(1) as i64;
            sqlx::query!(
                "
				UPDATE synced_game_option_values
				SET seeded = 0, revision = ?, updated_at = ?
				WHERE option_id = ?
				",
                revision,
                now,
                custom_id,
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!(
                "
				UPDATE synced_game_option_preferences
				SET enabled = 0, source = 'catalog_default', revision = ?
				WHERE option_id = ?
				",
                revision,
                custom_id,
            )
            .execute(&mut *tx)
            .await?;
            discovered = true;
        }
    }

    if supported_settings_cover_game_version(game_version) {
        for definition in all_supported_settings() {
            if existing.contains_key(definition.id) {
                continue;
            }
            let discovered_value = definition.keys.iter().find_map(|key| {
                if excluded_keys.contains(*key) {
                    return None;
                }
                let raw = document.value(key)?;
                decode_value_for_version(definition, key, raw, game_version)
            });
            let Some(value) = discovered_value else {
                continue;
            };
            let value_json = serde_json::to_string(&value)?;
            let canonical_type = value.type_name();
            sqlx::query!(
                "
				INSERT INTO synced_game_option_values
					(option_id, kind, raw_key, canonical_type,
					 canonical_value_json, value_codec, seeded, revision, origin,
					 source_game_version, source_instance_id, updated_at)
				VALUES (?, 'vanilla', NULL, ?, ?, 'catalog', 1, 1,
					'instance', ?, ?, ?)
				",
                definition.id,
                canonical_type,
                value_json,
                game_version,
                metadata.instance.id,
                now,
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!(
                "
				INSERT INTO synced_game_option_preferences
					(option_id, enabled, source, revision)
				VALUES (?, ?, 'catalog_default', 1)
				",
                definition.id,
                definition.default_on,
            )
            .execute(&mut *tx)
            .await?;
            discovered = true;
        }
    }

    for (key, (index, _)) in entries {
        if excluded_keys.contains(key)
            || setting_by_file_key(key).is_some()
            || is_never_sync_key(key)
        {
            continue;
        }
        let Some(entry) = &document.lines[index].entry else {
            continue;
        };
        if validate_raw_key_value(key, &entry.value).is_err() {
            continue;
        }
        let option_id = custom_setting_id(key);
        let should_seed =
            existing.get(&option_id).is_none_or(|stored| !stored.seeded);
        if !should_seed {
            continue;
        }
        let revision = existing
            .get(&option_id)
            .map(|stored| stored.revision)
            .unwrap_or(0)
            .max(
                preferences
                    .get(&option_id)
                    .map(|preference| preference.revision)
                    .unwrap_or(0),
            )
            .saturating_add(1) as i64;
        let value = CanonicalValue::ExternalRaw(entry.value.clone());
        let value_json = serde_json::to_string(&value)?;
        sqlx::query!(
            "
			INSERT INTO synced_game_option_values
				(option_id, kind, raw_key, canonical_type,
				 canonical_value_json, value_codec, seeded, revision, origin,
				 source_game_version, source_instance_id, updated_at)
			VALUES (?, 'external', ?, 'external_raw', ?, 'exact_raw',
				1, ?, 'instance', ?, ?, ?)
			ON CONFLICT(option_id) DO UPDATE SET
				raw_key = excluded.raw_key,
				canonical_value_json = excluded.canonical_value_json,
				seeded = 1, revision = excluded.revision,
				origin = 'instance',
				source_game_version = excluded.source_game_version,
				source_instance_id = excluded.source_instance_id,
				updated_at = excluded.updated_at
			",
            option_id,
            key,
            value_json,
            revision,
            game_version,
            metadata.instance.id,
            now,
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "
			INSERT INTO synced_game_option_preferences
				(option_id, enabled, source, revision)
			VALUES (?, 1, 'discovery_default', ?)
			ON CONFLICT(option_id) DO NOTHING
			",
            option_id,
            revision,
        )
        .execute(&mut *tx)
        .await?;
        discovered = true;
    }

    if discovered {
        let next_revision = canonical_revision.saturating_add(1) as i64;
        sqlx::query!(
            "
			UPDATE synced_game_option_state
			SET revision = ?, catalog_revision = ?
			WHERE singleton = 1
			",
            next_revision,
            CATALOG_REVISION as i64,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(discovered)
}
