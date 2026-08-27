use tauri::{
    AppHandle, Emitter, Runtime,
    menu::{
        AboutMetadata, HELP_SUBMENU_ID, Menu, MenuBuilder, MenuEvent,
        MenuItemBuilder, SubmenuBuilder, WINDOW_SUBMENU_ID,
    },
};

// Default macos undo/redo doesn't get passed through into the webkit window
const UNDO_MENU_ITEM_ID: &str = "edit-menu-undo";
const REDO_MENU_ITEM_ID: &str = "edit-menu-redo";
const UNDO_EVENT: &str = "edit-menu://undo";
const REDO_EVENT: &str = "edit-menu://redo";

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let package_info = app.package_info();
    let config = app.config();
    let about_metadata = AboutMetadata {
        name: Some(package_info.name.clone()),
        version: Some(package_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config
            .bundle
            .publisher
            .clone()
            .map(|publisher| vec![publisher]),
        ..Default::default()
    };

    let app_menu = SubmenuBuilder::new(app, package_info.name.clone())
        .about(Some(about_metadata))
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .separator()
        .quit()
        .build()?;
    let file_menu = SubmenuBuilder::new(app, "File").close_window().build()?;
    let undo = MenuItemBuilder::with_id(UNDO_MENU_ITEM_ID, "Undo")
        .accelerator("CmdOrCtrl+Z")
        .build(app)?;
    let redo = MenuItemBuilder::with_id(REDO_MENU_ITEM_ID, "Redo")
        .accelerator("CmdOrCtrl+Shift+Z")
        .build(app)?;
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&undo)
        .item(&redo)
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let view_menu = SubmenuBuilder::new(app, "View").fullscreen().build()?;
    let window_menu = SubmenuBuilder::with_id(app, WINDOW_SUBMENU_ID, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;
    let help_menu =
        SubmenuBuilder::with_id(app, HELP_SUBMENU_ID, "Help").build()?;

    MenuBuilder::new(app)
        .items(&[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
            &help_menu,
        ])
        .build()
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let event_name = match event.id().as_ref() {
        UNDO_MENU_ITEM_ID => UNDO_EVENT,
        REDO_MENU_ITEM_ID => REDO_EVENT,
        _ => return,
    };

    if let Err(error) = app.emit_to("main", event_name, ()) {
        tracing::warn!(?error, "Failed to emit edit menu event");
    }
}
