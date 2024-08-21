use crate::state::{ModrinthCredentials, ModrinthCredentialsResult};
use serde_json::Value;
use std::collections::HashMap;

#[tracing::instrument]
pub fn authenticate_begin_flow(provider: &str) -> String {
    crate::state::get_login_url(provider)
}

#[tracing::instrument]
pub async fn authenticate_finish_flow(
    response: HashMap<String, Value>,
) -> crate::Result<ModrinthCredentialsResult> {
    let state = crate::State::get().await?;

    let creds = crate::state::finish_login_flow(
        response,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    if let ModrinthCredentialsResult::Credentials(creds) = &creds {
        creds.upsert(&state.pool).await?;
    }

    Ok(creds)
}

pub async fn login_password(
    username: &str,
    password: &str,
    challenge: &str,
) -> crate::Result<ModrinthCredentialsResult> {
    let state = crate::State::get().await?;
    let creds = crate::state::login_password(
        username,
        password,
        challenge,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    if let ModrinthCredentialsResult::Credentials(creds) = &creds {
        creds.upsert(&state.pool).await?;
    }

    Ok(creds)
}

#[tracing::instrument]
pub async fn login_2fa(
    code: &str,
    flow: &str,
) -> crate::Result<ModrinthCredentials> {
    let state = crate::State::get().await?;
    let creds =
        crate::state::login_2fa(code, flow, &state.api_semaphore, &state.pool)
            .await?;

    creds.upsert(&state.pool).await?;

    Ok(creds)
}

#[tracing::instrument]
pub async fn create_account(
    username: &str,
    email: &str,
    password: &str,
    challenge: &str,
    sign_up_newsletter: bool,
) -> crate::Result<ModrinthCredentials> {
    let state = crate::State::get().await?;
    let creds = crate::state::create_account(
        username,
        email,
        password,
        challenge,
        sign_up_newsletter,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    creds.upsert(&state.pool).await?;

    Ok(creds)
}

#[tracing::instrument]
pub async fn logout() -> crate::Result<()> {
    let state = crate::State::get().await?;
    let current = ModrinthCredentials::get_active(&state.pool).await?;

    if let Some(current) = current {
        ModrinthCredentials::remove(&current.user_id, &state.pool).await?;
    }

    Ok(())
}

#[tracing::instrument]
pub async fn get_credentials() -> crate::Result<Option<ModrinthCredentials>> {
    let state = crate::State::get().await?;
    let current =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?;

    Ok(current)
}
