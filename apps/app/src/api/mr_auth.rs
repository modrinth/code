use crate::api::Result;
use crate::api::PteronSerializableError;
use crate::api::oauth_utils;
use tauri::Manager;
use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use tauri_plugin_opener::OpenerExt;
use pteron::prelude::*;
use tokio::sync::oneshot;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("mr-auth")
        .invoke_handler(tauri::generate_handler![
            modrinth_login,
            logout,
            get,
            cancel_modrinth_login,
        ])
        .build()
}

#[tauri::command]
pub async fn modrinth_login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<ModrinthCredentials> {
    let (auth_code_recv_socket_tx, auth_code_recv_socket) = oneshot::channel();
    let auth_code = tokio::spawn(oauth_utils::auth_code_reply::listen(
        auth_code_recv_socket_tx,
    ));

    let auth_code_recv_socket = auth_code_recv_socket.await.unwrap()?;

    let auth_request_uri = format!(
        "{}?launcher=true&ipver={}&port={}",
        mr_auth::authenticate_begin_flow(),
        if auth_code_recv_socket.is_ipv4() {
            "4"
        } else {
            "6"
        },
        auth_code_recv_socket.port()
    );

    app.opener()
        .open_url(auth_request_uri, None::<&str>)
        .map_err(|e| {
            PteronSerializableError::Theseus(
                pteron::ErrorKind::OtherError(format!(
                    "Failed to open auth request URI: {e}"
                ))
                .into(),
            )
        })?;

    let Some(auth_code) = auth_code.await.unwrap()? else {
        return Err(PteronSerializableError::Theseus(
            pteron::ErrorKind::OtherError("Login canceled".into()).into(),
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
    Ok(pteron::mr_auth::logout().await?)
}

#[tauri::command]
pub async fn get() -> Result<Option<ModrinthCredentials>> {
    Ok(pteron::mr_auth::get_credentials().await?)
}

#[tauri::command]
pub fn cancel_modrinth_login() {
    oauth_utils::auth_code_reply::stop_listeners();
}
