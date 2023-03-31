use crate::api::Result;
use theseus::prelude::*;

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn auth_authenticate_begin_flow() -> Result<url::Url> {
    Ok(auth::authenticate_begin_flow().await?)
}

/// Authenticate a user with Hydra - part 2
/// This completes the authentication flow quasi-synchronously, returning the sign-in credentials
/// (and also adding the credentials to the state)
#[tauri::command]
pub async fn auth_authenticate_await_completion() -> Result<Credentials> {
    Ok(auth::authenticate_await_complete_flow().await?)
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

/// Get a user from the UUID
/// Prefer to use refresh instead, as it will refresh the credentials as well
// invoke('auth_users',user)
#[tauri::command]
pub async fn auth_get_user(user: uuid::Uuid) -> Result<Credentials> {
    Ok(auth::get_user(user).await?)
}

