//! Syncs selected values from Minecraft's `options.txt` between instances.
//!
//! When sync is first enabled, the user chooses an instance to supply the initial
//! values. Each instance keeps its own `options.txt`; we update only the selected
//! settings and leave modpack defaults, comments, and disabled settings alone.
//!
//! The watcher imports enabled values whenever `options.txt` changes. After the
//! app writes the file, it stores the file hash and ignores the matching watcher
//! event. On conflict, the latest save is persisted.
//!
//! Minecraft has renamed and reformatted settings over time. `catalog`
//! contains the file keys and conversions for the versions we support. Any other
//! key is treated as a custom setting and copied as-is to every synced instance.
//! Custom settings can be turned off individually in the settings screen.
//!
//! `catalog` will need to be checked every time a Minecraft update comes out. For maintenance reasons we will just
//! do this for major releases, no snapshots/pre-releases.
//!
//! Launcher-enforced settings, such as fullscreen, are applied last and excluded
//! when importing `options.txt`.

mod api_types;
mod catalog;
mod fullscreen;
mod instance_support;
mod launch_overrides;
mod local_settings_editor;
mod options_file;
mod pack_updates;
mod read_instance_changes;
mod resource_pack_options;
mod settings_editor;
mod source_selection;
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
pub use local_settings_editor::{
    get_config as get_local_config, preview_changes as preview_local_changes,
    save_changes as save_local_changes,
};
pub use pack_updates::{GameOptionsPackSource, capture_pack_base};
pub use settings_editor::{get_config, preview_changes, save_changes};
pub use source_selection::list_sync_sources;

pub(crate) use fullscreen::{
    shared_fullscreen_value, update_shared_fullscreen_from_app,
};
pub(crate) use write_shared_settings::sync_all_participating_instances;

pub(in crate::api::instance) use resource_pack_options::{
    ResourcePackOptionsUpdate, merge_resource_pack_entries,
    merge_resource_pack_order, read_resource_pack_entries,
};

pub(in crate::api::instance) use pack_updates::{
    detach_instance, prepare_instance_update_with_state,
};
pub(in crate::api::instance) use source_selection::initialize_from_source_instance;
pub(in crate::api::instance) use write_shared_settings::{
    apply_instance_with_state, capture_instance_file_change,
};

pub(in crate::api::instance) async fn canonical_exists(
    state: &crate::state::State,
) -> crate::Result<bool> {
    crate::state::shared_game_options_exist(&state.pool).await
}

const OPTIONS_FILE: &str = "options.txt";
const CATALOG_REVISION: u32 = 6;
const MAX_OPTIONS_BYTES: usize = 2 * 1024 * 1024;
const MAX_OPTIONS_LINES: usize = 16_384;
const MAX_KEY_BYTES: usize = 1_024;
const MAX_VALUE_BYTES: usize = 32 * 1024;
