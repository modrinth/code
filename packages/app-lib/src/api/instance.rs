//! Theseus instance management interface

mod content;
mod content_set_diff;
mod export_mrpack;
mod get;
mod install;
mod lifecycle;
mod paths;
mod projects;
mod run;
mod shared;

pub use self::content::{
    get_content_items, get_dependencies_as_content_items,
    get_install_candidates, get_installed_project_ids,
    get_linked_modpack_content, get_linked_modpack_info, get_projects,
    list_content_sets, sync_content_files,
};
pub use self::export_mrpack::{
    create_mrpack_json, export_mrpack, get_pack_export_candidates,
};
pub use self::get::{get, get_many, list};
pub use self::install::get_optimal_jre_key;
pub(crate) use self::lifecycle::create;
pub use self::lifecycle::{edit, edit_icon, remove};
pub use self::paths::{get_full_path, get_mod_full_path};
pub use self::projects::{
    InstallProjectWithDependenciesRequest, add_project_from_path,
    add_project_from_version, install_project_with_dependencies,
    remove_project, repair_managed_modrinth,
    switch_project_version_with_dependencies, toggle_disable_project,
    update_all_projects, update_managed_modrinth_version, update_project,
};
pub use self::run::{
    QuickPlayType, kill, run, try_update_playtime_by_instance_id,
};
pub(crate) use self::shared::mark_shared_instance_stale;
pub use self::shared::{
    SharedInstanceExternalFilePreview, SharedInstanceInstallPreview,
    SharedInstanceInviteInstallPreview, SharedInstanceInviteLink,
    SharedInstanceJoinType, SharedInstancePublishPreview,
    SharedInstanceUpdateDiff, SharedInstanceUpdateDiffType,
    SharedInstanceUpdatePreview, SharedInstanceUser, SharedInstanceUsers,
    accept_pending_shared_instance_invite,
    accept_shared_instance_invite_for_install,
    create_shared_instance_invite_link, decline_pending_shared_instance_invite,
    get_shared_instance_install_preview, get_shared_instance_publish_preview,
    get_shared_instance_update_preview, get_shared_instance_users,
    install_shared_instance, invite_shared_instance_users,
    publish_shared_instance, remove_shared_instance_users,
    unlink_shared_instance, unpublish_shared_instance, update_shared_instance,
};
