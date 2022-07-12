//! Authentication flow interface
use crate::{launcher::auth as inner, State};
use futures::prelude::*;
use tokio::sync::oneshot;

pub use inner::Credentials;

/// Authenticate a user with Hydra
/// To run this, you need to first spawn this function as a task, then
/// open a browser to the given URL and finally wait on the spawned future
/// with the ability to cancel in case the browser is closed before finishing
#[tracing::instrument]
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

    let credentials = flow.extract_credentials().await?;
    users.insert(&credentials)?;

    if state.settings.read().await.default_user.is_none() {
        let mut settings = state.settings.write().await;
        settings.default_user = Some(credentials.id);
    }

    Ok(credentials)
}

/// Refresh some credentials using Hydra, if needed
#[tracing::instrument]
pub async fn refresh(
    user: uuid::Uuid,
    update_name: bool,
) -> crate::Result<Credentials> {
    let state = State::get().await?;
    let mut users = state.users.write().await;

    futures::future::ready(users.get(user)?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to refresh nonexistent user with ID {user}"
        ))
        .as_error()
    }))
    .and_then(|mut credentials| async move {
        if chrono::offset::Utc::now() > credentials.expires {
            inner::refresh_credentials(&mut credentials).await?;
            if update_name {
                inner::refresh_username(&mut credentials).await?;
            }
        }
        users.insert(&credentials)?;
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
        settings.default_user = users
            .0
            .first()?
            .map(|it| uuid::Uuid::from_slice(&it.0))
            .transpose()?;
    }

    users.remove(user)?;
    Ok(())
}

/// Check if a user exists in Theseus
#[tracing::instrument]
pub async fn has_user(user: uuid::Uuid) -> crate::Result<bool> {
    let state = State::get().await?;
    let users = state.users.read().await;

    Ok(users.contains(user)?)
}

/// Get a copy of the list of all user credentials
#[tracing::instrument]
pub async fn users() -> crate::Result<Box<[Credentials]>> {
    let state = State::get().await?;
    let users = state.users.read().await;
    users.iter().collect()
}
