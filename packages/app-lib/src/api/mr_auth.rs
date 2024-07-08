use crate::state::{
    ModrinthAuthFlow, ModrinthCredentials, ModrinthCredentialsResult,
};
use crate::ErrorKind;

#[tracing::instrument]
pub async fn authenticate_begin_flow(provider: &str) -> crate::Result<String> {
    let state = crate::State::get().await?;

    // Don't start an uncompleteable new flow if there's an existing locked one
    let mut write: tokio::sync::RwLockWriteGuard<'_, Option<ModrinthAuthFlow>> =
        state.modrinth_auth_flow.write().await;

    let mut flow = ModrinthAuthFlow::new(provider).await?;
    let url = flow.prepare_login_url().await?;

    *write = Some(flow);

    Ok(url)
}

#[tracing::instrument]
pub async fn authenticate_await_complete_flow(
) -> crate::Result<ModrinthCredentialsResult> {
    let state = crate::State::get().await?;

    let mut write = state.modrinth_auth_flow.write().await;
    if let Some(ref mut flow) = *write {
        let creds = flow.extract_credentials(&state.fetch_semaphore).await?;

        if let ModrinthCredentialsResult::Credentials(creds) = &creds {
            let mut write = state.credentials.write().await;
            write.login(creds.clone()).await?;
        }

        Ok(creds)
    } else {
        Err(ErrorKind::OtherError(
            "No active Modrinth authenication flow!".to_string(),
        )
        .into())
    }
}

#[tracing::instrument]
pub async fn cancel_flow() -> crate::Result<()> {
    let state = crate::State::get().await?;
    let mut write = state.modrinth_auth_flow.write().await;
    if let Some(ref mut flow) = *write {
        flow.close().await?;
    }
    *write = None;
    Ok(())
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
        &state.fetch_semaphore,
    )
    .await?;

    if let ModrinthCredentialsResult::Credentials(creds) = &creds {
        let mut write = state.credentials.write().await;
        write.login(creds.clone()).await?;
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
        crate::state::login_2fa(code, flow, &state.fetch_semaphore).await?;

    let mut write = state.credentials.write().await;
    write.login(creds.clone()).await?;

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
        &state.fetch_semaphore,
    )
    .await?;

    let mut write = state.credentials.write().await;
    write.login(creds.clone()).await?;

    Ok(creds)
}

#[tracing::instrument]
pub async fn refresh() -> crate::Result<()> {
    let state = crate::State::get().await?;

    let mut write = state.credentials.write().await;
    crate::state::refresh_credentials(&mut write, &state.fetch_semaphore)
        .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn logout() -> crate::Result<()> {
    let state = crate::State::get().await?;
    let mut write = state.credentials.write().await;
    write.logout().await?;

    Ok(())
}

#[tracing::instrument]
pub async fn get_credentials() -> crate::Result<Option<ModrinthCredentials>> {
    let state = crate::State::get().await?;
    let read = state.credentials.read().await;

    Ok(read.0.clone())
}
