#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;

mod api;

// Should be called in launcher initialization
#[tauri::command]
async fn initialize_state() -> api::Result<()> {
    State::get().await?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            api::profile_create::profile_create_empty,
            api::profile_create::profile_create,
            api::profile::profile_remove,
            api::profile::profile_get,
            api::profile::profile_list,
            api::profile::profile_run,
            api::profile::profile_run_wait,
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
            api::tags::tags_get_licenses,
            api::tags::tags_get_report_types,
            api::tags::tags_get_tag_bundle,
            api::settings::settings_get,
            api::settings::settings_set,
            api::process::process_get_all_pids,
            api::process::process_get_all_running_pids,
            api::process::process_get_pids_by_profile_path,
            api::process::process_get_all_running_profile_paths,
            api::process::process_get_all_running_profiles,
            api::process::process_get_exit_status_by_pid,
            api::process::process_has_finished_by_pid,
            api::process::process_get_stderr_by_pid,
            api::process::process_get_stdout_by_pid,
            api::process::process_kill_by_pid,
            api::process::process_wait_for_by_pid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
