use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("mr-auth")
        .invoke_handler(tauri::generate_handler![modrinth_login, logout, get,])
        .build()
}

#[tauri::command]
pub async fn modrinth_login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<ModrinthCredentials>> {
    let redirect_uri = mr_auth::authenticate_begin_flow();

    let start = Utc::now();

    if let Some(window) = app.get_webview_window("modrinth-signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "modrinth-signin",
        tauri::WebviewUrl::External(redirect_uri.parse().map_err(|_| {
            theseus::ErrorKind::OtherError(
                "Error parsing auth redirect URL".to_string(),
            )
            .as_error()
        })?),
    )
    .min_inner_size(420.0, 632.0)
    .inner_size(420.0, 632.0)
    .max_inner_size(420.0, 632.0)
    .zoom_hotkeys_enabled(false)
    .title("Sign into Modrinth")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://launcher-files.modrinth.com")
        {
            let url = window.url()?;

            let code = url.query_pairs().find(|(key, _)| key == "code");

            window.close()?;

            return if let Some((_, code)) = code {
                let val = mr_auth::authenticate_finish_flow(&code).await?;

                Ok(Some(val))
            } else {
                Ok(None)
            };
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

#[tauri::command]
pub async fn logout() -> Result<()> {
    Ok(theseus::mr_auth::logout().await?)
}

#[tauri::command]
pub async fn get() -> Result<Option<ModrinthCredentials>> {
    Ok(theseus::mr_auth::get_credentials().await?)
}
