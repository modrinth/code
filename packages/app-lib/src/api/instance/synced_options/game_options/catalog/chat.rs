use super::{
    BOOL, SettingEditor, SupportedSetting, UNIT_INTERVAL, ValueEncoding,
    setting,
};

const VISIBILITY: &[&str] = &["0", "1", "2"];

pub(super) const SETTINGS: &[SupportedSetting] = &[
    setting(
        "chat_visibility",
        &["chatVisibility"],
        "chat",
        true,
        SettingEditor::Enum(VISIBILITY),
        ValueEncoding::Enum(VISIBILITY),
    ),
    setting(
        "chat_colors",
        &["chatColors"],
        "chat",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "chat_links",
        &["chatLinks"],
        "chat",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "chat_links_prompt",
        &["chatLinksPrompt"],
        "chat",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "chat_opacity",
        &["chatOpacity"],
        "chat",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
    setting(
        "chat_scale",
        &["chatScale"],
        "chat",
        true,
        UNIT_INTERVAL,
        ValueEncoding::Decimal,
    ),
];
