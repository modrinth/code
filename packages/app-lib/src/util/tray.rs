use image::imageops::FilterType;
use tauri::image::Image;
use tauri::menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{
    MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
};
use tauri::{AppHandle, Emitter, Manager};

const TRAY_ID: &str = "main-tray";
const SHOW_ID: &str = "show";
const INSTANCE_ID: &str = "instance";
const QUIT_ID: &str = "quit";
const OPEN_INSTANCE_EVENT: &str = "tray://open-instance";
const MENU_ICON_SIZE: u32 = 32;

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    app.remove_tray_by_id(TRAY_ID);
}

fn running_instance_icon(icon_path: Option<&str>) -> Option<Image<'static>> {
    let icon = image::open(icon_path?).ok()?;
    let icon = image::imageops::resize(
        &icon.to_rgba8(),
        MENU_ICON_SIZE,
        MENU_ICON_SIZE,
        FilterType::Lanczos3,
    );

    Some(Image::new_owned(
        icon.into_raw(),
        MENU_ICON_SIZE,
        MENU_ICON_SIZE,
    ))
}

pub fn hide_to_tray(
    app: &AppHandle,
    window: &tauri::Window,
    instance_id: String,
    instance_name: &str,
    instance_icon_path: Option<&str>,
) -> crate::Result<()> {
    if app.tray_by_id(TRAY_ID).is_none() {
        let instance = IconMenuItem::with_id(
            app,
            INSTANCE_ID,
            instance_name,
            true,
            running_instance_icon(instance_icon_path),
            None::<&str>,
        )?;
        let show = MenuItem::with_id(
            app,
            SHOW_ID,
            "Show Modrinth App",
            true,
            None::<&str>,
        )?;
        let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
        let separator = PredefinedMenuItem::separator(app)?;
        let menu =
            Menu::with_items(app, &[&instance, &separator, &show, &quit])?;

        let mut builder = TrayIconBuilder::with_id(TRAY_ID)
            .tooltip("Modrinth App")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(move |app, event| match event.id().as_ref() {
                SHOW_ID => show_main_window(app),
                INSTANCE_ID => {
                    show_main_window(app);
                    let _ = app.emit(OPEN_INSTANCE_EVENT, &instance_id);
                }
                QUIT_ID => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    show_main_window(tray.app_handle());
                }
            });

        if let Some(icon) = app.default_window_icon() {
            builder = builder.icon(icon.clone());
        }
        builder.build(app)?;
    }

    window.hide()?;
    Ok(())
}

pub fn restore_from_tray(app: &AppHandle) {
    if app.tray_by_id(TRAY_ID).is_some() {
        show_main_window(app);
    }
}
