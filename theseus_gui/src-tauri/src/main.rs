#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;

use tauri::Manager;

use tracing_error::ErrorLayer;
use tracing_subscriber::EnvFilter;

mod api;
mod error;

// Should be called in launcher initialization
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    theseus::EventState::init(app).await?;
    State::get().await?;
    State::update();
    Ok(())
}

#[tauri::command]
fn is_dev() -> bool {
    if cfg!(debug_assertions) {
        true
    } else {
        false
    }
}

use tracing_subscriber::prelude::*;

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn main() {
    let client = sentry::init("https://19a14416dafc4b4a858fa1a38db3b704@o485889.ingest.sentry.io/4505349067374592");

    let _guard = sentry_rust_minidump::init(&client);
    /*
       tracing is set basd on the environment variable RUST_LOG=xxx, depending on the amount of logs to show
           ERROR > WARN > INFO > DEBUG > TRACE
       eg. RUST_LOG=info will show info, warn, and error logs
           RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)
           RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)

       Error messages returned to Tauri will display as traced error logs if they return an error.
       This will also include an attached span trace if the error is from a tracing error, and the level is set to info, debug, or trace

       on unix:
           RUST_LOG="theseus=trace" {run command}

    */
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("theseus=info"));

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build());

    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.setup(|app| {
            let win = app.get_window("main").unwrap();
            win.set_decorations(false).unwrap();
            Ok(())
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        use window_shadows::set_shadow;

        builder = builder.setup(|app| {
            let win = app.get_window("main").unwrap();
            set_shadow(&win, true).unwrap();
            Ok(())
        });
    }

    #[cfg(target_os = "macos")]
    {
        use tauri::WindowEvent;

        builder = builder
            .setup(|app| {
                use api::window_ext::WindowExt;
                let win = app.get_window("main").unwrap();
                win.set_transparent_titlebar(true);
                win.position_traffic_lights(9.0, 16.0);
                Ok(())
            })
            .on_window_event(|e| {
                use api::window_ext::WindowExt;
                if let WindowEvent::Resized(..) = e.event() {
                    let win = e.window();
                    win.position_traffic_lights(9.0, 16.0);
                }
            })
    }

    builder = builder.invoke_handler(tauri::generate_handler![
        initialize_state,
        is_dev,
        api::progress_bars_list,
        api::profile_create::profile_create_empty,
        api::profile_create::profile_create,
        api::profile::profile_remove,
        api::profile::profile_get,
        api::profile::profile_get_optimal_jre_key,
        api::profile::profile_list,
        api::profile::profile_install,
        api::profile::profile_update_all,
        api::profile::profile_update_project,
        api::profile::profile_add_project_from_version,
        api::profile::profile_add_project_from_path,
        api::profile::profile_toggle_disable_project,
        api::profile::profile_remove_project,
        api::profile::profile_run,
        api::profile::profile_run_wait,
        api::profile::profile_run_credentials,
        api::profile::profile_run_wait_credentials,
        api::profile::profile_edit,
        api::profile::profile_edit_icon,
        api::profile::profile_check_installed,
        api::pack::pack_install_version_id,
        api::pack::pack_install_file,
        api::auth::auth_authenticate_begin_flow,
        api::auth::auth_authenticate_await_completion,
        api::auth::auth_cancel_flow,
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
        api::jre::jre_get_jre,
        api::jre::jre_auto_install_java,
        api::jre::jre_get_max_memory,
        api::process::process_get_all_uuids,
        api::process::process_get_all_running_uuids,
        api::process::process_get_uuids_by_profile_path,
        api::process::process_get_all_running_profile_paths,
        api::process::process_get_all_running_profiles,
        api::process::process_get_exit_status_by_uuid,
        api::process::process_has_finished_by_uuid,
        api::process::process_get_output_by_uuid,
        api::process::process_kill_by_uuid,
        api::process::process_wait_for_by_uuid,
        api::metadata::metadata_get_game_versions,
        api::metadata::metadata_get_fabric_versions,
        api::metadata::metadata_get_forge_versions,
        api::metadata::metadata_get_quilt_versions,
        api::logs::logs_get_logs,
        api::logs::logs_get_logs_by_datetime,
        api::logs::logs_get_output_by_datetime,
        api::logs::logs_delete_logs,
        api::logs::logs_delete_logs_by_datetime,
        api::utils::show_in_folder,
        api::utils::should_disable_mouseover,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    #[allow(deref_nullptr)]
    unsafe {
        *std::ptr::null_mut() = true;
    }
}
