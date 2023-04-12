use crate::api::Result;
use theseus::{prelude::*, window_scoped};

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn auth_authenticate_begin_flow(
    window: tauri::Window,
) -> Result<url::Url> {
    Ok(window_scoped!(window, auth::authenticate_begin_flow()).await?)
}

/// Authenticate a user with Hydra - part 2
/// This completes the authentication flow quasi-synchronously, returning the sign-in credentials
/// (and also adding the credentials to the state)
#[tauri::command]
pub async fn auth_authenticate_await_completion(
    window: tauri::Window,
) -> Result<Credentials> {
    Ok(
        window_scoped!(window, auth::authenticate_await_complete_flow())
            .await?,
    )
}

/// Refresh some credentials using Hydra, if needed
// invoke('auth_refresh',user)
#[tauri::command]
pub async fn auth_refresh(
    window: tauri::Window,
    user: uuid::Uuid,
    update_name: bool,
) -> Result<Credentials> {
    Ok(window_scoped!(window, auth::refresh(user, update_name)).await?)
}

/// Remove a user account from the database
// invoke('auth_remove_user',user)
#[tauri::command]
pub async fn auth_remove_user(
    window: tauri::Window,
    user: uuid::Uuid,
) -> Result<()> {
    Ok(window_scoped!(window, auth::remove_user(user)).await?)
}

/// Check if a user exists in Theseus
// invoke('auth_has_user',user)
#[tauri::command]
pub async fn auth_has_user(
    window: tauri::Window,
    user: uuid::Uuid,
) -> Result<bool> {
    Ok(window_scoped!(window, auth::has_user(user)).await?)
}

/// Get a copy of the list of all user credentials
// invoke('auth_users',user)
#[tauri::command]
pub async fn auth_users(window: tauri::Window) -> Result<Box<[Credentials]>> {
    Ok(window_scoped!(window, auth::users()).await?)
}

/// Get a user from the UUID
/// Prefer to use refresh instead, as it will refresh the credentials as well
// invoke('auth_users',user)
#[tauri::command]
pub async fn auth_get_user(
    window: tauri::Window,
    user: uuid::Uuid,
) -> Result<Credentials> {
    Ok(window_scoped!(window, auth::get_user(user)).await?)
}
