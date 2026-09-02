//! Handles settings Minecraft renamed or stored differently between versions.

use super::super::api_types::canonical_values_equal;
use super::super::options_file::{
    GameOptionsDocument, input_error, validate_raw_key_value,
};
use super::{
    GameOptionMappingKind, LEGACY_KEY_BINDINGS, SettingEditor,
    SupportedSetting, ValueEncoding, VersionedKey,
    setting_available_for_version,
};
use crate::state::{CanonicalValue, StoredOption};

const GRAPHICS_VALUES: &[&str] = &["fast", "fancy", "fabulous", "custom"];
const CLOUD_VALUES: &[&str] = &["false", "fast", "true"];
const MUSIC_TOAST_VALUES: &[&str] = &["never", "pause", "pause_and_toast"];

fn decode_string_token(raw: &str) -> Option<String> {
    if raw.starts_with('"') {
        serde_json::from_str(raw).ok()
    } else {
        Some(raw.to_string())
    }
}

fn encode_string_token(value: &str) -> Option<String> {
    serde_json::to_string(value).ok()
}

pub(in crate::api::instance) const GRAPHICS_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "fancyGraphics",
        since: "1.0",
        until: "1.15.2",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "graphicsMode",
        since: "1.16",
        until: "1.21.10",
        mapping: GameOptionMappingKind::Migrated,
    },
    VersionedKey {
        key: "graphicsPreset",
        since: "1.21.11",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const AMBIENT_OCCLUSION_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "ao",
        since: "1.0",
        until: "1.4.7",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "ao",
        since: "1.5.1",
        until: "1.19.2",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "ao",
        since: "1.19.3",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const FOV_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "fov",
        since: "1.0",
        until: "1.18.2",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "fov",
        since: "1.19",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const CLOUD_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "clouds",
        since: "1.0",
        until: "1.7.10",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "renderClouds",
        since: "1.8",
        until: "1.8",
        mapping: GameOptionMappingKind::Migrated,
    },
    VersionedKey {
        key: "renderClouds",
        since: "1.8.1",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const CHAT_PREVIEW_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "chatPreview",
        since: "1.19",
        until: "1.19",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "chatPreview",
        since: "1.19.1",
        until: "1.19.2",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const MENU_BACKGROUND_BLUR_KEYS:
    &[VersionedKey] = &[
    VersionedKey {
        key: "menuBackgroundBlurriness",
        since: "1.20.5",
        until: "1.20.6",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "menuBackgroundBlurriness",
        since: "1.21",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const MASTER_VOLUME_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "sound",
        since: "1.0",
        until: "1.6.4",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "soundCategory_master",
        since: "1.7.2",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const MUSIC_VOLUME_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "music",
        since: "1.0",
        until: "1.6.4",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "soundCategory_music",
        since: "1.7.2",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const CAPE_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "showCape",
        since: "1.4.2",
        until: "1.7.10",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "modelPart_cape",
        since: "1.8",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const MUSIC_TOAST_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "showNowPlayingToast",
        since: "1.21.6",
        until: "1.21.10",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "musicToast",
        since: "1.21.11",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) const SWAP_OFFHAND_KEYS: &[VersionedKey] = &[
    VersionedKey {
        key: "key_key.swapHands",
        since: "1.9",
        until: "1.15.2",
        mapping: GameOptionMappingKind::Legacy,
    },
    VersionedKey {
        key: "key_key.swapOffhand",
        since: "1.16",
        until: "26.3",
        mapping: GameOptionMappingKind::Direct,
    },
];

pub(in crate::api::instance) fn release_version(
    version: &str,
) -> Option<(u32, u32, u32)> {
    let components = version
        .split('.')
        .map(|component| component.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    match components.as_slice() {
        [major, minor] => Some((*major, *minor, 0)),
        [major, minor, patch] => Some((*major, *minor, *patch)),
        _ => None,
    }
}

pub(in crate::api::instance) fn physical_variant_for_version<'a>(
    definition: &'a SupportedSetting,
    version: &str,
) -> Option<&'a VersionedKey> {
    let version = release_version(version)?;
    definition.versioned_keys.iter().find(|variant| {
        let Some(since) = release_version(variant.since) else {
            return false;
        };
        let Some(until) = release_version(variant.until) else {
            return false;
        };
        version >= since && version <= until
    })
}

pub(in crate::api::instance) fn physical_variant_for_present_key<'a>(
    definition: &'a SupportedSetting,
    key: &str,
    version: &str,
) -> Option<&'a VersionedKey> {
    let version = release_version(version)?;
    definition.versioned_keys.iter().rev().find(|variant| {
        variant.key == key
            && release_version(variant.since)
                .is_some_and(|since| since <= version)
            && release_version(variant.until)
                .is_some_and(|until| version <= until)
    })
}

pub(in crate::api::instance) fn uses_modern_key_tokens(version: &str) -> bool {
    release_version(version).is_some_and(|version| version >= (1, 13, 0))
}

pub(in crate::api::instance) fn split_key_binding(
    value: &str,
) -> (&str, Option<&str>) {
    value
        .split_once(':')
        .map_or((value, None), |(key, modifier)| (key, Some(modifier)))
}

pub(in crate::api::instance) fn append_key_modifier(
    mut key: String,
    modifier: Option<&str>,
) -> String {
    if let Some(modifier) = modifier {
        key.push(':');
        key.push_str(modifier);
    }
    key
}

pub(in crate::api::instance) fn legacy_key_binding_to_token(
    value: &str,
) -> Option<String> {
    let (key, modifier) = split_key_binding(value);
    let key = key.parse::<i32>().ok()?;
    let token = match key {
        -100 => "key.mouse.left".to_string(),
        -99 => "key.mouse.right".to_string(),
        -98 => "key.mouse.middle".to_string(),
        -97..=-85 => format!("key.mouse.{}", key + 101),
        _ => LEGACY_KEY_BINDINGS
            .iter()
            .find(|(legacy, _)| *legacy == key)
            .map(|(_, token)| (*token).to_string())?,
    };
    Some(append_key_modifier(token, modifier))
}

pub(in crate::api::instance) fn token_key_binding_to_legacy(
    value: &str,
) -> Option<String> {
    let (token, modifier) = split_key_binding(value);
    let key = match token {
        "key.mouse.left" => -100,
        "key.mouse.right" => -99,
        "key.mouse.middle" => -98,
        _ => {
            if let Some(button) = token.strip_prefix("key.mouse.") {
                let button = button.parse::<i32>().ok()?;
                if !(4..=16).contains(&button) {
                    return None;
                }
                button - 101
            } else {
                LEGACY_KEY_BINDINGS
                    .iter()
                    .find(|(_, candidate)| *candidate == token)
                    .map(|(legacy, _)| *legacy)?
            }
        }
    };
    Some(append_key_modifier(key.to_string(), modifier))
}

pub(in crate::api::instance) fn valid_modern_key_binding(value: &str) -> bool {
    let (key, modifier) = split_key_binding(value);
    let key_valid = ["key.keyboard.", "key.mouse.", "scancode."]
        .into_iter()
        .find_map(|prefix| key.strip_prefix(prefix))
        .is_some_and(|identifier| {
            !identifier.is_empty()
                && identifier.chars().all(|character| {
                    character.is_ascii_alphanumeric()
                        || matches!(character, '.' | '_' | '-')
                })
        });
    key_valid
        && modifier.is_none_or(|modifier| {
            !modifier.is_empty()
                && modifier.len() <= 64
                && modifier.chars().all(|character| {
                    character.is_ascii_alphanumeric()
                        || matches!(character, '.' | '_' | '-' | ':' | '+')
                })
        })
}

pub(in crate::api::instance) fn supported_settings_cover_game_version(
    version: &str,
) -> bool {
    if version.contains('w') || version.contains("snapshot") {
        return false;
    }
    let components = version
        .split('.')
        .map(|component| component.parse::<u32>())
        .collect::<Result<Vec<_>, _>>();
    let Ok(components) = components else {
        return false;
    };
    match components.as_slice() {
        [1, minor] => *minor <= 21,
        [1, minor, patch] => *minor < 21 || (*minor == 21 && *patch <= 11),
        [26, minor] | [26, minor, _] => (1..=3).contains(minor),
        _ => false,
    }
}

pub(in crate::api::instance) fn decode_value(
    definition: &SupportedSetting,
    physical_key: &str,
    raw: &str,
) -> Option<CanonicalValue> {
    match definition.encoding {
        ValueEncoding::Bool => match raw {
            "true" => Some(CanonicalValue::Bool(true)),
            "false" => Some(CanonicalValue::Bool(false)),
            _ => None,
        },
        ValueEncoding::Integer => {
            raw.parse::<i64>().ok().map(CanonicalValue::Integer)
        }
        ValueEncoding::Decimal => raw
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|_| CanonicalValue::Decimal(raw.to_string())),
        ValueEncoding::Enum(choices) => {
            let value = decode_string_token(raw)?;
            choices
                .contains(&value.as_str())
                .then(|| CanonicalValue::Enum(value))
        }
        ValueEncoding::QuotedEnum(choices) => {
            let value = decode_string_token(raw)?;
            choices
                .contains(&value.as_str())
                .then(|| CanonicalValue::Enum(value))
        }
        ValueEncoding::Text => Some(CanonicalValue::Text(raw.to_string())),
        ValueEncoding::KeyBinding => {
            if valid_modern_key_binding(raw) {
                Some(CanonicalValue::KeyBinding(raw.to_string()))
            } else {
                legacy_key_binding_to_token(raw).map(CanonicalValue::KeyBinding)
            }
        }
        ValueEncoding::Fov => raw.parse::<f64>().ok().and_then(|value| {
            if value.fract() == 0.0 && (30.0..=110.0).contains(&value) {
                return Some(CanonicalValue::Integer(value as i64));
            }
            let degrees = value * 40.0 + 70.0;
            (value.is_finite()
                && (-1.0..=1.0).contains(&value)
                && (degrees - degrees.round()).abs() <= 0.000_001)
                .then(|| CanonicalValue::Integer(degrees.round() as i64))
        }),
        ValueEncoding::GuiScale => {
            raw.parse::<i64>().ok().map(CanonicalValue::Integer)
        }
        ValueEncoding::MenuBackgroundBlur => raw
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && value.fract() == 0.0)
            .map(|value| CanonicalValue::Integer(value as i64)),
        ValueEncoding::Graphics => {
            let raw = decode_string_token(raw)?;
            let value = match physical_key {
                "fancyGraphics" => match raw.as_str() {
                    "false" => "fast",
                    "true" => "fancy",
                    _ => return None,
                },
                "graphicsMode" => match raw.as_str() {
                    "0" => "fast",
                    "1" => "fancy",
                    "2" => "fabulous",
                    _ => return None,
                },
                "graphicsPreset" if GRAPHICS_VALUES.contains(&raw.as_str()) => {
                    raw.as_str()
                }
                _ => return None,
            };
            Some(CanonicalValue::Enum(value.to_string()))
        }
        ValueEncoding::AmbientOcclusion => {
            let value = match raw {
                "false" | "0" => "off",
                "true" => "on",
                "1" => "minimum",
                "2" => "maximum",
                _ => return None,
            };
            Some(CanonicalValue::Enum(value.to_string()))
        }
        ValueEncoding::ChatPreview => {
            let value = match raw {
                "false" | "0" => "0",
                "1" => "1",
                "true" | "2" => "2",
                _ => return None,
            };
            Some(CanonicalValue::Enum(value.to_string()))
        }
        ValueEncoding::Clouds => {
            let raw = decode_string_token(raw)?;
            CLOUD_VALUES
                .contains(&raw.as_str())
                .then(|| CanonicalValue::Enum(raw))
        }
        ValueEncoding::MusicToast => {
            let raw = decode_string_token(raw)?;
            let value = match (physical_key, raw.as_str()) {
                ("showNowPlayingToast", "false") => "never",
                ("showNowPlayingToast", "true") => "pause_and_toast",
                ("musicToast", "never" | "pause" | "pause_and_toast") => {
                    raw.as_str()
                }
                _ => return None,
            };
            Some(CanonicalValue::Enum(value.to_string()))
        }
    }
}

pub(in crate::api::instance) fn encode_value(
    definition: &SupportedSetting,
    physical_key: &str,
    value: &CanonicalValue,
    game_version: &str,
    current_raw: Option<&str>,
) -> Option<String> {
    if !setting_available_for_version(definition, game_version) {
        return None;
    }
    if let Some(current_raw) = current_raw
        && physical_representation_supported_for_target(
            definition,
            physical_key,
            current_raw,
            game_version,
        )
        && decode_value(definition, physical_key, current_raw)
            .as_ref()
            .is_some_and(|current| canonical_values_equal(current, value))
    {
        return Some(current_raw.to_string());
    }
    match (definition.encoding, value) {
        (ValueEncoding::Bool, CanonicalValue::Bool(value)) => {
            Some(value.to_string())
        }
        (ValueEncoding::Integer, CanonicalValue::Integer(value)) => {
            Some(value.to_string())
        }
        (ValueEncoding::Decimal, CanonicalValue::Decimal(value)) => {
            Some(value.clone())
        }
        (ValueEncoding::Decimal, CanonicalValue::Integer(value)) => {
            Some(value.to_string())
        }
        (ValueEncoding::Enum(choices), CanonicalValue::Enum(value))
            if choices.contains(&value.as_str()) =>
        {
            if current_raw.is_some_and(|raw| raw.starts_with('"')) {
                encode_string_token(value)
            } else {
                Some(value.clone())
            }
        }
        (ValueEncoding::QuotedEnum(choices), CanonicalValue::Enum(value))
            if choices.contains(&value.as_str()) =>
        {
            encode_string_token(value)
        }
        (ValueEncoding::Text, CanonicalValue::Text(value)) => {
            Some(value.clone())
        }
        (ValueEncoding::KeyBinding, CanonicalValue::KeyBinding(value))
            if valid_modern_key_binding(value) =>
        {
            if !uses_modern_key_tokens(game_version) {
                token_key_binding_to_legacy(value)
            } else {
                Some(value.clone())
            }
        }
        (ValueEncoding::Fov, CanonicalValue::Integer(value))
            if (30..=110).contains(value) =>
        {
            if release_version(game_version)
                .is_some_and(|version| version >= (1, 19, 0))
            {
                Some(value.to_string())
            } else {
                let normalized = (*value as f64 - 70.0) / 40.0;
                Some(format_decimal(normalized))
            }
        }
        (ValueEncoding::GuiScale, CanonicalValue::Integer(value))
            if (0..=8).contains(value)
                && (release_version(game_version)
                    .is_some_and(|version| version >= (1, 13, 0))
                    || *value <= 3) =>
        {
            Some(value.to_string())
        }
        (ValueEncoding::MenuBackgroundBlur, CanonicalValue::Integer(value))
            if (0..=10).contains(value) =>
        {
            if release_version(game_version)
                .is_some_and(|version| version >= (1, 21, 0))
            {
                Some(value.to_string())
            } else {
                Some(format!("{value}.0"))
            }
        }
        (ValueEncoding::Graphics, CanonicalValue::Enum(value)) => {
            match physical_key {
                "fancyGraphics" => match value.as_str() {
                    "fast" => Some("false".to_string()),
                    "fancy" => Some("true".to_string()),
                    _ => None,
                },
                "graphicsMode" => match value.as_str() {
                    "fast" => Some("0".to_string()),
                    "fancy" => Some("1".to_string()),
                    "fabulous" => Some("2".to_string()),
                    _ => None,
                },
                "graphicsPreset"
                    if GRAPHICS_VALUES.contains(&value.as_str()) =>
                {
                    encode_string_token(value)
                }
                _ => None,
            }
        }
        (ValueEncoding::AmbientOcclusion, CanonicalValue::Enum(value)) => {
            let version = release_version(game_version)?;
            if ((1, 5, 1)..=(1, 19, 2)).contains(&version) {
                match value.as_str() {
                    "off" => Some("0".to_string()),
                    "minimum" => Some("1".to_string()),
                    "maximum" => Some("2".to_string()),
                    _ => None,
                }
            } else {
                match value.as_str() {
                    "off" => Some("false".to_string()),
                    "on" => Some("true".to_string()),
                    _ => None,
                }
            }
        }
        (ValueEncoding::ChatPreview, CanonicalValue::Enum(value)) => {
            if release_version(game_version) == Some((1, 19, 0)) {
                match value.as_str() {
                    "0" => Some("false".to_string()),
                    "2" => Some("true".to_string()),
                    _ => None,
                }
            } else if matches!(value.as_str(), "0" | "1" | "2") {
                Some(value.clone())
            } else {
                None
            }
        }
        (ValueEncoding::Clouds, CanonicalValue::Enum(value)) => {
            let version = release_version(game_version)?;
            if version <= (1, 8, 0) {
                match value.as_str() {
                    "false" | "true" => Some(value.clone()),
                    _ => None,
                }
            } else if CLOUD_VALUES.contains(&value.as_str()) {
                Some(value.clone())
            } else {
                None
            }
        }
        (ValueEncoding::MusicToast, CanonicalValue::Enum(value)) => {
            match physical_key {
                "showNowPlayingToast" => match value.as_str() {
                    "never" => Some("false".to_string()),
                    "pause_and_toast" => Some("true".to_string()),
                    _ => None,
                },
                "musicToast"
                    if MUSIC_TOAST_VALUES.contains(&value.as_str()) =>
                {
                    encode_string_token(value)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn validate_value(
    definition: Option<&SupportedSetting>,
    value: &CanonicalValue,
    enforce_editor_increment: bool,
) -> crate::Result<()> {
    if let Some(definition) = definition {
        let type_valid = match definition.encoding {
            ValueEncoding::Bool => matches!(value, CanonicalValue::Bool(_)),
            ValueEncoding::Integer
            | ValueEncoding::GuiScale
            | ValueEncoding::MenuBackgroundBlur => {
                matches!(value, CanonicalValue::Integer(_))
            }
            ValueEncoding::Decimal => matches!(
                value,
                CanonicalValue::Decimal(_) | CanonicalValue::Integer(_)
            ),
            ValueEncoding::Enum(_)
            | ValueEncoding::QuotedEnum(_)
            | ValueEncoding::Graphics
            | ValueEncoding::AmbientOcclusion
            | ValueEncoding::ChatPreview
            | ValueEncoding::Clouds
            | ValueEncoding::MusicToast => {
                matches!(value, CanonicalValue::Enum(_))
            }
            ValueEncoding::Text => matches!(value, CanonicalValue::Text(_)),
            ValueEncoding::Fov => matches!(value, CanonicalValue::Integer(_)),
            ValueEncoding::KeyBinding => {
                matches!(value, CanonicalValue::KeyBinding(_))
            }
        };
        if !type_valid {
            return Err(input_error(format!(
                "Invalid value type for {}",
                definition.id
            )));
        }
        match (definition.editor, value) {
            (
                SettingEditor::UnboundedDecimal,
                CanonicalValue::Decimal(value),
            ) if value
                .parse::<f64>()
                .ok()
                .is_none_or(|value| !value.is_finite()) =>
            {
                return Err(input_error("Invalid decimal value"));
            }
            (
                SettingEditor::Integer { min, max, step },
                CanonicalValue::Integer(value),
            ) if *value < min
                || *value > max
                || enforce_editor_increment
                    && step > 0
                    && (*value - min) % step != 0 =>
            {
                return Err(input_error(format!(
                    "{} is outside its supported range or increment",
                    definition.id
                )));
            }
            (
                SettingEditor::Decimal { min, max, step, .. },
                CanonicalValue::Decimal(value),
            ) => {
                let parsed = value
                    .parse::<f64>()
                    .map_err(|_| input_error("Invalid decimal value"))?;
                let increment = (parsed - min) / step;
                if !parsed.is_finite()
                    || parsed < min
                    || parsed > max
                    || enforce_editor_increment
                        && step > 0.0
                        && (increment - increment.round()).abs() > 0.000_001
                {
                    return Err(input_error(format!(
                        "{} is outside its supported range or increment",
                        definition.id
                    )));
                }
            }
            (
                SettingEditor::Decimal { min, max, step, .. },
                CanonicalValue::Integer(value),
            ) => {
                let parsed = *value as f64;
                let increment = (parsed - min) / step;
                if parsed < min
                    || parsed > max
                    || enforce_editor_increment
                        && step > 0.0
                        && (increment - increment.round()).abs() > 0.000_001
                {
                    return Err(input_error(format!(
                        "{} is outside its supported range or increment",
                        definition.id
                    )));
                }
            }
            (SettingEditor::Enum(choices), CanonicalValue::Enum(value))
                if !choices.iter().any(|choice| *choice == value.as_str()) =>
            {
                return Err(input_error(format!(
                    "Invalid choice for {}",
                    definition.id
                )));
            }
            (SettingEditor::Language, CanonicalValue::Text(value))
                if value.is_empty()
                    || value.len() > 64
                    || value.chars().any(char::is_control) =>
            {
                return Err(input_error("Invalid language value"));
            }
            (SettingEditor::KeyBinding, CanonicalValue::KeyBinding(value))
                if value.len() > 256
                    || value.contains(['\r', '\n'])
                    || !valid_modern_key_binding(value) =>
            {
                return Err(input_error("Invalid key binding value"));
            }
            _ => {}
        }
    } else if let CanonicalValue::ExternalRaw(raw) = value {
        validate_raw_key_value("external", raw)?;
    } else {
        return Err(input_error("Custom settings require a raw string value"));
    }
    Ok(())
}

pub(in crate::api::instance) fn validate_canonical_value(
    definition: Option<&SupportedSetting>,
    value: &CanonicalValue,
) -> crate::Result<()> {
    validate_value(definition, value, true)
}

pub(in crate::api::instance) fn validate_file_value(
    definition: Option<&SupportedSetting>,
    value: &CanonicalValue,
) -> crate::Result<()> {
    validate_value(definition, value, false)
}

pub(in crate::api::instance) fn decode_value_for_version(
    definition: &SupportedSetting,
    physical_key: &str,
    raw: &str,
    game_version: &str,
) -> Option<CanonicalValue> {
    if !setting_available_for_version(definition, game_version) {
        return None;
    }
    let value = decode_value(definition, physical_key, raw)?;
    validate_file_value(Some(definition), &value).ok()?;
    let target_key = if definition.versioned_keys.is_empty() {
        physical_key
    } else {
        physical_variant_for_version(definition, game_version)?.key
    };
    encode_value(definition, target_key, &value, game_version, None)?;
    Some(value)
}

pub(in crate::api::instance) fn target_physical_key(
    definition: Option<&SupportedSetting>,
    stored: &StoredOption,
    document: &GameOptionsDocument,
    game_version: &str,
) -> Option<String> {
    if let Some(definition) = definition {
        if !setting_available_for_version(definition, game_version) {
            return None;
        }
        if !definition.versioned_keys.is_empty() {
            observed_physical_key(definition, document, game_version)?;
            return physical_variant_for_version(definition, game_version)
                .map(|variant| variant.key.to_string());
        }
        definition
            .keys
            .iter()
            .find(|key| document.value(key).is_some())
            .map(|key| (*key).to_string())
    } else {
        stored.raw_key.clone()
    }
}

pub(in crate::api::instance) fn observed_physical_key(
    definition: &SupportedSetting,
    document: &GameOptionsDocument,
    game_version: &str,
) -> Option<String> {
    if !setting_available_for_version(definition, game_version) {
        return None;
    }
    if let Some(target) = physical_variant_for_version(definition, game_version)
        && document.value(target.key).is_some()
    {
        return Some(target.key.to_string());
    }
    definition
        .keys
        .iter()
        .find(|key| document.value(key).is_some())
        .map(|key| (*key).to_string())
}

pub(in crate::api::instance) fn alias_migration_needed(
    definition: &SupportedSetting,
    document: &GameOptionsDocument,
    target_key: &str,
) -> bool {
    document.value(target_key).is_none()
        && definition
            .keys
            .iter()
            .any(|key| *key != target_key && document.value(key).is_some())
}

pub(in crate::api::instance) fn physical_representation_supported_for_target(
    definition: &SupportedSetting,
    physical_key: &str,
    raw: &str,
    game_version: &str,
) -> bool {
    if !setting_available_for_version(definition, game_version) {
        return false;
    }
    let Some(target_version) = release_version(game_version) else {
        return false;
    };
    if matches!(definition.encoding, ValueEncoding::KeyBinding) {
        return if target_version >= (1, 13, 0) {
            valid_modern_key_binding(raw)
        } else {
            legacy_key_binding_to_token(raw).is_some()
        };
    }
    if matches!(definition.encoding, ValueEncoding::AmbientOcclusion) {
        return if ((1, 5, 1)..=(1, 19, 2)).contains(&target_version) {
            matches!(raw, "0" | "1" | "2")
        } else {
            matches!(raw, "true" | "false")
        };
    }
    if matches!(definition.encoding, ValueEncoding::Fov) {
        return if target_version >= (1, 19, 0) {
            raw.parse::<i64>()
                .is_ok_and(|value| (30..=110).contains(&value))
        } else {
            raw.parse::<f64>().is_ok_and(|value| {
                value.is_finite() && (-1.0..=1.0).contains(&value)
            })
        };
    }
    if matches!(definition.encoding, ValueEncoding::ChatPreview) {
        return if target_version == (1, 19, 0) {
            matches!(raw, "true" | "false")
        } else {
            matches!(raw, "0" | "1" | "2")
        };
    }
    if matches!(definition.encoding, ValueEncoding::GuiScale) {
        return raw.parse::<i64>().is_ok_and(|value| {
            if target_version >= (1, 13, 0) {
                (0..=8).contains(&value)
            } else {
                (0..=3).contains(&value)
            }
        });
    }
    if matches!(definition.encoding, ValueEncoding::MenuBackgroundBlur) {
        return if target_version >= (1, 21, 0) {
            raw.parse::<i64>()
                .is_ok_and(|value| (0..=10).contains(&value))
        } else {
            raw.parse::<f64>().is_ok_and(|value| {
                value.is_finite()
                    && value.fract() == 0.0
                    && (0.0..=10.0).contains(&value)
            })
        };
    }
    if matches!(definition.encoding, ValueEncoding::Clouds) {
        return if target_version <= (1, 8, 0) {
            matches!(raw, "true" | "false")
        } else {
            decode_string_token(raw)
                .is_some_and(|value| CLOUD_VALUES.contains(&value.as_str()))
        };
    }

    let mut variants = definition
        .versioned_keys
        .iter()
        .filter(|variant| variant.key == physical_key)
        .peekable();
    if variants.peek().is_none() {
        return true;
    }
    variants.any(|variant| {
        release_version(variant.since)
            .is_some_and(|since| since <= target_version)
            && release_version(variant.until)
                .is_some_and(|until| target_version <= until)
    })
}

pub(in crate::api::instance) fn format_decimal(value: f64) -> String {
    let value = format!("{value:.8}");
    let value = value.trim_end_matches('0').trim_end_matches('.');
    if value == "-0" {
        "0".to_string()
    } else {
        value.to_string()
    }
}
