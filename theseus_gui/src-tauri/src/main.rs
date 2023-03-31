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
            api::profile::profile_add,
            api::profile::profile_add_path,
            api::profile::profile_remove,
            api::profile::profile_get,
            api::profile::profile_is_managed,
            api::profile::profile_is_loaded,
            api::profile::profile_list,
            api::profile::profile_run,
            api::profile::profile_run_wait,
            api::profile::profile_kill,
            api::profile::profile_wait_for,
            api::auth::auth_authenticate_begin_flow,
            api::auth::auth_authenticate_await_completion,
            api::auth::auth_refresh,
            api::auth::auth_remove_user,
            api::auth::auth_has_user,
            api::auth::auth_users,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
