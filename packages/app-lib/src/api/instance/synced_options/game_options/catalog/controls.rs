use super::{
    BOOL, SettingEditor, SupportedSetting, ValueEncoding, ranged_setting,
    setting, version_changes, versioned_setting,
};

macro_rules! key_setting {
    ($id:literal, $key:literal, $since:literal, $until:literal) => {
        ranged_setting(
            $id,
            &[$key],
            $since,
            $until,
            "controls",
            true,
            SettingEditor::KeyBinding,
            ValueEncoding::KeyBinding,
        )
    };
}

pub(super) const SETTINGS: &[SupportedSetting] = &[
    setting(
        "sensitivity",
        &["mouseSensitivity"],
        "controls",
        true,
        SettingEditor::Decimal {
            min: 0.0,
            max: 1.0,
            step: 0.005,
            unit: Some("percent"),
        },
        ValueEncoding::Decimal,
    ),
    setting(
        "invert_mouse",
        &["invertYMouse"],
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "invert_horizontal_mouse",
        &["invertXMouse"],
        "1.21.9",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "auto_jump",
        &["autoJump"],
        "1.10",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "toggle_crouch",
        &["toggleCrouch"],
        "1.15",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "toggle_sprint",
        &["toggleSprint"],
        "1.15",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "toggle_attack",
        &["toggleAttack"],
        "1.21.9",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "toggle_use",
        &["toggleUse"],
        "1.21.9",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "discrete_mouse_scroll",
        &["discrete_mouse_scroll"],
        "1.14",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "mouse_wheel_sensitivity",
        &["mouseWheelSensitivity"],
        "1.13",
        "26.3",
        "controls",
        true,
        SettingEditor::UnboundedDecimal,
        ValueEncoding::Decimal,
    ),
    ranged_setting(
        "raw_mouse_input",
        &["rawMouseInput"],
        "1.14.4",
        "26.2",
        "controls",
        false,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "touchscreen",
        &["touchscreen"],
        "1.4.4",
        "26.1.2",
        "controls",
        false,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "allow_cursor_changes",
        &["allowCursorChanges"],
        "1.21.9",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "sprint_window",
        &["sprintWindow"],
        "1.21.9",
        "26.3",
        "controls",
        true,
        SettingEditor::UnboundedInteger,
        ValueEncoding::Integer,
    ),
    ranged_setting(
        "operator_items_tab",
        &["operatorItemsTab"],
        "1.19.3",
        "26.3",
        "controls",
        true,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "ctrl_click_right_click",
        &["ctrlClickEmulatesRightClick"],
        "26.3",
        "26.3",
        "controls",
        false,
        BOOL,
        ValueEncoding::Bool,
    ),
    ranged_setting(
        "quit_shortcuts",
        &["quitShortcuts"],
        "26.3",
        "26.3",
        "controls",
        false,
        BOOL,
        ValueEncoding::Bool,
    ),
    key_setting!("key.forward", "key_key.forward", "1.0", "26.3"),
    key_setting!("key.left", "key_key.left", "1.0", "26.3"),
    key_setting!("key.back", "key_key.back", "1.0", "26.3"),
    key_setting!("key.right", "key_key.right", "1.0", "26.3"),
    key_setting!("key.jump", "key_key.jump", "1.0", "26.3"),
    key_setting!("key.sneak", "key_key.sneak", "1.0", "26.3"),
    key_setting!("key.sprint", "key_key.sprint", "1.7.2", "26.3"),
    key_setting!("key.inventory", "key_key.inventory", "1.0", "26.3"),
    versioned_setting(
        "key.swap_offhand",
        &["key_key.swapOffhand", "key_key.swapHands"],
        version_changes::SWAP_OFFHAND_KEYS,
        "controls",
        true,
        SettingEditor::KeyBinding,
        ValueEncoding::KeyBinding,
    ),
    key_setting!("key.drop", "key_key.drop", "1.0", "26.3"),
    key_setting!("key.use", "key_key.use", "1.0", "26.3"),
    key_setting!("key.attack", "key_key.attack", "1.0", "26.3"),
    key_setting!("key.pick_item", "key_key.pickItem", "1.0", "26.3"),
    key_setting!("key.chat", "key_key.chat", "1.0", "26.3"),
    key_setting!("key.player_list", "key_key.playerlist", "1.0", "26.3"),
    key_setting!("key.command", "key_key.command", "1.3.1", "26.3"),
    key_setting!("key.screenshot", "key_key.screenshot", "1.7.2", "26.3"),
    key_setting!(
        "key.perspective",
        "key_key.togglePerspective",
        "1.7.2",
        "26.3"
    ),
    key_setting!("key.fullscreen", "key_key.fullscreen", "1.7.3", "26.3"),
    key_setting!("key.advancements", "key_key.advancements", "1.12", "26.3"),
    key_setting!("key.smooth_camera", "key_key.smoothCamera", "1.7.2", "26.3"),
    key_setting!(
        "key.spectator_outlines",
        "key_key.spectatorOutlines",
        "1.8",
        "26.3"
    ),
    key_setting!(
        "key.save_toolbar",
        "key_key.saveToolbarActivator",
        "1.12",
        "26.3"
    ),
    key_setting!(
        "key.load_toolbar",
        "key_key.loadToolbarActivator",
        "1.12",
        "26.3"
    ),
    key_setting!(
        "key.social_interactions",
        "key_key.socialInteractions",
        "1.16.4",
        "26.3"
    ),
    key_setting!(
        "key.quick_actions",
        "key_key.quickActions",
        "1.21.6",
        "26.3"
    ),
    key_setting!(
        "key.spectator_hotbar",
        "key_key.spectatorHotbar",
        "1.21.9",
        "26.3"
    ),
    key_setting!("key.friends", "key_key.friends", "26.2", "26.3"),
    key_setting!("key.toggle_gui", "key_key.toggleGui", "1.21.11", "26.3"),
    key_setting!(
        "key.toggle_spectator_shader",
        "key_key.toggleSpectatorShaderEffects",
        "1.21.11",
        "26.3"
    ),
    key_setting!("key.hotbar.1", "key_key.hotbar.1", "1.7.2", "26.3"),
    key_setting!("key.hotbar.2", "key_key.hotbar.2", "1.7.2", "26.3"),
    key_setting!("key.hotbar.3", "key_key.hotbar.3", "1.7.2", "26.3"),
    key_setting!("key.hotbar.4", "key_key.hotbar.4", "1.7.2", "26.3"),
    key_setting!("key.hotbar.5", "key_key.hotbar.5", "1.7.2", "26.3"),
    key_setting!("key.hotbar.6", "key_key.hotbar.6", "1.7.2", "26.3"),
    key_setting!("key.hotbar.7", "key_key.hotbar.7", "1.7.2", "26.3"),
    key_setting!("key.hotbar.8", "key_key.hotbar.8", "1.7.2", "26.3"),
    key_setting!("key.hotbar.9", "key_key.hotbar.9", "1.7.2", "26.3"),
    key_setting!(
        "key.debug.overlay",
        "key_key.debug.overlay",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.modifier",
        "key_key.debug.modifier",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.reload_chunks",
        "key_key.debug.reloadChunk",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.hitboxes",
        "key_key.debug.showHitboxes",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.clear_chat",
        "key_key.debug.clearChat",
        "1.21.11",
        "26.3"
    ),
    key_setting!("key.debug.crash", "key_key.debug.crash", "1.21.11", "26.3"),
    key_setting!(
        "key.debug.chunk_borders",
        "key_key.debug.showChunkBorders",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.advanced_tooltips",
        "key_key.debug.showAdvancedTooltips",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.copy_recreate_command",
        "key_key.debug.copyRecreateCommand",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.spectate",
        "key_key.debug.spectate",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.switch_game_mode",
        "key_key.debug.switchGameMode",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.options",
        "key_key.debug.debugOptions",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.focus_pause",
        "key_key.debug.focusPause",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.dump_dynamic_textures",
        "key_key.debug.dumpDynamicTextures",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.reload_resource_packs",
        "key_key.debug.reloadResourcePacks",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.profiling",
        "key_key.debug.profiling",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.copy_location",
        "key_key.debug.copyLocation",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.dump_version",
        "key_key.debug.dumpVersion",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.profiling_chart",
        "key_key.debug.profilingChart",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.fps_charts",
        "key_key.debug.fpsCharts",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.network_charts",
        "key_key.debug.networkCharts",
        "1.21.11",
        "26.3"
    ),
    key_setting!(
        "key.debug.lightmap_texture",
        "key_key.debug.lightmapTexture",
        "26.1",
        "26.3"
    ),
    key_setting!(
        "key.debug.improved_transparency",
        "key_key.debug.improvedTransparency",
        "26.3",
        "26.3"
    ),
];
