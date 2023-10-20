//! Authentication flow interface
use crate::{
    hydra::{self, init::DeviceLoginSuccess},
    launcher::auth as inner,
    State,
};
use chrono::Utc;

use crate::state::AuthTask;
pub use inner::Credentials;

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL
/// This can be used in conjunction with 'authenticate_await_complete_flow'
/// to call authenticate and call the flow from the frontend.
/// Visit the URL in a browser, then call and await 'authenticate_await_complete_flow'.
pub async fn authenticate_begin_flow() -> crate::Result<DeviceLoginSuccess> {
    let url = AuthTask::begin_auth().await?;
    Ok(url)
}

/// Authenticate a user with Hydra - part 2
/// This completes the authentication flow quasi-synchronously, returning the credentials
/// This can be used in conjunction with 'authenticate_begin_flow'
/// to call authenticate and call the flow from the frontend.
pub async fn authenticate_await_complete_flow() -> crate::Result<Credentials> {
    let credentials = AuthTask::await_auth_completion().await?;
    Ok(credentials)
}

/// Cancels the active authentication flow
pub async fn cancel_flow() -> crate::Result<()> {
    AuthTask::cancel().await
}

/// Refresh some credentials using Hydra, if needed
/// This is the primary desired way to get credentials, as it will also refresh them.
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn refresh(user: uuid::Uuid) -> crate::Result<Credentials> {
    let state = State::get().await?;
    let mut users = state.users.write().await;

    let mut credentials = users.get(user).ok_or_else(|| {
        crate::ErrorKind::OtherError(
            "You are not logged in with a Minecraft account!".to_string(),
        )
        .as_error()
    })?;

    let offline = *state.offline.read().await;

    if !offline {
        let fetch_semaphore: &crate::util::fetch::FetchSemaphore =
            &state.fetch_semaphore;
        if Utc::now() > credentials.expires
            && inner::refresh_credentials(&mut credentials, fetch_semaphore)
                .await
                .is_err()
        {
            users.remove(credentials.id).await?;

            return Err(crate::ErrorKind::OtherError(
                "Please re-authenticate with your Minecraft account!"
                    .to_string(),
            )
            .as_error());
        }

        // Update player info from bearer token
        let player_info =
            hydra::stages::player_info::fetch_info(&credentials.access_token)
                .await
                .map_err(|_err| {
                    crate::ErrorKind::HydraError(
                        "No Minecraft account for your profile. Please try again or contact support in our Discord for help!".to_string(),
                    )
                })?;

        credentials.username = player_info.name;
        users.insert(&credentials).await?;
    }

    Ok(credentials)
}

/// Remove a user account from the database
#[tracing::instrument]
pub async fn remove_user(user: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let mut users = state.users.write().await;

    if state.settings.read().await.default_user == Some(user) {
        let mut settings = state.settings.write().await;
        settings.default_user = users.0.values().next().map(|it| it.id);
    }

    users.remove(user).await?;
    Ok(())
}

/// Check if a user exists in Theseus
#[tracing::instrument]
pub async fn has_user(user: uuid::Uuid) -> crate::Result<bool> {
    let state = State::get().await?;
    let users = state.users.read().await;

    Ok(users.contains(user))
}

/// Get a copy of the list of all user credentials
#[tracing::instrument]
pub async fn users() -> crate::Result<Vec<Credentials>> {
    let state = State::get().await?;
    let users = state.users.read().await;
    Ok(users.0.values().cloned().collect())
}

/// Get a specific user by user ID
/// Prefer to use 'refresh' instead of this function
#[tracing::instrument]
pub async fn get_user(user: uuid::Uuid) -> crate::Result<Credentials> {
    let state = State::get().await?;
    let users = state.users.read().await;
    let user = users.get(user).ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to get nonexistent user with ID {user}"
        ))
        .as_error()
    })?;
    Ok(user)
}
