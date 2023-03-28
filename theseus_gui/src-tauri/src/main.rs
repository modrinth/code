#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]


mod api;

fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        api::profile_create::profile_create_empty

    ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
