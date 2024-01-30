use crate::api::Result;
use tauri::plugin::TauriPlugin;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("auth")
        .invoke_handler(tauri::generate_handler![
            auth_begin_login,
            auth_finish_login,
            auth_get_default_user,
            auth_set_default_user,
            auth_remove_user,
            auth_users,
            auth_get_user,
        ])
        .build()
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn auth_begin_login() -> Result<MinecraftLoginFlow> {
    Ok(minecraft_auth::begin_login().await?)
}

/// Authenticate a user with Hydra - part 2
/// This completes the authentication flow quasi-synchronously, returning the sign-in credentials
/// (and also adding the credentials to the state)
#[tauri::command]
pub async fn auth_finish_login(
    code: &str,
    flow: MinecraftLoginFlow,
) -> Result<Credentials> {
    Ok(minecraft_auth::finish_login(code, flow).await?)
}

#[tauri::command]
pub async fn auth_remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn auth_get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn auth_set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
// invoke('plugin:auth|auth_users',user)
#[tauri::command]
pub async fn auth_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}

/// Get a user from the UUID
/// Prefer to use refresh instead, as it will refresh the credentials as well
// invoke('plugin:auth|auth_users',user)
#[tauri::command]
pub async fn auth_get_user(user: uuid::Uuid) -> Result<Credentials> {
    Ok(minecraft_auth::get_user(user).await?)
}
