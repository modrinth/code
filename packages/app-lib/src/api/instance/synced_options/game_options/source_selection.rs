//! Finds eligible source instances and copies the first shared settings from one.

use super::super as synced_options;
use super::CATALOG_REVISION;
use super::api_types::{GameOptionsSourceCandidate, GameOptionsSourceIssue};
use super::catalog::*;
use super::fullscreen::update_app_fullscreen_setting;
use super::launch_overrides::currently_launcher_owned_keys;
use super::options_file::{
    input_error, options_path, read_document, sha1_bytes,
    validate_raw_key_value,
};
use super::read_instance_changes::{
    custom_setting_id, launcher_keys_for_document,
};
use crate::state::{
    CanonicalValue, InstanceMetadata, State, load_game_option_preferences,
    load_game_options_sync_state, load_shared_game_options,
};
use chrono::Utc;

/// Uses the selected instance's `options.txt` as the initial shared settings.
pub(in crate::api::instance) async fn initialize_from_source_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let path = options_path(metadata, state);
    if !path.exists() {
        return Err(input_error(
            "Launch the source instance once so Minecraft can create options.txt.",
        ));
    }
    let (document, input_bytes) = read_document(&path).await?;
    let input_sha1 = sha1_bytes(&input_bytes);
    let mut launcher_keys =
        launcher_keys_for_document(&metadata.instance.id, &input_sha1, state)
            .await?;
    let currently_launcher_owned = currently_launcher_owned_keys(metadata);
    if !currently_launcher_owned.contains("fullscreen") {
        launcher_keys.remove("fullscreen");
    }
    launcher_keys.extend(currently_launcher_owned);
    let entries = document.effective_entries();
    let current_values = load_shared_game_options(&state.pool).await?;
    let current_preferences = load_game_option_preferences(&state.pool).await?;
    let (current_revision, _) =
        load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
    let now = Utc::now().timestamp();
    let source_version = metadata.applied_content_set.game_version.as_str();
    let source_id = metadata.instance.id.as_str();
    let mut tx = state.pool.begin().await?;
    let catalog_revision = CATALOG_REVISION as i64;

    sqlx::query!(
        "
		INSERT INTO sync_feature_settings
			(feature, globally_enabled, new_instance_default)
		VALUES ('game_options', 1, 1)
		ON CONFLICT(feature) DO UPDATE SET globally_enabled = 1
		",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "
		INSERT INTO instance_sync_preferences (instance_id, feature, enabled)
		VALUES (?, 'game_options', 1)
		ON CONFLICT(instance_id, feature) DO UPDATE SET enabled = 1
		",
        source_id,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "
		INSERT INTO synced_game_option_state (singleton, revision, catalog_revision)
		VALUES (1, 0, ?)
		ON CONFLICT(singleton) DO UPDATE SET
			catalog_revision = excluded.catalog_revision
		",
        catalog_revision,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE synced_game_option_values
		 SET seeded = 0, revision = revision + 1, updated_at = ?",
        now,
    )
    .execute(&mut *tx)
    .await?;
    for definition in all_supported_settings() {
        for key in definition.keys {
            let custom_id = custom_setting_id(key);
            let Some(custom) = current_values.get(&custom_id) else {
                continue;
            };
            let revision = custom
                .revision
                .max(
                    current_preferences
                        .get(&custom_id)
                        .map(|preference| preference.revision)
                        .unwrap_or(0),
                )
                .saturating_add(1) as i64;
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
        }
    }
    let mut seeded_any = false;
    let mut fullscreen_value = None;
    let mut fullscreen_sync_enabled = false;

    for definition in all_supported_settings() {
        if !supported_settings_cover_game_version(source_version) {
            break;
        }
        let found = definition.keys.iter().find_map(|key| {
            if launcher_keys.contains(*key) {
                return None;
            }
            let (index, _) = entries.get(*key).copied()?;
            let entry = document.lines[index].entry.as_ref()?;
            decode_value_for_version(
                definition,
                key,
                &entry.value,
                source_version,
            )
            .map(|value| ((*key).to_string(), value))
        });
        let Some((_physical_key, value)) = found else {
            continue;
        };
        if definition.id == "fullscreen" {
            fullscreen_value = Some(value.clone());
            fullscreen_sync_enabled = current_preferences
                .get(definition.id)
                .map(|preference| preference.enabled)
                .unwrap_or(definition.default_on);
        }
        seeded_any = true;
        let option_revision = current_values
            .get(definition.id)
            .map(|value| value.revision)
            .unwrap_or(0)
            .max(
                current_preferences
                    .get(definition.id)
                    .map(|value| value.revision)
                    .unwrap_or(0),
            )
            .saturating_add(1) as i64;
        let value_json = serde_json::to_string(&value)?;
        let canonical_type = value.type_name();
        sqlx::query!(
            "
			INSERT INTO synced_game_option_values
				(option_id, kind, raw_key, canonical_type,
				 canonical_value_json, value_codec, seeded, revision, origin,
				 source_game_version, source_instance_id, updated_at)
			VALUES (?, 'vanilla', NULL, ?, ?, 'catalog', 1, ?,
				'source_seed', ?, ?, ?)
			ON CONFLICT(option_id) DO UPDATE SET
				kind = 'vanilla', raw_key = NULL,
				canonical_type = excluded.canonical_type,
				canonical_value_json = excluded.canonical_value_json,
				value_codec = excluded.value_codec, seeded = 1,
				revision = excluded.revision, origin = 'source_seed',
				source_game_version = excluded.source_game_version,
				source_instance_id = excluded.source_instance_id,
				updated_at = excluded.updated_at
			",
            definition.id,
            canonical_type,
            value_json,
            option_revision,
            source_version,
            source_id,
            now,
        )
        .execute(&mut *tx)
        .await?;
        let default_enabled = definition.default_on;
        sqlx::query!(
            "
			INSERT INTO synced_game_option_preferences
				(option_id, enabled, source, revision)
			VALUES (?, ?, 'catalog_default', ?)
			ON CONFLICT(option_id) DO NOTHING
			",
            definition.id,
            default_enabled,
            option_revision,
        )
        .execute(&mut *tx)
        .await?;
    }

    for (key, (index, _)) in entries {
        if launcher_keys.contains(key)
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
        let value = CanonicalValue::ExternalRaw(entry.value.clone());
        seeded_any = true;
        let option_revision = current_values
            .get(&option_id)
            .map(|value| value.revision)
            .unwrap_or(0)
            .max(
                current_preferences
                    .get(&option_id)
                    .map(|value| value.revision)
                    .unwrap_or(0),
            )
            .saturating_add(1) as i64;
        let value_json = serde_json::to_string(&value)?;
        sqlx::query!(
            "
			INSERT INTO synced_game_option_values
				(option_id, kind, raw_key, canonical_type,
				 canonical_value_json, value_codec, seeded, revision, origin,
				 source_game_version, source_instance_id, updated_at)
			VALUES (?, 'external', ?, 'external_raw', ?, 'exact_raw',
				1, ?, 'source_seed', ?, ?, ?)
			ON CONFLICT(option_id) DO UPDATE SET
				raw_key = excluded.raw_key,
				canonical_value_json = excluded.canonical_value_json,
				seeded = 1, revision = excluded.revision,
				origin = 'source_seed',
				source_game_version = excluded.source_game_version,
				source_instance_id = excluded.source_instance_id,
				updated_at = excluded.updated_at
			",
            option_id,
            key,
            value_json,
            option_revision,
            source_version,
            source_id,
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
            option_revision,
        )
        .execute(&mut *tx)
        .await?;
    }
    if !seeded_any {
        return Err(input_error(
            "The selected options.txt has no settings that can be safely used as a sync baseline.",
        ));
    }

    let next_revision = current_revision.saturating_add(1) as i64;
    sqlx::query!(
        "
		UPDATE synced_game_option_state
		SET revision = ?, catalog_revision = ?
		WHERE singleton = 1
		",
        next_revision,
        catalog_revision,
    )
    .execute(&mut *tx)
    .await?;
    if let Some(value) = fullscreen_value.as_ref() {
        update_app_fullscreen_setting(&mut tx, value, fullscreen_sync_enabled)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_sync_sources()
-> crate::Result<Vec<GameOptionsSourceCandidate>> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let instances = crate::state::list_instances(&state.pool).await?;
    let mut sources = Vec::with_capacity(instances.len());
    for metadata in instances {
        let path = options_path(&metadata, &state);
        let version_supported = supported_settings_cover_game_version(
            &metadata.applied_content_set.game_version,
        );
        let mut eligible = true;
        let mut disabled_reason = None;
        let mut recognized_setting_count = 0;
        let mut custom_setting_count = 0;
        if synced_options::sync_files_are_protected(&metadata) {
            eligible = false;
            disabled_reason =
                Some(GameOptionsSourceIssue::InstallingOrUpdating);
        } else if synced_options::instance_is_running(&metadata, &state).await?
        {
            eligible = false;
            disabled_reason = Some(GameOptionsSourceIssue::Running);
        } else if !path.exists() {
            eligible = false;
            disabled_reason = Some(GameOptionsSourceIssue::MissingOptionsFile);
        } else {
            match read_document(&path).await {
                Ok((document, input_bytes)) => {
                    let mut launcher_keys = launcher_keys_for_document(
                        &metadata.instance.id,
                        &sha1_bytes(&input_bytes),
                        &state,
                    )
                    .await?;
                    let currently_launcher_owned =
                        currently_launcher_owned_keys(&metadata);
                    if !currently_launcher_owned.contains("fullscreen") {
                        launcher_keys.remove("fullscreen");
                    }
                    launcher_keys.extend(currently_launcher_owned);
                    let entries = document.effective_entries();
                    let source_version =
                        metadata.applied_content_set.game_version.as_str();
                    let recognized = if version_supported {
                        all_supported_settings()
                            .filter(|setting| {
                                setting.keys.iter().any(|key| {
                                    if launcher_keys.contains(*key) {
                                        return false;
                                    }
                                    document.value(key).is_some_and(|raw| {
                                        decode_value_for_version(
                                            setting,
                                            key,
                                            raw,
                                            source_version,
                                        )
                                        .is_some()
                                    })
                                })
                            })
                            .count()
                    } else {
                        0
                    };
                    let custom = entries
                        .into_iter()
                        .filter(|(key, (index, _))| {
                            !launcher_keys.contains(*key)
                                && setting_by_file_key(key).is_none()
                                && !is_never_sync_key(key)
                                && document.lines[*index]
                                    .entry
                                    .as_ref()
                                    .is_some_and(|entry| {
                                        validate_raw_key_value(
                                            key,
                                            &entry.value,
                                        )
                                        .is_ok()
                                    })
                        })
                        .count();
                    if recognized + custom == 0 {
                        eligible = false;
                        disabled_reason = Some(if version_supported {
                            GameOptionsSourceIssue::NoSyncableSettings
                        } else {
                            GameOptionsSourceIssue::UnsupportedVersion
                        });
                    } else {
                        recognized_setting_count = recognized as u32;
                        custom_setting_count = custom as u32;
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        "Could not inspect {} as a game-settings source: {error}",
                        metadata.instance.id
                    );
                    eligible = false;
                    disabled_reason =
                        Some(GameOptionsSourceIssue::UnreadableOptionsFile);
                }
            }
        }
        sources.push(GameOptionsSourceCandidate {
            source_id: metadata.instance.id.clone(),
            instance_id: metadata.instance.id,
            name: metadata.instance.name,
            icon_path: metadata.instance.icon_path,
            game_version: Some(metadata.applied_content_set.game_version),
            eligible,
            disabled_reason,
            recognized_setting_count,
            custom_setting_count,
        });
    }
    sources.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.instance_id.cmp(&right.instance_id))
    });
    Ok(sources)
}
