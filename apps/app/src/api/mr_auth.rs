use crate::api::Result;
use crate::api::TheseusSerializableError;
use crate::api::oauth_utils;
use tauri::Manager;
use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use tauri_plugin_opener::OpenerExt;
use theseus::prelude::*;
use tokio::sync::oneshot;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("mr-auth")
        .invoke_handler(tauri::generate_handler![
            modrinth_login,
            logout,
            get,
            get_all,
            set_active,
            remove_account,
            cancel_modrinth_login,
        ])
        .build()
}

#[tauri::command]
pub async fn modrinth_login<R: Runtime>(
    app: tauri::AppHandle<R>,
    flow: mr_auth::ModrinthAuthFlow,
    add_account: Option<bool>,
) -> Result<ModrinthCredentials> {
    let (auth_code_recv_socket_tx, auth_code_recv_socket) = oneshot::channel();
    let auth_code = tokio::spawn(oauth_utils::auth_code_reply::listen(
        auth_code_recv_socket_tx,
    ));

    let auth_code_recv_socket = auth_code_recv_socket.await.unwrap()?;

    let mut auth_request_uri = format!(
        "{}?launcher=true&ipver={}&port={}",
        mr_auth::authenticate_begin_flow(flow).await?,
        if auth_code_recv_socket.is_ipv4() {
            "4"
        } else {
            "6"
        },
        auth_code_recv_socket.port()
    );

    if add_account.unwrap_or(false) {
        auth_request_uri.push_str("&add_account=true");
    }

    app.opener()
        .open_url(auth_request_uri, None::<&str>)
        .map_err(|e| {
            TheseusSerializableError::Theseus(
                theseus::ErrorKind::OtherError(format!(
                    "Failed to open auth request URI: {e}"
                ))
                .into(),
            )
        })?;

    let Some(auth_code) = auth_code.await.unwrap()? else {
        return Err(TheseusSerializableError::Theseus(
            theseus::ErrorKind::OtherError("Login canceled".into()).into(),
        ));
    };

    let credentials = mr_auth::authenticate_finish_flow(&auth_code).await?;

    if let Some(main_window) = app.get_window("main") {
        main_window.set_focus().ok();
    }

    Ok(credentials)
}

#[tauri::command]
pub async fn logout() -> Result<()> {
    Ok(theseus::mr_auth::logout().await?)
}

#[tauri::command]
pub async fn get() -> Result<Option<ModrinthCredentials>> {
    Ok(theseus::mr_auth::get_credentials().await?)
}

#[tauri::command]
pub async fn get_all() -> Result<Vec<ModrinthCredentials>> {
    Ok(theseus::mr_auth::get_all().await?)
}

#[tauri::command]
pub async fn set_active(user_id: String) -> Result<()> {
    Ok(theseus::mr_auth::set_active(&user_id).await?)
}

#[tauri::command]
pub async fn remove_account(user_id: String) -> Result<()> {
    Ok(theseus::mr_auth::remove_user(&user_id).await?)
}

#[tauri::command]
pub fn cancel_modrinth_login() {
    oauth_utils::auth_code_reply::stop_listeners();
}
