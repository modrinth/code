use crate::state::{FeatureFlag, ModrinthCredentials, Settings};
use serde::Deserialize;

const LOCALHOST_LOGIN_URL: &str = "http://localhost:3000/auth/sign-in";
const LOCALHOST_SIGNUP_URL: &str = "http://localhost:3000/auth/sign-up";

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModrinthAuthFlow {
    SignIn,
    SignUp,
}

#[tracing::instrument]
pub async fn authenticate_begin_flow(
    flow: ModrinthAuthFlow,
) -> crate::Result<&'static str> {
    let state = crate::State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    let use_localhost = settings
        .feature_flags
        .get(&FeatureFlag::LocalhostSignIn)
        .copied()
        .unwrap_or(false);

    Ok(match (use_localhost, flow) {
        (true, ModrinthAuthFlow::SignIn) => LOCALHOST_LOGIN_URL,
        (true, ModrinthAuthFlow::SignUp) => LOCALHOST_SIGNUP_URL,
        (false, ModrinthAuthFlow::SignIn) => crate::state::get_login_url(),
        (false, ModrinthAuthFlow::SignUp) => crate::state::get_signup_url(),
    })
}

#[tracing::instrument]
pub async fn authenticate_finish_flow(
    code: &str,
) -> crate::Result<ModrinthCredentials> {
    let state = crate::State::get().await?;

    let creds = crate::state::finish_login_flow(
        code,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    creds.upsert(&state.pool).await?;

    if let Err(error) =
        crate::onboarding_checklist::mark_logged_into_modrinth().await
    {
        tracing::warn!(
            "Failed to mark Modrinth login in onboarding checklist: {error}"
        );
    }

    state.friends_socket.disconnect().await?;
    state
        .friends_socket
        .connect(&state.pool, &state.api_semaphore, &state.process_manager)
        .await?;

    Ok(creds)
}

#[tracing::instrument]
pub async fn logout() -> crate::Result<()> {
    let state = crate::State::get().await?;
    let current = ModrinthCredentials::get_active(&state.pool).await?;

    if let Some(current) = current {
        ModrinthCredentials::remove(&current.user_id, &state.pool).await?;
    }
    state.friends_socket.disconnect().await?;

    Ok(())
}

#[tracing::instrument]
pub async fn get_credentials() -> crate::Result<Option<ModrinthCredentials>> {
    let state = crate::State::get().await?;
    let current =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?;
    if current.is_none() {
        state.friends_socket.disconnect().await?;
    }

    Ok(current)
}
