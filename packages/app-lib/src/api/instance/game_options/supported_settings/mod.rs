//! Defines the Minecraft settings Modrinth knows how to read and write.
//!
//! File keys and version conversions live here. The frontend supplies the labels
//! and translations shown to the user.

mod accessibility;
mod chat;
mod controls;
mod known_vanilla_keys;
mod legacy_key_bindings;
mod sound;
mod version_changes;
mod video;

use super::api_types::{
    GameOptionEditorChoice, GameOptionEditorDefinition, GameOptionMappingKind,
};

use known_vanilla_keys::{KNOWN_VANILLA_KEYS, NEVER_SYNC_KEYS};
pub(in crate::api::instance) use legacy_key_bindings::LEGACY_KEY_BINDINGS;
pub(in crate::api::instance) use version_changes::*;

#[derive(Clone, Copy)]
pub(super) enum SettingEditor {
    Boolean,
    Integer {
        min: i64,
        max: i64,
        step: i64,
    },
    Decimal {
        min: f64,
        max: f64,
        step: f64,
        unit: Option<&'static str>,
    },
    Enum(&'static [&'static str]),
    Language,
    KeyBinding,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ValueEncoding {
    Bool,
    Integer,
    Decimal,
    Enum(&'static [&'static str]),
    Text,
    Graphics,
    AmbientOcclusion,
    MusicToast,
    Fov,
    KeyBinding,
}

pub(super) struct SupportedSetting {
    pub(super) id: &'static str,
    pub(super) keys: &'static [&'static str],
    pub(super) versioned_keys: &'static [VersionedKey],
    pub(super) category: &'static str,
    pub(super) default_on: bool,
    pub(super) editor: SettingEditor,
    pub(super) encoding: ValueEncoding,
}

#[derive(Clone, Copy)]
pub(super) struct VersionedKey {
    pub(super) key: &'static str,
    pub(super) since: &'static str,
    pub(super) until: &'static str,
    pub(super) mapping: GameOptionMappingKind,
}

pub(super) const BOOL: SettingEditor = SettingEditor::Boolean;
pub(super) const UNIT_INTERVAL: SettingEditor = SettingEditor::Decimal {
    min: 0.0,
    max: 1.0,
    step: 0.01,
    unit: Some("percent"),
};

pub(super) const fn setting(
    id: &'static str,
    keys: &'static [&'static str],
    category: &'static str,
    default_on: bool,
    editor: SettingEditor,
    encoding: ValueEncoding,
) -> SupportedSetting {
    SupportedSetting {
        id,
        keys,
        versioned_keys: &[],
        category,
        default_on,
        editor,
        encoding,
    }
}

pub(super) const fn versioned_setting(
    id: &'static str,
    keys: &'static [&'static str],
    versioned_keys: &'static [VersionedKey],
    category: &'static str,
    default_on: bool,
    editor: SettingEditor,
    encoding: ValueEncoding,
) -> SupportedSetting {
    SupportedSetting {
        id,
        keys,
        versioned_keys,
        category,
        default_on,
        editor,
        encoding,
    }
}

const SETTING_GROUPS: &[&[SupportedSetting]] = &[
    video::SETTINGS,
    sound::SETTINGS,
    controls::SETTINGS,
    chat::SETTINGS,
    accessibility::SETTINGS,
    GENERAL_SETTINGS,
];

const MAIN_HAND: &[&str] = &["left", "right"];

// These settings all use the common value formats above, so they can stay together.
const GENERAL_SETTINGS: &[SupportedSetting] = &[
    setting(
        "main_hand",
        &["mainHand"],
        "skin_customization",
        true,
        SettingEditor::Enum(MAIN_HAND),
        ValueEncoding::Enum(MAIN_HAND),
    ),
    setting(
        "cape",
        &["modelPart_cape"],
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "hat",
        &["modelPart_hat"],
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "jacket",
        &["modelPart_jacket"],
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "allow_server_listing",
        &["allowServerListing"],
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "realms_notifications",
        &["realmsNotifications"],
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
];

pub(super) fn all_supported_settings()
-> impl Iterator<Item = &'static SupportedSetting> {
    SETTING_GROUPS.iter().flat_map(|group| group.iter())
}

pub(super) fn setting_by_id(
    option_id: &str,
) -> Option<&'static SupportedSetting> {
    all_supported_settings().find(|setting| setting.id == option_id)
}

pub(super) fn setting_by_file_key(
    key: &str,
) -> Option<&'static SupportedSetting> {
    all_supported_settings().find(|setting| setting.keys.contains(&key))
}

pub(super) fn is_never_sync_key(key: &str) -> bool {
    NEVER_SYNC_KEYS.contains(&key)
        || KNOWN_VANILLA_KEYS.contains(&key)
        || key.starts_with("skip")
        || key.starts_with("hideBundleTutorial")
        || key.starts_with("key_") && !key.starts_with("key_key.")
}

pub(super) fn editor_for(
    setting: &SupportedSetting,
) -> GameOptionEditorDefinition {
    let mut editor = GameOptionEditorDefinition {
        type_: String::new(),
        min: None,
        max: None,
        step: None,
        unit: None,
        choices: Vec::new(),
    };
    match setting.editor {
        SettingEditor::Boolean => editor.type_ = "bool".to_string(),
        SettingEditor::Integer { min, max, step } => {
            editor.type_ = "integer".to_string();
            editor.min = Some(min as f64);
            editor.max = Some(max as f64);
            editor.step = Some(step as f64);
        }
        SettingEditor::Decimal {
            min,
            max,
            step,
            unit,
        } => {
            editor.type_ = "decimal".to_string();
            editor.min = Some(min);
            editor.max = Some(max);
            editor.step = Some(step);
            editor.unit = unit.map(str::to_string);
        }
        SettingEditor::Enum(choices) => {
            editor.type_ = "enum".to_string();
            editor.choices = choices
                .iter()
                .map(|value| GameOptionEditorChoice {
                    value: (*value).to_string(),
                })
                .collect();
        }
        SettingEditor::Language => editor.type_ = "text".to_string(),
        SettingEditor::KeyBinding => editor.type_ = "key_binding".to_string(),
    }
    editor
}

pub(super) fn custom_setting_editor() -> GameOptionEditorDefinition {
    GameOptionEditorDefinition {
        type_: "external_raw".to_string(),
        min: None,
        max: None,
        step: None,
        unit: None,
        choices: Vec::new(),
    }
}
