//! Theseus instance management interface

mod content;
mod content_set_diff;
mod export_mrpack;
mod get;
mod groups;
mod icon;
mod install;
mod lifecycle;
mod paths;
mod projects;
mod run;
mod screenshot_groups;
mod screenshots;
mod shared;
mod synced_options;
pub(crate) mod synced_servers;

pub use self::content::{
    get_content_items, get_dependencies_as_content_items,
    get_install_candidates, get_installed_project_ids,
    get_linked_modpack_content, get_linked_modpack_info, get_projects,
    list_content_sets, refresh_content_updates, sync_content_files,
};
pub use self::export_mrpack::{
    PackExportCandidate, create_mrpack_json, export_mrpack,
    get_pack_export_candidates, get_pack_export_candidates_for_parent,
};
pub use self::get::{get, get_many, list};
pub use self::groups::{
    FAVORITES_GROUP_ID, InstanceGroup, InstanceGroupMembershipUpdate,
    create_group, delete_group, list_groups, rename_group,
    set_group_memberships, set_group_order,
};
pub use self::icon::{
    cache_generated_icon, edit_generated_icon, edit_generated_icon_if_empty,
    edit_icon, get_recent_icon_configs,
};
pub(crate) use self::icon::{
    cache_icon, cache_icon_from_path, migrate_legacy_icons,
};
pub use self::install::get_optimal_jre_key;
pub(crate) use self::lifecycle::create;
pub use self::lifecycle::{edit, remove, set_synced_option};
pub use self::paths::{get_full_path, get_mod_full_path};
pub use self::projects::{
    InstallProjectWithDependenciesRequest, add_project_from_path,
    add_project_from_version, install_project_with_dependencies,
    is_file_on_modrinth, remove_project, repair_managed_modrinth,
    set_project_locked, switch_project_version_with_dependencies,
    toggle_disable_project, update_all_projects,
    update_managed_modrinth_version, update_project,
};
pub use self::run::{
    QuickPlayType, kill, run, try_update_playtime_by_instance_id,
};
pub use self::screenshot_groups::{
    ScreenshotGroup, ScreenshotGroupImport, ScreenshotGroupMembershipUpdate,
    create_screenshot_group, delete_screenshot_group, import_screenshot_groups,
    list_screenshot_groups, rename_screenshot_group,
    set_screenshot_group_memberships,
};
pub(crate) use self::screenshots::reconcile_screenshots;
pub use self::screenshots::{
    InstanceScreenshot, ScreenshotEditSaveMode, ScreenshotKey,
    delete_screenshots, export_screenshots, get_screenshot_path,
    list_all_screenshots, list_screenshots, list_synced_screenshots,
    move_screenshots, save_edited_screenshot,
};
pub(crate) use self::shared::{
    CONFIG_BUNDLE_FILE_TYPE, CONFIG_DIRECTORY, CONFIG_FILE_EXTENSIONS,
    CONFIG_SYNC_ENABLED, MAX_CONFIG_BUNDLE_ENTRIES,
    read_bounded_config_bundle_entry,
};
pub use self::shared::{
    SharedInstanceExternalFilePreview, SharedInstanceInstallPreview,
    SharedInstanceInvite, SharedInstanceInviteInstallPreview,
    SharedInstanceInviteLink, SharedInstanceJoinType,
    SharedInstancePublishPreview, SharedInstanceUpdateDiff,
    SharedInstanceUpdateDiffType, SharedInstanceUpdatePreview,
    SharedInstanceUser, SharedInstanceUsers,
    accept_pending_shared_instance_invite,
    accept_shared_instance_invite_for_install,
    can_active_user_use_shared_instances, create_shared_instance_invite_link,
    decline_pending_shared_instance_invite,
    get_shared_instance_install_preview, get_shared_instance_invites,
    get_shared_instance_publish_preview, get_shared_instance_update_preview,
    get_shared_instance_users, install_shared_instance,
    invite_shared_instance_users, publish_shared_instance,
    remove_shared_instance_users, revoke_shared_instance_invite,
    unlink_shared_instance, unpublish_shared_instance, update_shared_instance,
};
pub use self::synced_options::{
    GlobalSyncedOptions, SyncedOptionCapability, SyncedOptionJoinAction,
    SyncedOptionJoinPreview, SyncedOptionJoinResolution, SyncedOptionsOverview,
    get_capabilities as get_synced_option_capabilities, get_command_history,
    get_global_options as get_global_synced_options,
    get_instance_option_join_preview as get_synced_option_join_preview,
    get_overview as get_synced_options_overview, get_synced_options_folder,
    set_command_history, set_global_option as set_global_synced_option,
};
pub(crate) use self::synced_options::{
    monitor_persisted_processes, prepare_instance_update,
    reconcile_changed_file as reconcile_synced_option_file,
    remove_generated_instance_files,
};
pub use self::synced_options::{
    reconcile_all as reconcile_all_synced_options,
    reconcile_instance as reconcile_instance_synced_options,
};
pub use self::synced_servers::{
    DesyncServerMode, ServerSource, SyncedServer, desync_server,
    list_synced_servers, remove_synced_server, update_synced_server,
};
