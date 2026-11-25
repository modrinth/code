use crate::api::Result;
use anyhow::Context;
use chrono::{Duration, Utc};
use std::time::Instant;
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::analytics::{self, AppEventType, MicrosoftSignInFailureReason};
use theseus::prelude::*;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("auth")
        .invoke_handler(tauri::generate_handler![
            check_reachable,
            login,
            remove_user,
            get_default_user,
            set_default_user,
            get_users,
        ])
        .build()
}

/// Checks if the authentication servers are reachable.
#[tauri::command]
pub async fn check_reachable() -> Result<()> {
    minecraft_auth::check_reachable().await?;
    Ok(())
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    let started_at = Instant::now();
    let result = sign_in(&app).await;
    let duration_ms = analytics::elapsed_ms(started_at);

    let (event, credentials) = match result {
        Ok(SignInOutcome::SignedIn {
            credentials,
            nth_account,
            is_new_account,
        }) => (
            AppEventType::MicrosoftSignIn {
                nth_account,
                is_new_account,
                duration_ms,
            },
            Some(credentials),
        ),
        Ok(SignInOutcome::Cancelled) => (
            AppEventType::MicrosoftSignInFailed {
                reason: MicrosoftSignInFailureReason::Cancelled,
                duration_ms,
            },
            None,
        ),
        Ok(SignInOutcome::TimedOut) => (
            AppEventType::MicrosoftSignInFailed {
                reason: MicrosoftSignInFailureReason::TimedOut,
                duration_ms,
            },
            None,
        ),
        Err(error) => {
            analytics::add_event(
                minecraft_auth::ANALYTICS_SOURCE,
                AppEventType::MicrosoftSignInFailed {
                    reason: minecraft_auth::sign_in_failure_reason(&error),
                    duration_ms,
                },
            );
            return Err(error.into());
        }
    };

    analytics::add_event(minecraft_auth::ANALYTICS_SOURCE, event);
    Ok(credentials)
}

enum SignInOutcome {
    SignedIn {
        credentials: Credentials,
        nth_account: u32,
        is_new_account: bool,
    },
    Cancelled,
    TimedOut,
}

async fn sign_in<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> theseus::Result<SignInOutcome> {
    let flow = minecraft_auth::begin_login().await?;

    let start = Utc::now();

    if let Some(window) = app.get_webview_window("signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        "signin",
        tauri::WebviewUrl::External(
            flow.auth_request_uri
                .parse()
                .context("error parsing auth redirect URL")?,
        ),
    )
    .title("Sign into Modrinth")
    .always_on_top(true)
    .min_inner_size(500.0, 500.0)
    .inner_size(1000.0, 700.0)
    .focused(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(SignInOutcome::Cancelled);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
            && let Some((_, code)) =
                window.url()?.query_pairs().find(|x| x.0 == "code")
        {
            window.close()?;
            let previous_users = minecraft_auth::users().await?;
            let credentials =
                minecraft_auth::finish_login(&code.clone(), flow).await?;

            let is_new_account = !previous_users.iter().any(|user| {
                user.offline_profile.id == credentials.offline_profile.id
            });
            let nth_account = u32::try_from(previous_users.len())
                .unwrap_or(u32::MAX)
                .saturating_add(u32::from(is_new_account));

            return Ok(SignInOutcome::SignedIn {
                credentials,
                nth_account,
                is_new_account,
            });
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(SignInOutcome::TimedOut)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}
