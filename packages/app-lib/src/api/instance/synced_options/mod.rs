mod command_history;
mod files;
mod hotbars;
mod orchestration;

const COMMAND_HISTORY_FILE: &str = "command_history.txt";
const HOTBAR_FILE: &str = "hotbar.nbt";

pub use self::command_history::{get_command_history, set_command_history};
pub(super) use self::files::{
    CheckpointStatus, checkpoint, detach_link, ensure_link, finish_checkpoint,
    instance_dir, instance_is_running, instance_option_enabled, nbt_from_bytes,
    nbt_to_bytes, read_nbt_file, safe_instance_id, sha1_bytes, sha1_file,
    sync_files_are_protected,
};
pub(super) use self::orchestration::instance_option_supported;
pub use self::orchestration::{
    GlobalSyncedOptions, SyncedOptionCapability, SyncedOptionJoinAction,
    SyncedOptionJoinPreview, SyncedOptionJoinResolution, SyncedOptionsOverview,
    get_capabilities, get_global_options, get_instance_option_join_preview,
    get_overview, get_synced_options_folder, reconcile_all,
    reconcile_changed_file, reconcile_instance, set_global_option,
    set_instance_option, synced_options_path,
};
pub(crate) use self::orchestration::{
    monitor_persisted_processes, prepare_instance_update,
    remove_generated_instance_files,
};
