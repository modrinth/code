use crate::api::Result;
use tauri::plugin::TauriPlugin;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("mr_auth")
        .invoke_handler(tauri::generate_handler![
            authenticate_begin_flow,
            authenticate_await_completion,
            cancel_flow,
            login_pass,
            login_2fa,
            create_account,
            refresh,
            logout,
            get,
        ])
        .build()
}

#[tauri::command]
pub async fn authenticate_begin_flow(provider: &str) -> Result<String> {
    Ok(theseus::mr_auth::authenticate_begin_flow(provider).await?)
}

#[tauri::command]
pub async fn authenticate_await_completion() -> Result<ModrinthCredentialsResult>
{
    Ok(theseus::mr_auth::authenticate_await_complete_flow().await?)
}

#[tauri::command]
pub async fn cancel_flow() -> Result<()> {
    Ok(theseus::mr_auth::cancel_flow().await?)
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
pub async fn refresh() -> Result<()> {
    Ok(theseus::mr_auth::refresh().await?)
}

#[tauri::command]
pub async fn logout() -> Result<()> {
    Ok(theseus::mr_auth::logout().await?)
}

#[tauri::command]
pub async fn get() -> Result<Option<ModrinthCredentials>> {
    Ok(theseus::mr_auth::get_credentials().await?)
}
