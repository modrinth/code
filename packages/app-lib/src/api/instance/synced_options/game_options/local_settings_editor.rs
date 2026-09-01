//! Loads and saves the game settings stored by one unsynced instance.

use super::super as synced_options;
use super::CATALOG_REVISION;
use super::api_types::*;
use super::catalog::*;
use super::launch_overrides::currently_launcher_owned_keys;
use super::options_file::{
    GameOptionsDocument, input_error, options_path, read_document, sha1_bytes,
    validate_raw_key_value,
};
use super::read_instance_changes::custom_setting_id;
use crate::state::{CanonicalValue, GameOptionKind, InstanceMetadata, State};
use crate::util::io;
use std::collections::HashSet;

fn compatibility(controlled: bool) -> GameOptionCompatibility {
    if controlled {
        GameOptionCompatibility {
            total_participating: 1,
            will_receive: 0,
            write_now: 0,
            left_local: 1,
            buckets: vec![GameOptionCompatibilityBucket {
                instance_count: 1,
                write_keys: Vec::new(),
                eventual_keys: Vec::new(),
                game_versions: Vec::new(),
                status: GameOptionCompatibilityStatus::Controlled,
                mapping: None,
                reason: Some(
                    GameOptionCompatibilityReason::LauncherControlled,
                ),
            }],
        }
    } else {
        GameOptionCompatibility {
            total_participating: 1,
            will_receive: 1,
            write_now: 1,
            left_local: 0,
            buckets: Vec::new(),
        }
    }
}

fn categories(settings: &[EditableGameSetting]) -> Vec<GameSettingCategory> {
    [
        "skin_customization",
        "video",
        "language",
        "music_and_sound",
        "controls",
        "chat",
        "accessibility",
        "online",
        "custom",
    ]
    .into_iter()
    .filter(|id| settings.iter().any(|setting| setting.category_id == *id))
    .map(|id| GameSettingCategory {
        id: id.to_string(),
        is_custom: id == "custom",
    })
    .collect()
}

fn editor_revision(metadata: &InstanceMetadata, input_bytes: &[u8]) -> String {
    sha1_bytes(
        format!(
            "{CATALOG_REVISION}:{}:{}",
            metadata.applied_content_set.game_version,
            sha1_bytes(input_bytes)
        )
        .as_bytes(),
    )
}

async fn load_instance_document(
    instance_id: &str,
    state: &State,
) -> crate::Result<(
    InstanceMetadata,
    GameOptionsDocument,
    Vec<u8>,
    HashSet<String>,
)> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| input_error("Unknown instance"))?;
    if crate::state::game_options_sync_is_enabled(&state.pool).await?
        && metadata.synced_options.game_options
    {
        return Err(input_error(
            "This instance is using synced game settings.",
        ));
    }
    if synced_options::sync_files_are_protected(&metadata) {
        return Err(input_error(
            "Game settings cannot be edited while this instance is installing or updating.",
        ));
    }
    if synced_options::instance_is_running(&metadata, state).await? {
        return Err(input_error(
            "Game settings cannot be edited while this instance is running.",
        ));
    }
    let path = options_path(&metadata, state);
    if !path.exists() {
        return Err(input_error(
            "Launch this instance once so Minecraft can create options.txt.",
        ));
    }
    let (document, input_bytes) = read_document(&path).await?;
    let mut controlled_keys = currently_launcher_owned_keys(&metadata);
    if metadata.launch_overrides.force_fullscreen.is_none()
        && crate::state::Settings::get(&state.pool)
            .await?
            .force_fullscreen
    {
        controlled_keys.insert("fullscreen".to_string());
    }
    Ok((metadata, document, input_bytes, controlled_keys))
}

fn build_editor_state(
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    input_bytes: &[u8],
    controlled_keys: &HashSet<String>,
) -> GameSettingsEditorState {
    let game_version = metadata.applied_content_set.game_version.as_str();
    let mut settings = Vec::new();

    if supported_settings_cover_game_version(game_version) {
        for definition in all_supported_settings() {
            let Some(physical_key) =
                observed_physical_key(definition, document, game_version)
            else {
                continue;
            };
            let Some(value) = document.value(&physical_key).and_then(|raw| {
                decode_value_for_version(
                    definition,
                    &physical_key,
                    raw,
                    game_version,
                )
            }) else {
                continue;
            };
            let controlled = controlled_keys.contains(definition.id);
            settings.push(EditableGameSetting {
                option_id: definition.id.to_string(),
                category_id: definition.category.to_string(),
                kind: GameOptionKind::Vanilla,
                raw_key: None,
                sync_enabled: true,
                canonical_value: Some(value),
                value_state: GameOptionValueState::Canonical,
                option_revision: 0,
                editor: editor_for(definition),
                compatibility: compatibility(controlled),
                validation_error: None,
                controlled,
            });
        }
    }

    let mut custom_settings = document
        .effective_entries()
        .into_iter()
        .filter_map(|(key, (index, _))| {
            if controlled_keys.contains(key)
                || setting_by_file_key(key).is_some()
                || is_never_sync_key(key)
            {
                return None;
            }
            let entry = document.lines[index].entry.as_ref()?;
            validate_raw_key_value(key, &entry.value).ok()?;
            Some(EditableGameSetting {
                option_id: custom_setting_id(key),
                category_id: "custom".to_string(),
                kind: GameOptionKind::External,
                raw_key: Some(key.to_string()),
                sync_enabled: true,
                canonical_value: Some(CanonicalValue::ExternalRaw(
                    entry.value.clone(),
                )),
                value_state: GameOptionValueState::Canonical,
                option_revision: 0,
                editor: custom_setting_editor(),
                compatibility: compatibility(false),
                validation_error: None,
                controlled: false,
            })
        })
        .collect::<Vec<_>>();
    custom_settings.sort_by(|left, right| left.raw_key.cmp(&right.raw_key));
    settings.extend(custom_settings);

    GameSettingsEditorState {
        summary_revision: editor_revision(metadata, input_bytes),
        canonical_revision: 0,
        catalog_revision: CATALOG_REVISION,
        total_participating: 1,
        categories: categories(&settings),
        settings,
    }
}

fn encode_setting_value(
    metadata: &InstanceMetadata,
    document: &GameOptionsDocument,
    setting: &EditableGameSetting,
    value: &CanonicalValue,
) -> crate::Result<(String, String)> {
    let game_version = metadata.applied_content_set.game_version.as_str();
    if let Some(definition) = setting_by_id(&setting.option_id) {
        validate_canonical_value(Some(definition), value)?;
        let key = observed_physical_key(definition, document, game_version)
            .ok_or_else(|| {
                input_error(format!(
                    "{} is no longer present in options.txt",
                    setting.option_id
                ))
            })?;
        let raw = encode_value(
            definition,
            &key,
            value,
            game_version,
            document.value(&key),
        )
        .ok_or_else(|| {
            input_error(format!(
                "{} cannot be represented by this Minecraft version",
                setting.option_id
            ))
        })?;
        Ok((key, raw))
    } else {
        validate_canonical_value(None, value)?;
        let key = setting
            .raw_key
            .clone()
            .ok_or_else(|| input_error("Custom setting is missing its file key"))?;
        let CanonicalValue::ExternalRaw(raw) = value else {
            return Err(input_error(
                "Custom settings require a raw string value",
            ));
        };
        validate_raw_key_value(&key, raw)?;
        Ok((key, raw.clone()))
    }
}

async fn load_editor(
    instance_id: &str,
    state: &State,
) -> crate::Result<GameSettingsEditorState> {
    let (metadata, document, input_bytes, controlled_keys) =
        load_instance_document(instance_id, state).await?;
    Ok(build_editor_state(
        &metadata,
        &document,
        &input_bytes,
        &controlled_keys,
    ))
}

pub async fn get_config(
    instance_id: &str,
) -> crate::Result<GameSettingsEditorState> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    load_editor(instance_id, &state).await
}

pub async fn preview_changes(
    instance_id: &str,
    request: UpdateGameSettingsRequest,
) -> crate::Result<GameSettingsEditorState> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let (metadata, document, input_bytes, controlled_keys) =
        load_instance_document(instance_id, &state).await?;
    let mut editor = build_editor_state(
        &metadata,
        &document,
        &input_bytes,
        &controlled_keys,
    );
    let stale = request.expected_catalog_revision != CATALOG_REVISION
        || request.expected_summary_revision != editor.summary_revision
        || request.expected_canonical_revision != editor.canonical_revision;

    for change in request.changes {
        let Some(setting) = editor
            .settings
            .iter_mut()
            .find(|setting| setting.option_id == change.option_id)
        else {
            continue;
        };
        if stale || change.base_option_revision != setting.option_revision {
            setting.validation_error =
                Some(GameOptionValidationIssue::ChangedSinceOpened);
            continue;
        }
        if change.sync_enabled.is_some() {
            setting.validation_error =
                Some(GameOptionValidationIssue::InvalidValue);
            continue;
        }
        if let Some(value) = change.canonical_value {
            match value {
                Some(value)
                    if encode_setting_value(
                        &metadata,
                        &document,
                        setting,
                        &value,
                    )
                    .is_ok() =>
                {
                    setting.canonical_value = Some(value);
                    setting.value_state = GameOptionValueState::Canonical;
                    setting.validation_error = None;
                }
                _ => {
                    setting.canonical_value = None;
                    setting.value_state = GameOptionValueState::Invalid;
                    setting.validation_error =
                        Some(GameOptionValidationIssue::InvalidValue);
                }
            }
        }
    }
    Ok(editor)
}

pub async fn save_changes(
    instance_id: &str,
    request: UpdateGameSettingsRequest,
) -> crate::Result<SaveGameSettingsResult> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let (metadata, mut document, input_bytes, controlled_keys) =
        load_instance_document(instance_id, &state).await?;
    let current_editor = build_editor_state(
        &metadata,
        &document,
        &input_bytes,
        &controlled_keys,
    );
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

    let mut changed = false;
    for change in changes {
        let setting = current_editor
            .settings
            .iter()
            .find(|setting| setting.option_id == change.option_id)
            .ok_or_else(|| {
                input_error(format!(
                    "Unknown game setting {}",
                    change.option_id
                ))
            })?;
        if setting.controlled {
            return Err(input_error(format!(
                "{} is controlled by the instance's launcher settings",
                setting.option_id
            )));
        }
        if change.base_option_revision != setting.option_revision {
            return Ok(SaveGameSettingsResult {
                state: Some(current_editor),
                conflicts: vec![change.option_id],
                ..SaveGameSettingsResult::default()
            });
        }
        if change.sync_enabled.is_some() {
            return Err(input_error(
                "Sync selection cannot be changed in the instance editor",
            ));
        }
        let Some(Some(value)) = change.canonical_value else {
            return Err(input_error(format!(
                "Choose a value for {}",
                setting.option_id
            )));
        };
        let (key, raw) =
            encode_setting_value(&metadata, &document, setting, &value)?;
        changed |= document.set(&key, &raw, false)?;
    }

    if changed {
        io::write(options_path(&metadata, &state), document.serialize()?)
            .await?;
    }
    let refreshed = load_editor(instance_id, &state).await?;
    Ok(SaveGameSettingsResult {
        state: Some(refreshed),
        applied: u32::from(changed),
        ..SaveGameSettingsResult::default()
    })
}
