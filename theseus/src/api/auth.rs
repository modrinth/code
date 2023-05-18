//! Authentication flow interface
use crate::{launcher::auth as inner, State};
use futures::prelude::*;
use tokio::sync::oneshot;

pub use inner::Credentials;

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL
/// This can be used in conjunction with 'authenticate_await_complete_flow'
/// to call authenticate and call the flow from the frontend.
/// Visit the URL in a browser, then call and await 'authenticate_await_complete_flow'.
pub async fn authenticate_begin_flow() -> crate::Result<url::Url> {
    let st = State::get().await?.clone();
    let url = st.auth_flow.write().await.begin_auth().await?;
    Ok(url)
}

/// Authenticate a user with Hydra - part 2
/// This completes the authentication flow quasi-synchronously, returning the credentials
/// This can be used in conjunction with 'authenticate_begin_flow'
/// to call authenticate and call the flow from the frontend.
pub async fn authenticate_await_complete_flow() -> crate::Result<Credentials> {
    let st = State::get().await?.clone();
    let credentials =
        st.auth_flow.write().await.await_auth_completion().await?;
    Ok(credentials)
}

/// Authenticate a user with Hydra
/// To run this, you need to first spawn this function as a task, then
/// open a browser to the given URL and finally wait on the spawned future
/// with the ability to cancel in case the browser is closed before finishing
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn authenticate(
    browser_url: oneshot::Sender<url::Url>,
) -> crate::Result<Credentials> {
    let mut flow = inner::HydraAuthFlow::new().await?;
    let state = State::get().await?;
    let mut users = state.users.write().await;

    let url = flow.prepare_login_url().await?;
    browser_url.send(url).map_err(|url| {
        crate::ErrorKind::OtherError(format!(
            "Error sending browser url to parent: {url}"
        ))
    })?;

    let credentials = flow.extract_credentials(&state.fetch_semaphore).await?;
    users.insert(&credentials).await?;

    if state.settings.read().await.default_user.is_none() {
        let mut settings = state.settings.write().await;
        settings.default_user = Some(credentials.id);
    }

    Ok(credentials)
}

/// Refresh some credentials using Hydra, if needed
/// This is the primary desired way to get credentials, as it will also refresh them.
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn refresh(user: uuid::Uuid) -> crate::Result<Credentials> {
    let state = State::get().await?;
    let mut users = state.users.write().await;

    let fetch_semaphore = &state.fetch_semaphore;
    futures::future::ready(users.get(user).ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to refresh nonexistent user with ID {user}"
        ))
        .as_error()
    }))
    .and_then(|mut credentials| async move {
        if chrono::offset::Utc::now() > credentials.expires {
            inner::refresh_credentials(&mut credentials, fetch_semaphore)
                .await?;
        }
        users.insert(&credentials).await?;
        Ok(credentials)
    })
    .await
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
