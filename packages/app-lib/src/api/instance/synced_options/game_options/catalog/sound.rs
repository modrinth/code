use super::{
    SettingEditor, SupportedSetting, UNIT_INTERVAL, ValueEncoding, setting,
    version_changes, versioned_setting,
};

const MUSIC_TOAST: &[&str] = &["never", "pause", "pause_and_toast"];

pub(super) const SETTINGS: &[SupportedSetting] = &[
    setting(
        "language",
        &["lang"],
        "language",
        true,
        SettingEditor::Language,
        ValueEncoding::Text,
    ),
    setting(
        "master_volume",
        &["soundCategory_master"],
        "music_and_sound",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
    setting(
        "music_volume",
        &["soundCategory_music"],
        "music_and_sound",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
    versioned_setting(
        "music_toast",
        &["musicToast", "showNowPlayingToast"],
        version_changes::MUSIC_TOAST_KEYS,
        "music_and_sound",
        true,
        SettingEditor::Enum(MUSIC_TOAST),
        ValueEncoding::MusicToast,
    ),
    setting(
        "blocks_volume",
        &["soundCategory_block"],
        "music_and_sound",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
    setting(
        "hostile_volume",
        &["soundCategory_hostile"],
        "music_and_sound",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
    setting(
        "players_volume",
        &["soundCategory_player"],
        "music_and_sound",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
    setting(
        "voice_volume",
        &["soundCategory_voice"],
        "music_and_sound",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
];
