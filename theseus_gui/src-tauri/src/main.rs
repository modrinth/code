#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use std::error::Error;
use std::time::Duration;

use tauri::{
	Manager, RunEvent, WindowBuilder,
};
use tokio::time::sleep;
use tracing::{debug, error};

mod menu;

#[tauri::command(async)]
async fn app_ready(app_handle: tauri::AppHandle) {
    let window = app_handle.get_window("main").unwrap();
    
    window.show().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let app = tauri::Builder::default()
		.setup(|app| {
			tokio::spawn({
				let window = WindowBuilder::new(app, "main", tauri::WindowUrl::default())
				.title("Modrinth")
				.resizable(true)
				.decorations(true)
				.always_on_top(false)
				.inner_size(800.0, 550.0)
				.min_inner_size(400.0, 200.0)
				.skip_taskbar(false)
				.fullscreen(false)
				.build()?;

				async move {
					sleep(Duration::from_secs(3)).await;
					if !window.is_visible().unwrap_or(true) {
						println!("Window did not emit `app_ready` event fast enough. Showing window...");
						let _ = window.show();
					}
				}
			});

			Ok(())
		})
		.on_menu_event(menu::handle_menu_event)
		.invoke_handler(tauri::generate_handler![app_ready])
		.menu(menu::get_menu())
		.build(tauri::generate_context!())?; // Run `pnpm build:web` to remove this error.
	
	app.run(move |app_handler, event| {
		if let RunEvent::ExitRequested { .. } = event {
			debug!("Closing all open windows...");
			app_handler
				.windows()
				.iter()
				.for_each(|(window_name, window)| {
					debug!("closing window: {window_name}");
					if let Err(e) = window.close() {
						error!("failed to close window '{}': {:#?}", window_name, e);
					}
				});

			app_handler.exit(0);
		}
	});
	
	Ok(())
}