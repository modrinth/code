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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
