use tauri::{
    AboutMetadata, CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowMenuEvent, Wry,
};

use std::env::consts;

pub(crate) fn get_menu() -> Menu {
	match consts::OS {
		"macos" => custom_menu_bar(),
		_ => Menu::new(),
	}
}

fn custom_menu_bar() -> Menu {
    let app_menu = Menu::new()
        .add_native_item(MenuItem::About(
            "Theseus".to_string(),
            AboutMetadata::new(),
        ))
        .add_native_item(MenuItem::Separator)
        .add_item(
            CustomMenuItem::new("open_settings".to_string(), "Settings...")
                .accelerator("CmdOrCtrl+Comma"),
        )
        .add_native_item(MenuItem::Separator)
		.add_native_item(MenuItem::Services)
		.add_native_item(MenuItem::Separator)
		.add_native_item(MenuItem::Hide)
		.add_native_item(MenuItem::HideOthers)
		.add_native_item(MenuItem::ShowAll)
		.add_native_item(MenuItem::Separator)
		.add_native_item(MenuItem::Quit);
    
    let file_menu = Menu::new()
		.add_item(
			CustomMenuItem::new("new_window".to_string(), "New Window")
				.accelerator("CmdOrCtrl+N")
				.disabled(),
		)
		.add_item(
			CustomMenuItem::new("close".to_string(), "Close Window").accelerator("CmdOrCtrl+W"),
		);

    let edit_menu = Menu::new()
		.add_native_item(MenuItem::Copy)
		.add_native_item(MenuItem::Paste)
		.add_native_item(MenuItem::SelectAll);

    let view_menu = Menu::new();
	let window_menu = Menu::new().add_native_item(MenuItem::EnterFullScreen);

    #[cfg(debug_assertions)]
	let view_menu = {
		let view_menu = view_menu.add_native_item(MenuItem::Separator);

		#[cfg(target_os = "macos")]
		let view_menu = view_menu.add_item(
			CustomMenuItem::new("reload_app".to_string(), "Reload").accelerator("CmdOrCtrl+R"),
		);

		view_menu.add_item(
			CustomMenuItem::new("toggle_devtools".to_string(), "Toggle Developer Tools")
				.accelerator("CmdOrCtrl+Alt+I"),
		)
	};

    Menu::new()
		.add_submenu(Submenu::new("Theseus", app_menu))
		.add_submenu(Submenu::new("File", file_menu))
		.add_submenu(Submenu::new("Edit", edit_menu))
		.add_submenu(Submenu::new("View", view_menu))
		.add_submenu(Submenu::new("Window", window_menu))
}

pub(crate) fn handle_menu_event(event: WindowMenuEvent<Wry>) {
	match event.menu_item_id() {
		"quit" => {
			let app = event.window().app_handle();
			app.exit(0);
		}
		"open_settings" => event.window().emit("keybind", "open_settings").unwrap(),
		"close" => {
			let window = event.window();

			#[cfg(debug_assertions)]
			if window.is_devtools_open() {
				window.close_devtools();
			} else {
				window.close().unwrap();
			}

			#[cfg(not(debug_assertions))]
			window.close().unwrap();
		}
		"open_search" => event
			.window()
			.emit("keybind", "open_search".to_string())
			.unwrap(),
		"reload_app" => {
				unimplemented!();
		}
		#[cfg(debug_assertions)]
		"toggle_devtools" => {
			let window = event.window();

			if window.is_devtools_open() {
				window.close_devtools();
			} else {
				window.open_devtools();
			}
		}
		_ => {}
	}
}