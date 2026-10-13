use crate::api::Result;
use crate::api::TheseusSerializableError;
use crate::api::oauth_utils;
use crate::api::oauth_utils::auth_code_reply::AuthReplySession;
use crate::api::oauth_utils::auth_seal::AuthSeal;
use tauri::Manager;
use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use tauri_plugin_opener::OpenerExt;
use theseus::prelude::*;
use url::Url;

const LAUNCHER_AUTH_PROTOCOL: &str = "2";

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
    let seal = AuthSeal::generate();
    let mut reply = AuthReplySession::start(seal.nonce().to_string());
    let auth_code_recv_socket = reply.socket_addr().await?;
    let auth_request_uri = auth_request_url(
        mr_auth::authenticate_begin_flow(flow).await?,
        &auth_code_recv_socket,
        &seal.public_key()?,
        seal.nonce(),
        add_account.unwrap_or(false),
    )?;

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

    let Some(auth_code) = reply.wait().await? else {
        return Err(TheseusSerializableError::Theseus(
            theseus::ErrorKind::OtherError("Login canceled".into()).into(),
        ));
    };
    let session_token = seal.open(&auth_code)?;
    let credentials = mr_auth::authenticate_finish_flow(&session_token).await?;

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
    oauth_utils::auth_code_reply::cancel_pending_auth();
}

fn auth_request_url(
    begin_url: &str,
    socket: &std::net::SocketAddr,
    public_key: &str,
    nonce: &str,
    add_account: bool,
) -> Result<String> {
    let mut url = Url::parse(begin_url).map_err(|error| -> theseus::Error {
        theseus::ErrorKind::OtherError(format!(
            "Failed to build auth request URI: {error}"
        ))
        .into()
    })?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("launcher", "true");
        query.append_pair("protocol", LAUNCHER_AUTH_PROTOCOL);
        query.append_pair("key", public_key);
        query.append_pair("nonce", nonce);
        query.append_pair("ipver", if socket.is_ipv4() { "4" } else { "6" });
        query.append_pair("port", &socket.port().to_string());
        if add_account {
            query.append_pair("add_account", "true");
        }
    }

    Ok(url.into())
}
