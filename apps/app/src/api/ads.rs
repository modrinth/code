use tauri::plugin::TauriPlugin;
use tauri::{LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .invoke_handler(tauri::generate_handler![
            init_ads_window,
            hide_ads_window,
        ])
        .build()
}

#[tauri::command]
pub async fn init_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::api::Result<()> {
    if let Some(webview) = app.webviews().get("ads-window") {
        webview.set_position(LogicalPosition::new(x, y)).unwrap();
        webview.set_size(LogicalSize::new(width, height)).unwrap();
    } else {
        let window = app.get_window("main").unwrap();

        window.add_child(
            tauri::webview::WebviewBuilder::new(
                "ads-window",
                WebviewUrl::External("https://aditude-test.modrinth.com/promo-frame.html".parse().unwrap()),
            )
                .user_agent("ModrinthApp Ads Webview")
                .accept_first_mouse(true)
                .zoom_hotkeys_enabled(false)
                .transparent(true),
            LogicalPosition::new(x, y),
            LogicalSize::new(width, height),
        ).unwrap();
    }


    Ok(())
}

#[tauri::command]
pub async fn hide_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    if let Some(webview) = app.webviews().get("ads-window") {
        webview.set_position(LogicalPosition::new(-1000, -1000)).unwrap();
    }

    Ok(())
}