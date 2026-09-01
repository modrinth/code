//! Loads, previews, and saves the game settings shown in the app.

use super::CATALOG_REVISION;
use super::api_types::*;
use super::instance_support::{
    describe_instance_support, find_common_local_value,
    load_participating_instances, summary_revision,
};
use super::options_file::input_error;
use super::supported_settings::*;
use super::write_shared_settings::sync_all_participating_instances;
use crate::state::{
    CanonicalValue, GameOptionKind, State, StoredOption,
    game_options_sync_is_enabled, load_game_option_preferences,
    load_game_options_sync_state, load_shared_game_options,
};
use chrono::Utc;

fn validation_issue(
    sync_enabled: bool,
    value_state: GameOptionValueState,
    value: Option<&CanonicalValue>,
    compatibility: &GameOptionCompatibility,
) -> Option<GameOptionValidationIssue> {
    if !sync_enabled {
        return None;
    }
    if value_state == GameOptionValueState::UniformLocal {
        return Some(GameOptionValidationIssue::LocalValueNeedsSaving);
    }
    if value.is_none() {
        return Some(GameOptionValidationIssue::MissingValue);
    }
    if compatibility.total_participating > 0 && compatibility.will_receive == 0
    {
        return Some(GameOptionValidationIssue::NoCompatibleInstances);
    }
    None
}

/// Loads everything shown in the game-settings modal.
///
/// It returns message IDs and setting details; the frontend provides the translated
/// labels and explanations.
pub(super) async fn load_settings_editor(
    state: &State,
) -> crate::Result<GameSettingsEditorState> {
    let (canonical_revision, catalog_revision) =
        load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
    let values = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let participants = load_participating_instances(state).await?;
    let mut settings = Vec::new();

    for definition in all_supported_settings() {
        let retained = values.contains_key(definition.id)
            || preferences.contains_key(definition.id);
        let observed = participants.iter().any(|participant| {
            participant.document.as_ref().is_some_and(|document| {
                definition
                    .keys
                    .iter()
                    .any(|key| document.value(key).is_some())
            })
        });
        if !retained && !observed {
            continue;
        }
        let stored = values.get(definition.id).cloned().unwrap_or_else(|| {
            StoredOption {
                option_id: definition.id.to_string(),
                kind: GameOptionKind::Vanilla,
                raw_key: None,
                value: None,
                seeded: false,
                revision: 0,
            }
        });
        let preference = preferences.get(definition.id);
        let (canonical_value, value_state) = if stored.seeded {
            (stored.value.clone(), GameOptionValueState::Canonical)
        } else {
            find_common_local_value(Some(definition), &stored, &participants)
        };
        let compatibility = describe_instance_support(
            Some(definition),
            &stored,
            canonical_value.as_ref(),
            &participants,
        );
        let sync_enabled =
            preference.map(|value| value.enabled).unwrap_or(false);
        let validation_error = validation_issue(
            sync_enabled,
            value_state,
            canonical_value.as_ref(),
            &compatibility,
        );
        settings.push(EditableGameSetting {
            option_id: definition.id.to_string(),
            category_id: definition.category.to_string(),
            kind: GameOptionKind::Vanilla,
            raw_key: None,
            sync_enabled,
            canonical_value,
            value_state,
            option_revision: stored
                .revision
                .max(preference.map(|value| value.revision).unwrap_or(0)),
            editor: editor_for(definition),
            compatibility,
            validation_error,
            controlled: false,
        });
    }

    let mut custom_values = values
        .values()
        .filter(|value| {
            matches!(value.kind, GameOptionKind::External)
                && value
                    .raw_key
                    .as_deref()
                    .is_none_or(|key| setting_by_file_key(key).is_none())
        })
        .cloned()
        .collect::<Vec<_>>();
    custom_values.sort_by(|left, right| left.raw_key.cmp(&right.raw_key));
    for stored in custom_values {
        let preference = preferences.get(&stored.option_id);
        let (canonical_value, value_state) = if stored.seeded {
            (stored.value.clone(), GameOptionValueState::Canonical)
        } else {
            find_common_local_value(None, &stored, &participants)
        };
        let compatibility = describe_instance_support(
            None,
            &stored,
            canonical_value.as_ref(),
            &participants,
        );
        let sync_enabled = preference.is_some_and(|value| value.enabled);
        let validation_error = validation_issue(
            sync_enabled,
            value_state,
            canonical_value.as_ref(),
            &compatibility,
        );
        settings.push(EditableGameSetting {
            option_id: stored.option_id.clone(),
            category_id: "custom".to_string(),
            kind: GameOptionKind::External,
            raw_key: stored.raw_key.clone(),
            sync_enabled,
            canonical_value,
            value_state,
            option_revision: stored
                .revision
                .max(preference.map(|value| value.revision).unwrap_or(0)),
            editor: custom_setting_editor(),
            compatibility,
            validation_error,
            controlled: false,
        });
    }

    let category_definitions = [
        "skin_customization",
        "video",
        "language",
        "music_and_sound",
        "controls",
        "chat",
        "accessibility",
        "online",
        "custom",
    ];
    let categories = category_definitions
        .into_iter()
        .filter(|id| {
            *id == "custom"
                || settings.iter().any(|setting| setting.category_id == *id)
        })
        .map(|id| GameSettingCategory {
            id: id.to_string(),
            is_custom: id == "custom",
        })
        .collect();

    Ok(GameSettingsEditorState {
        summary_revision: summary_revision(canonical_revision, &participants),
        canonical_revision,
        catalog_revision,
        total_participating: participants.len() as u32,
        categories,
        settings,
    })
}

pub async fn get_config() -> crate::Result<GameSettingsEditorState> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    load_settings_editor(&state).await
}

/// Previews how the user's unsaved changes would apply to their instances.
pub async fn preview_changes(
    request: UpdateGameSettingsRequest,
) -> crate::Result<GameSettingsEditorState> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    if request.expected_catalog_revision != CATALOG_REVISION {
        return Ok(load_settings_editor(&state).await?);
    }
    let mut editor_state = load_settings_editor(&state).await?;
    let participants = load_participating_instances(&state).await?;
    for change in request.changes {
        let Some(setting) = editor_state
            .settings
            .iter_mut()
            .find(|setting| setting.option_id == change.option_id)
        else {
            continue;
        };
        if setting.option_revision != change.base_option_revision {
            setting.validation_error =
                Some(GameOptionValidationIssue::ChangedSinceOpened);
            continue;
        }
        if let Some(value) = change.canonical_value {
            let definition = setting_by_id(&setting.option_id);
            if let Some(value) = value {
                if validate_canonical_value(definition, &value).is_err() {
                    setting.validation_error =
                        Some(GameOptionValidationIssue::InvalidValue);
                } else {
                    setting.canonical_value = Some(value);
                    setting.value_state = GameOptionValueState::Canonical;
                    setting.validation_error = None;
                }
            } else {
                setting.canonical_value = None;
                setting.value_state = GameOptionValueState::Unset;
                setting.validation_error = None;
            }
        }
        if let Some(enabled) = change.sync_enabled {
            setting.sync_enabled = enabled;
        }
        let stored = StoredOption {
            option_id: setting.option_id.clone(),
            kind: setting.kind,
            raw_key: setting.raw_key.clone(),
            value: setting.canonical_value.clone(),
            seeded: setting.canonical_value.is_some(),
            revision: setting.option_revision,
        };
        setting.compatibility = describe_instance_support(
            setting_by_id(&setting.option_id),
            &stored,
            setting.canonical_value.as_ref(),
            &participants,
        );
        if setting.sync_enabled && setting.canonical_value.is_none() {
            setting.validation_error =
                Some(GameOptionValidationIssue::MissingValue);
        } else if setting.sync_enabled
            && setting.compatibility.total_participating > 0
            && setting.compatibility.will_receive == 0
        {
            setting.validation_error =
                Some(GameOptionValidationIssue::NoCompatibleInstances);
        }
    }
    Ok(editor_state)
}

/// Saves changes if nothing has changed since the modal opened, then updates each
/// instance that can receive them.
pub async fn save_changes(
    request: UpdateGameSettingsRequest,
) -> crate::Result<SaveGameSettingsResult> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    if !game_options_sync_is_enabled(&state.pool).await? {
        return Err(input_error(
            "Turn on game-settings sync before editing shared values.",
        ));
    }
    let current_editor = load_settings_editor(&state).await?;
    let UpdateGameSettingsRequest {
        expected_summary_revision,
        expected_canonical_revision,
        expected_catalog_revision,
        changes,
    } = request;
    if expected_catalog_revision != CATALOG_REVISION
        || expected_summary_revision != current_editor.summary_revision
        || expected_canonical_revision != current_editor.canonical_revision
    {
        return Ok(SaveGameSettingsResult {
            state: Some(current_editor),
            conflicts: changes
                .into_iter()
                .map(|change| change.option_id)
                .collect(),
            ..SaveGameSettingsResult::default()
        });
    }
    let stored_values = load_shared_game_options(&state.pool).await?;
    let participants = load_participating_instances(&state).await?;
    let mut conflicts = Vec::new();
    let mut accepted = Vec::new();

    for mut change in changes {
        let Some(setting) = current_editor
            .settings
            .iter()
            .find(|setting| setting.option_id == change.option_id)
        else {
            return Err(input_error(format!(
                "Unknown game setting {}",
                change.option_id
            )));
        };
        if setting.option_revision != change.base_option_revision {
            conflicts.push(change.option_id);
            continue;
        }
        if let Some(Some(value)) = &change.canonical_value {
            validate_canonical_value(setting_by_id(&setting.option_id), value)?;
        }
        let resulting_enabled =
            change.sync_enabled.unwrap_or(setting.sync_enabled);
        let stored_seeded = stored_values
            .get(&setting.option_id)
            .is_some_and(|stored| stored.seeded);
        if resulting_enabled
            && !stored_seeded
            && change.canonical_value.is_none()
            && setting.canonical_value.is_some()
        {
            change.canonical_value = Some(setting.canonical_value.clone());
        }
        let resulting_value = match &change.canonical_value {
            Some(value) => value.as_ref(),
            None => setting.canonical_value.as_ref(),
        };
        if resulting_enabled {
            if resulting_value.is_none() {
                return Err(input_error(format!(
                    "Choose a value before enabling {}",
                    setting.option_id
                )));
            }
            let projected = StoredOption {
                option_id: setting.option_id.clone(),
                kind: setting.kind,
                raw_key: setting.raw_key.clone(),
                value: resulting_value.cloned(),
                seeded: resulting_value.is_some(),
                revision: setting.option_revision,
            };
            let compatibility = describe_instance_support(
                setting_by_id(&setting.option_id),
                &projected,
                resulting_value,
                &participants,
            );
            if compatibility.total_participating > 0
                && compatibility.will_receive == 0
            {
                return Err(input_error(format!(
                    "{} is not compatible with any participating instance",
                    setting.option_id
                )));
            }
        }
        accepted.push((setting.clone(), change));
    }

    let mut changed = false;
    let mut tx = state.pool.begin().await?;
    for (setting, change) in accepted {
        let value_changed =
            change.canonical_value.as_ref().is_some_and(|value| {
                setting.value_state != GameOptionValueState::Canonical
                    || value.as_ref() != setting.canonical_value.as_ref()
            });
        let selection_changed = change
            .sync_enabled
            .is_some_and(|enabled| enabled != setting.sync_enabled);
        if !value_changed && !selection_changed {
            continue;
        }
        changed = true;
        let revision = setting.option_revision.saturating_add(1) as i64;
        if let Some(value) = change.canonical_value {
            let now = Utc::now().timestamp();
            if let Some(value) = value {
                let value_json = serde_json::to_string(&value)?;
                let canonical_type = value.type_name();
                let kind = if matches!(setting.kind, GameOptionKind::External) {
                    "external"
                } else {
                    "vanilla"
                };
                let raw_key = setting.raw_key.as_deref();
                let value_codec = if setting_by_id(&setting.option_id).is_some()
                {
                    "catalog"
                } else {
                    "exact_raw"
                };
                let source_game_version: Option<&str> = None;
                sqlx::query!(
                    "
				INSERT INTO synced_game_option_values
					(option_id, kind, raw_key, canonical_type,
					 canonical_value_json, value_codec, seeded, revision, origin,
					 source_game_version, source_instance_id, updated_at)
				VALUES (?, ?, ?, ?, ?, ?, 1, ?, 'app_editor', ?, NULL, ?)
				ON CONFLICT(option_id) DO UPDATE SET
					kind = excluded.kind, raw_key = excluded.raw_key,
					canonical_type = excluded.canonical_type,
					canonical_value_json = excluded.canonical_value_json,
					value_codec = excluded.value_codec, seeded = 1,
					revision = excluded.revision, origin = 'app_editor',
					source_game_version = NULL, source_instance_id = NULL,
					updated_at = excluded.updated_at
				",
                    setting.option_id,
                    kind,
                    raw_key,
                    canonical_type,
                    value_json,
                    value_codec,
                    revision,
                    source_game_version,
                    now,
                )
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query!(
                    "
					UPDATE synced_game_option_values
					SET canonical_value_json = NULL, seeded = 0, revision = ?,
						origin = 'app_editor', source_game_version = NULL,
						source_instance_id = NULL, updated_at = ?
					WHERE option_id = ?
					",
                    revision,
                    now,
                    setting.option_id,
                )
                .execute(&mut *tx)
                .await?;
            }
        }
        if let Some(enabled) = change.sync_enabled {
            sqlx::query!(
                "
				INSERT INTO synced_game_option_preferences
					(option_id, enabled, source, revision)
				VALUES (?, ?, ?, ?)
				ON CONFLICT(option_id) DO UPDATE SET
					enabled = excluded.enabled, source = 'user',
					revision = excluded.revision
				",
                setting.option_id,
                enabled,
                "user",
                revision,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    if changed {
        let next_revision =
            current_editor.canonical_revision.saturating_add(1) as i64;
        let catalog_revision = CATALOG_REVISION as i64;
        sqlx::query!(
            "
			INSERT INTO synced_game_option_state (singleton, revision, catalog_revision)
			VALUES (1, ?, ?)
			ON CONFLICT(singleton) DO UPDATE SET
				revision = excluded.revision,
				catalog_revision = excluded.catalog_revision
			",
            next_revision,
            catalog_revision,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    let projection = if changed {
        sync_all_participating_instances(&state).await
    } else {
        SaveGameSettingsResult {
            state: None,
            applied: 0,
            migrated: 0,
            deferred: 0,
            unsupported: 0,
            failed: 0,
            conflicts: Vec::new(),
        }
    };
    Ok(SaveGameSettingsResult {
        state: Some(load_settings_editor(&state).await?),
        conflicts,
        ..projection
    })
}
