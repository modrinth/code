use super::{
    BOOL, SettingEditor, SupportedSetting, ValueEncoding, setting,
    version_changes, versioned_setting,
};

const GRAPHICS: &[&str] = &["fast", "fancy", "fabulous", "custom"];
const AMBIENT_OCCLUSION: &[&str] = &["off", "on", "minimum", "maximum"];
const PARTICLES: &[&str] = &["0", "1", "2"];
const CLOUDS: &[&str] = &["false", "fast", "true"];

pub(super) const SETTINGS: &[SupportedSetting] = &[
    setting(
        "fov",
        &["fov"],
        "video",
        true,
        SettingEditor::Integer {
            min: 30,
            max: 110,
            step: 1,
        },
        ValueEncoding::Fov,
    ),
    versioned_setting(
        "graphics",
        &["graphicsPreset", "graphicsMode", "fancyGraphics"],
        version_changes::GRAPHICS_KEYS,
        "video",
        true,
        SettingEditor::Enum(GRAPHICS),
        ValueEncoding::Graphics,
    ),
    versioned_setting(
        "ambient_occlusion",
        &["ao"],
        version_changes::AMBIENT_OCCLUSION_KEYS,
        "video",
        true,
        SettingEditor::Enum(AMBIENT_OCCLUSION),
        ValueEncoding::AmbientOcclusion,
    ),
    setting(
        "render_distance",
        &["renderDistance"],
        "video",
        false,
        SettingEditor::Integer {
            min: 2,
            max: 64,
            step: 1,
        },
        ValueEncoding::Integer,
    ),
    setting(
        "simulation_distance",
        &["simulationDistance"],
        "video",
        false,
        SettingEditor::Integer {
            min: 2,
            max: 64,
            step: 1,
        },
        ValueEncoding::Integer,
    ),
    setting(
        "gui_scale",
        &["guiScale"],
        "video",
        true,
        SettingEditor::Integer {
            min: 0,
            max: 8,
            step: 1,
        },
        ValueEncoding::Integer,
    ),
    setting(
        "particles",
        &["particles"],
        "video",
        true,
        SettingEditor::Enum(PARTICLES),
        ValueEncoding::Enum(PARTICLES),
    ),
    setting(
        "clouds",
        &["renderClouds"],
        "video",
        true,
        SettingEditor::Enum(CLOUDS),
        ValueEncoding::Enum(CLOUDS),
    ),
    setting(
        "entity_shadows",
        &["entityShadows"],
        "video",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "view_bobbing",
        &["bobView"],
        "video",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "vsync",
        &["enableVsync"],
        "video",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "fullscreen",
        &["fullscreen"],
        "video",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    setting(
        "max_framerate",
        &["maxFps"],
        "video",
        true,
        SettingEditor::Integer {
            min: 10,
            max: 260,
            step: 10,
        },
        ValueEncoding::Integer,
    ),
    setting(
        "mipmap_levels",
        &["mipmapLevels"],
        "video",
        false,
        SettingEditor::Integer {
            min: 0,
            max: 4,
            step: 1,
        },
        ValueEncoding::Integer,
    ),
    setting(
        "biome_blend_radius",
        &["biomeBlendRadius"],
        "video",
        true,
        SettingEditor::Integer {
            min: 0,
            max: 7,
            step: 1,
        },
        ValueEncoding::Integer,
    ),
];
