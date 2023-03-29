use crate::api::Result;
use tokio::sync::oneshot;

use theseus::prelude::*;

/// Authenticate a user with Hydra
/// TODO
/// To run this, you need to first spawn this function as a task, then
/// open a browser to the given URL and finally wait on the spawned future
/// with the ability to cancel in case the browser is closed before finishing
#[tauri::command]
pub async fn auth_authenticate_get_browser(
    browser_url: oneshot::Sender<url::Url>,
) -> Result<Credentials> {
    Ok(auth::authenticate(browser_url).await?)
}

/// Refresh some credentials using Hydra, if needed
// invoke('auth_refresh',user)
#[tauri::command]
pub async fn auth_refresh(
    user: uuid::Uuid,
    update_name: bool,
) -> Result<Credentials> {
    Ok(auth::refresh(user, update_name).await?)
}

/// Remove a user account from the database
// invoke('auth_remove_user',user)
#[tauri::command]
pub async fn auth_remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(auth::remove_user(user).await?)
}

/// Check if a user exists in Theseus
// invoke('auth_has_user',user)
#[tauri::command]
pub async fn auth_has_user(user: uuid::Uuid) -> Result<bool> {
    Ok(auth::has_user(user).await?)
}

/// Get a copy of the list of all user credentials
// invoke('auth_users',user)
#[tauri::command]
pub async fn auth_users() -> Result<Box<[Credentials]>> {
    Ok(auth::users().await?)
}