use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, UserAttentionType};
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("mr-auth")
        .invoke_handler(tauri::generate_handler![
            login_pass,
            login_2fa,
            create_account,
            logout,
            get,
        ])
        .build()
}

#[tauri::command]
pub async fn modrinth_auth_login(
    app: tauri::AppHandle,
    provider: &str,
) -> Result<Option<ModrinthCredentialsResult>> {
    let redirect_uri = mr_auth::authenticate_begin_flow(provider);

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
            .starts_with("https://launcher-files.modrinth.com/detect.txt")
        {
            let query = window
                .url()?
                .query_pairs()
                .map(|(key, val)| {
                    (
                        key.to_string(),
                        serde_json::Value::String(val.to_string()),
                    )
                })
                .collect();
            window.close()?;

            let val = mr_auth::authenticate_finish_flow(query).await?;

            return Ok(Some(val));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

#[tauri::command]
pub async fn login_pass(
    username: &str,
    password: &str,
    challenge: &str,
) -> Result<ModrinthCredentialsResult> {
    Ok(theseus::mr_auth::login_password(username, password, challenge).await?)
}

#[tauri::command]
pub async fn login_2fa(code: &str, flow: &str) -> Result<ModrinthCredentials> {
    Ok(theseus::mr_auth::login_2fa(code, flow).await?)
}

#[tauri::command]
pub async fn create_account(
    username: &str,
    email: &str,
    password: &str,
    challenge: &str,
    sign_up_newsletter: bool,
) -> Result<ModrinthCredentials> {
    Ok(theseus::mr_auth::create_account(
        username,
        email,
        password,
        challenge,
        sign_up_newsletter,
    )
    .await?)
}

#[tauri::command]
pub async fn logout() -> Result<()> {
    Ok(theseus::mr_auth::logout().await?)
}

#[tauri::command]
pub async fn get() -> Result<Option<ModrinthCredentials>> {
    Ok(theseus::mr_auth::get_credentials().await?)
}
