#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;

mod api;

// Should be called in launcher initialization
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    theseus::EventState::init(app).await?;
    State::get().await?;
    State::update();
    Ok(())
}

// cfg only on mac os
// disables mouseover and fixes a random crash error only fixed by recent versions of macos
#[cfg(target_os = "macos")]
#[tauri::command]
async fn should_disable_mouseover() -> bool {
    // We try to match version to 12.2 or higher. If unrecognizable to pattern or lower, we default to the css with disabled mouseover for safety
    let os = os_info::get();
    if let os_info::Version::Semantic(major, minor, _) = os.version() {
        if *major >= 12 && *minor >= 3 {
            // Mac os version is 12.3 or higher, we allow mouseover
            return false;
        }
    }
    true
}
#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn should_disable_mouseover() -> bool {
    false
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            should_disable_mouseover,
            api::progress_bars_list,
            api::profile_create::profile_create_empty,
            api::profile_create::profile_create,
            api::profile::profile_remove,
            api::profile::profile_get,
            api::profile::profile_list,
            api::profile::profile_sync,
            api::profile::profile_install,
            api::profile::profile_update_all,
            api::profile::profile_update_project,
            api::profile::profile_replace_project,
            api::profile::profile_add_project_from_version,
            api::profile::profile_add_project_from_path,
            api::profile::profile_toggle_disable_project,
            api::profile::profile_remove_project,
            api::profile::profile_run,
            api::profile::profile_run_wait,
            api::profile::profile_run_credentials,
            api::profile::profile_run_wait_credentials,
            api::pack::pack_install_version_id,
            api::pack::pack_install_file,
            api::auth::auth_authenticate_begin_flow,
            api::auth::auth_authenticate_await_completion,
            api::auth::auth_refresh,
            api::auth::auth_remove_user,
            api::auth::auth_has_user,
            api::auth::auth_users,
            api::auth::auth_get_user,
            api::tags::tags_get_categories,
            api::tags::tags_get_donation_platforms,
            api::tags::tags_get_game_versions,
            api::tags::tags_get_loaders,
            api::tags::tags_get_report_types,
            api::tags::tags_get_tag_bundle,
            api::settings::settings_get,
            api::settings::settings_set,
            api::jre::jre_get_all_jre,
            api::jre::jre_autodetect_java_globals,
            api::jre::jre_find_jre_18plus_jres,
            api::jre::jre_find_jre_17_jres,
            api::jre::jre_find_jre_8_jres,
            api::jre::jre_validate_globals,
            api::jre::jre_get_optimal_jre_key,
            api::jre::jre_get_optimal_jre_key_by_path,
            api::jre::jre_get_jre,
            api::process::process_get_all_uuids,
            api::process::process_get_all_running_uuids,
            api::process::process_get_uuids_by_profile_path,
            api::process::process_get_all_running_profile_paths,
            api::process::process_get_all_running_profiles,
            api::process::process_get_exit_status_by_uuid,
            api::process::process_has_finished_by_uuid,
            api::process::process_get_stderr_by_uuid,
            api::process::process_get_stdout_by_uuid,
            api::process::process_kill_by_uuid,
            api::process::process_wait_for_by_uuid,
            api::metadata::metadata_get_game_versions,
            api::metadata::metadata_get_fabric_versions,
            api::metadata::metadata_get_forge_versions,
            api::logs::logs_get_logs,
            api::logs::logs_get_log_by_datetime,
            api::logs::logs_get_stdout_by_datetime,
            api::logs::logs_get_stderr_by_datetime,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
