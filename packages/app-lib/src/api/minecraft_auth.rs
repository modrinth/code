use anyhow::Context;
use chrono::{Duration, Utc};

use crate::State;
use crate::state::{Credentials, MinecraftLoginFlow, MinecraftProfile};
use uuid::Uuid;

#[tracing::instrument]
pub async fn check_reachable() -> crate::Result<()> {
    Ok(())
}

#[tracing::instrument]
pub async fn begin_login() -> crate::Result<MinecraftLoginFlow> {
    let state = State::get().await?;

    crate::state::login_begin(&state.pool).await
}

#[tracing::instrument]
pub async fn finish_login(
    code: &str,
    flow: MinecraftLoginFlow,
) -> crate::Result<Credentials> {
    let state = State::get().await?;

    let credentials =
        crate::state::login_finish(code, flow, &state.pool).await?;

    if let Err(error) =
        crate::onboarding_checklist::mark_logged_into_minecraft().await
    {
        tracing::warn!(
            "Failed to mark Minecraft login in onboarding checklist: {error}"
        );
    }

    Ok(credentials)
}

#[tracing::instrument]
pub async fn create_offline_account(username: &str) -> crate::Result<Credentials> {
    if username.is_empty()
        || username.len() > 16
        || !username
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(crate::ErrorKind::InputError(
            "offline username must contain 1 to 16 letters, numbers, or underscores".to_string(),
        )
        .into());
    }

    let state = State::get().await?;
    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id: Uuid::new_v4(),
            name: username.to_string(),
            ..MinecraftProfile::default()
        },
        access_token: String::new(),
        refresh_token: String::new(),
        expires: Utc::now() + Duration::days(36500),
        active: true,
    };
    credentials.upsert(&state.pool).await?;

    Ok(credentials)
}

#[tracing::instrument]
pub async fn get_default_user() -> crate::Result<Option<uuid::Uuid>> {
    let state = State::get().await?;
    let user = Credentials::get_default_credential(&state.pool).await?;
    Ok(user.map(|user| user.offline_profile.id))
}

#[tracing::instrument]
pub async fn set_default_user(user: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    let (_, mut user) = users
        .remove(&user)
        .context(format!("tried to get nonexistent user with ID {user}"))?;

    user.active = true;
    user.upsert(&state.pool).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn remove_user(uuid: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;

    if let Some((uuid, user)) = users.remove(&uuid) {
        Credentials::remove(uuid, &state.pool).await?;

        if user.active
            && let Some((_, mut user)) = users.into_iter().next()
        {
            user.active = true;
            user.upsert(&state.pool).await?;
        }
    }

    Ok(())
}

#[tracing::instrument]
pub async fn users() -> crate::Result<Vec<Credentials>> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    Ok(users.into_iter().map(|x| x.1).collect())
}
