use super::{BOOL, SettingEditor, SupportedSetting, ValueEncoding, setting};

const NARRATOR: &[&str] = &["0", "1", "2", "3"];

pub(super) const SETTINGS: &[SupportedSetting] = &[
    setting(
        "narrator",
        &["narrator"],
        "accessibility",
        true,
        SettingEditor::Enum(NARRATOR),
        ValueEncoding::Enum(NARRATOR),
    ),
    setting(
        "subtitles",
        &["showSubtitles"],
        "accessibility",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "high_contrast",
        &["highContrast"],
        "accessibility",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "dark_splash",
        &["darkMojangStudiosBackground"],
        "accessibility",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "notification_time",
        &["notificationDisplayTime"],
        "accessibility",
        true,
        SettingEditor::Decimal {
            min: 0.5,
            max: 10.0,
            step: 0.5,
            unit: Some("multiplier"),
        },
        ValueEncoding::Decimal,
    ),
];
