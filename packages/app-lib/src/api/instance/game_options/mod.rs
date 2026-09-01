//! Syncs selected values from Minecraft's `options.txt` between instances.
//!
//! When sync is first enabled, the user chooses an instance to supply the initial
//! values. Each instance keeps its own `options.txt`; we update only the selected
//! settings and leave modpack defaults, comments, and disabled settings alone.
//!
//! Before writing a file, we compare it with what Modrinth wrote last time. This
//! lets changes made in Minecraft become the new shared values. When multiple
//! instances change the same setting, the latest change processed by sync wins.
//!
//! Minecraft has renamed and reformatted settings over time. `supported_settings`
//! contains the file keys and conversions for the versions we support. Any other
//! key is treated as a custom setting and copied as-is to every synced instance.
//! Custom settings can be turned off individually in the settings screen.
//!
//! Launcher-enforced settings, such as fullscreen, are applied last and are not
//! mistaken for changes made by the player.

mod api_types;
mod fullscreen;
mod instance_support;
mod launch_overrides;
mod options_file;
mod pack_updates;
mod read_instance_changes;
mod settings_editor;
mod source_selection;
mod supported_settings;
mod write_shared_settings;

pub use crate::state::{CanonicalValue, GameOptionKind};
pub use api_types::{
    EditableGameSetting, GameOptionCompatibility,
    GameOptionCompatibilityBucket, GameOptionCompatibilityReason,
    GameOptionCompatibilityStatus, GameOptionEditorChoice,
    GameOptionEditorDefinition, GameOptionMappingKind,
    GameOptionValidationIssue, GameOptionValueState,
    GameOptionsSourceCandidate, GameOptionsSourceIssue, GameSettingCategory,
    GameSettingChange, GameSettingsEditorState, SaveGameSettingsResult,
    UpdateGameSettingsRequest,
};
pub use launch_overrides::{apply_launcher_overrides, sync_before_launch};
pub use pack_updates::capture_pack_base;
pub use settings_editor::{get_config, preview_changes, save_changes};
pub use source_selection::list_sync_sources;

pub(crate) use fullscreen::{
    shared_fullscreen_value, update_shared_fullscreen_from_app,
};
pub(crate) use write_shared_settings::sync_all_participating_instances;

pub(in crate::api::instance) use api_types::SyncReason;
pub(in crate::api::instance) use pack_updates::{
    detach_instance, prepare_instance_update_with_state,
};
pub(in crate::api::instance) use source_selection::initialize_from_source_instance;
pub(in crate::api::instance) use write_shared_settings::sync_instance_with_state;

pub(in crate::api::instance) async fn canonical_exists(
    state: &crate::state::State,
) -> crate::Result<bool> {
    crate::state::shared_game_options_exist(&state.pool).await
}

const OPTIONS_FILE: &str = "options.txt";
const CATALOG_REVISION: u32 = 3;
const MAX_OPTIONS_BYTES: usize = 2 * 1024 * 1024;
const MAX_OPTIONS_LINES: usize = 16_384;
const MAX_KEY_BYTES: usize = 1_024;
const MAX_VALUE_BYTES: usize = 32 * 1024;
