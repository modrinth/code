//! Finds settings changed in Minecraft and shares those changes with other instances.

use super::super as synced_options;
use super::CATALOG_REVISION;
use super::api_types::canonical_values_equal;
use super::catalog::*;
use super::fullscreen::update_app_fullscreen_setting;
use super::launch_overrides::currently_launcher_owned_keys;
use super::options_file::{GameOptionsDocument, validate_raw_key_value};
use crate::state::{
    CanonicalValue, GameOptionKind, GameOptionsProjection, InstanceMetadata,
    ProjectionOrigin, State, SyncedOption, load_game_option_preferences,
    load_game_options_sync_state, load_shared_game_options,
};
use chrono::Utc;
use std::collections::HashSet;

pub(super) fn custom_setting_id(key: &str) -> String {
    format!("external:{key}")
}

pub(super) fn previous_applied_settings(
    bytes: Option<&[u8]>,
) -> Option<GameOptionsProjection> {
    serde_json::from_slice(bytes?).ok()
}

pub(super) async fn launcher_keys_for_document(
    instance_id: &str,
    input_sha1: &str,
    state: &State,
) -> crate::Result<HashSet<String>> {
    Ok(synced_options::checkpoint(
        instance_id,
        SyncedOption::GameOptions,
        "default",
        state,
    )
    .await?
    .filter(|checkpoint| checkpoint.expected_sha1 == input_sha1)
    .and_then(|checkpoint| {
        previous_applied_settings(checkpoint.merge_base.as_deref())
    })
    .into_iter()
    .flat_map(|projection| projection.fields.into_iter())
    .filter(|field| field.origin == ProjectionOrigin::LauncherOverride)
    .map(|field| field.physical_key)
    .collect())
}

/// Compares `options.txt` with what Modrinth last wrote. A valid change becomes
/// the new shared value unless the launcher made it.
pub(super) async fn read_instance_changes_into_shared_settings(
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    projection: Option<&GameOptionsProjection>,
    state: &State,
) -> crate::Result<bool> {
    let values = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let currently_launcher_owned = currently_launcher_owned_keys(metadata);
    let Some(projection) = projection else {
        return discover_custom_settings(
            metadata,
            document,
            &currently_launcher_owned,
            state,
        )
        .await;
    };
    let mut launcher_keys = projection
        .fields
        .iter()
        .filter(|field| field.origin == ProjectionOrigin::LauncherOverride)
        .map(|field| field.physical_key.clone())
        .collect::<HashSet<_>>();
    if !currently_launcher_owned.contains("fullscreen") {
        launcher_keys.remove("fullscreen");
    }
    launcher_keys.extend(currently_launcher_owned);
    let now = Utc::now().timestamp();
    let mut updates = Vec::new();

    for field in &projection.fields {
        if field.origin == ProjectionOrigin::LauncherOverride
            || launcher_keys.contains(&field.physical_key)
            || !preferences
                .get(&field.option_id)
                .is_some_and(|preference| preference.enabled)
        {
            continue;
        }
        let Some(stored) = values.get(&field.option_id) else {
            continue;
        };
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
        let definition = setting_by_id(&field.option_id);
        if definition.is_some()
            && !supported_settings_cover_game_version(
                &metadata.applied_content_set.game_version,
            )
        {
            continue;
        }
        let current_key = if document.value(&field.physical_key).is_some() {
            Some(field.physical_key.clone())
        } else if let Some(definition) = definition {
            observed_physical_key(
                definition,
                document,
                &metadata.applied_content_set.game_version,
            )
        } else {
            target_physical_key(
                None,
                stored,
                document,
                &metadata.applied_content_set.game_version,
            )
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
        if current_key == field.physical_key && current_raw == field.raw_value {
            continue;
        }
        let projected_candidate = if let Some(definition) = definition {
            decode_value(definition, &field.physical_key, &field.raw_value)
        } else if matches!(stored.kind, GameOptionKind::External) {
            Some(CanonicalValue::ExternalRaw(field.raw_value.clone()))
        } else {
            None
        };
        let candidate = if let Some(definition) = definition {
            decode_value(definition, &current_key, current_raw)
        } else if matches!(stored.kind, GameOptionKind::External) {
            Some(CanonicalValue::ExternalRaw(current_raw.to_string()))
        } else {
            None
        };
        let Some(candidate) = candidate else {
            continue;
        };
        if validate_canonical_value(definition, &candidate).is_err() {
            continue;
        }
        if projected_candidate.as_ref().is_some_and(|projected| {
            canonical_values_equal(projected, &candidate)
        }) {
            continue;
        }
        if stored
            .value
            .as_ref()
            .is_some_and(|current| canonical_values_equal(current, &candidate))
        {
            continue;
        }
        let ambiguous_lossy_migration = definition.is_some_and(|definition| {
            matches!(definition.encoding, ValueEncoding::AmbientOcclusion)
                && matches!(field.raw_value.as_str(), "1" | "2")
                && current_raw == "true"
        });
        if ambiguous_lossy_migration {
            continue;
        }
        updates.push((field.option_id.clone(), candidate, stored.revision));
    }

    let mut changed = false;
    let mut fullscreen_value = None;
    if !updates.is_empty() {
        let (canonical_revision, _) =
            load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
        let mut tx = state.pool.begin().await?;
        for (option_id, candidate, revision) in updates {
            if option_id == "fullscreen" {
                fullscreen_value = Some(candidate.clone());
            }
            let next_revision = revision.saturating_add(1) as i64;
            let current_revision = revision as i64;
            let value_json = serde_json::to_string(&candidate)?;
            let canonical_type = candidate.type_name();
            sqlx::query!(
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
    Ok(
        discover_custom_settings(metadata, document, &launcher_keys, state)
            .await?
            || changed,
    )
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
