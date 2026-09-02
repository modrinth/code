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
    UnboundedInteger,
    Integer {
        min: i64,
        max: i64,
        step: i64,
    },
    UnboundedDecimal,
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
    QuotedEnum(&'static [&'static str]),
    Text,
    Graphics,
    AmbientOcclusion,
    ChatPreview,
    Clouds,
    GuiScale,
    MenuBackgroundBlur,
    MusicToast,
    Fov,
    KeyBinding,
}

pub(super) struct SupportedSetting {
    pub(super) id: &'static str,
    pub(super) keys: &'static [&'static str],
    pub(super) versioned_keys: &'static [VersionedKey],
    /// The README's observed range: versions whose fresh options file contains
    /// this key. Versioned settings carry the same ranges on each physical key.
    pub(super) since: &'static str,
    pub(super) until: &'static str,
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
        since: "1.0",
        until: "26.3",
        category,
        default_on,
        editor,
        encoding,
    }
}

pub(super) const fn ranged_setting(
    id: &'static str,
    keys: &'static [&'static str],
    since: &'static str,
    until: &'static str,
    category: &'static str,
    default_on: bool,
    editor: SettingEditor,
    encoding: ValueEncoding,
) -> SupportedSetting {
    SupportedSetting {
        id,
        keys,
        versioned_keys: &[],
        since,
        until,
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
        since: "1.0",
        until: "26.3",
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
const SHARE_PRESENCE: &[&str] = &["all", "limited", "none"];

// These settings all use the common value formats above, so they can stay together.
const GENERAL_SETTINGS: &[SupportedSetting] = &[
    ranged_setting(
        "main_hand",
        &["mainHand"],
        "1.9",
        "26.3",
        "skin_customization",
        true,
        SettingEditor::Enum(MAIN_HAND),
        ValueEncoding::Enum(MAIN_HAND),
    ),
    versioned_setting(
        "cape",
        &["modelPart_cape", "showCape"],
        version_changes::CAPE_KEYS,
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "hat",
        &["modelPart_hat"],
        "1.8",
        "26.3",
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "jacket",
        &["modelPart_jacket"],
        "1.8",
        "26.3",
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "left_sleeve",
        &["modelPart_left_sleeve"],
        "1.8",
        "26.3",
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "right_sleeve",
        &["modelPart_right_sleeve"],
        "1.8",
        "26.3",
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "left_pants_leg",
        &["modelPart_left_pants_leg"],
        "1.8",
        "26.3",
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "right_pants_leg",
        &["modelPart_right_pants_leg"],
        "1.8",
        "26.3",
        "skin_customization",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "allow_server_listing",
        &["allowServerListing"],
        "1.18",
        "26.3",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "realms_notifications",
        &["realmsNotifications"],
        "1.8.9",
        "26.3",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "hide_server_address",
        &["hideServerAddress"],
        "1.3.2",
        "26.3",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "server_textures",
        &["serverTextures"],
        "1.3.1",
        "1.7.5",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "snooper",
        &["snooperEnabled"],
        "1.3.1",
        "1.17.1",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "extra_telemetry",
        &["telemetryOptInExtra"],
        "1.19.3",
        "26.3",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "in_game_notifications",
        &["inGameNotification"],
        "26.2",
        "26.3",
        "online",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "share_presence",
        &["sharePresence"],
        "26.2",
        "26.3",
        "online",
        true,
        SettingEditor::Enum(SHARE_PRESENCE),
        ValueEncoding::QuotedEnum(SHARE_PRESENCE),
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

pub(super) fn setting_available_for_version(
    setting: &SupportedSetting,
    version: &str,
) -> bool {
    if !setting.versioned_keys.is_empty() {
        return physical_variant_for_version(setting, version).is_some();
    }
    let Some(version) = release_version(version) else {
        return false;
    };
    release_version(setting.since).is_some_and(|since| since <= version)
        && release_version(setting.until).is_some_and(|until| version <= until)
}

pub(super) fn target_key_for_version<'a>(
    setting: &'a SupportedSetting,
    version: &str,
) -> Option<&'a str> {
    if setting.versioned_keys.is_empty() {
        setting_available_for_version(setting, version)
            .then(|| setting.keys.first().copied())
            .flatten()
    } else {
        physical_variant_for_version(setting, version)
            .map(|variant| variant.key)
    }
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
        SettingEditor::UnboundedInteger => {
            editor.type_ = "integer".to_string();
            editor.step = Some(1.0);
        }
        SettingEditor::Integer { min, max, step } => {
            editor.type_ = "integer".to_string();
            editor.min = Some(min as f64);
            editor.max = Some(max as f64);
            editor.step = Some(step as f64);
        }
        SettingEditor::UnboundedDecimal => {
            editor.type_ = "decimal".to_string();
            editor.step = Some(0.01);
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
