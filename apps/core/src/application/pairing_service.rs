//! Pairing: how a Core comes to belong to an account.
//!
//! An unowned Core shows a short code and registers it with the backend. The owner types the code
//! into the App, which sends it here with their account token. Core checks the code itself, asks
//! the backend whether that account really claimed it, and only then stores the owner together
//! with the key the backend issues. A paired Core reports its address with that key on every start.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    application::state::{AppState, PairingCode},
    presentation::{error::ApiError, extractors::Account},
};

/// How long a shown code can be used.
const CODE_TTL: Duration = Duration::from_secs(15 * 60);
/// Wait between attempts to reach the backend.
const RETRY: Duration = Duration::from_secs(10);
/// Wrong codes one account may send within `GUESS_WINDOW`.
const MAX_WRONG_GUESSES: u32 = 5;
const GUESS_WINDOW: Duration = Duration::from_secs(10 * 60);
/// 32 characters, none of which look alike (no I, O, 0 or 1).
const ALPHABET: &[u8] = b"abcdefghjklmnpqrstuvwxyz23456789";

/// Keeps this Core either showing a pairing code or known to the backend. Runs for Core's lifetime.
pub async fn run(state: Arc<AppState>) {
    loop {
        let owner = match state.owner().await {
            Ok(owner) => owner,
            Err(error) => {
                warn!(%error, "could not read the Core owner");
                tokio::time::sleep(RETRY).await;
                continue;
            }
        };
        match owner.and_then(|owner| owner.backend_key) {
            Some(key) => match report_address(&state, &key).await {
                Report::Accepted => state.pairing_changed.notified().await,
                Report::Changed => {}
                Report::Rejected => {
                    println!(
                        "\nThis Core was unpaired from its account. Its servers and files are untouched.\n"
                    );
                    if let Err(error) = state.forget_owner().await {
                        warn!(%error, "could not forget the Core owner");
                        tokio::time::sleep(RETRY).await;
                    }
                }
            },
            None => show_code(&state).await,
        }
    }
}

enum Report {
    Accepted,
    /// The backend no longer lists this Core: it was unpaired while it was unreachable.
    Rejected,
    /// Pairing changed while waiting to retry.
    Changed,
}

/// Tell the backend where this Core is. Retries until the backend answers.
async fn report_address(state: &AppState, key: &str) -> Report {
    loop {
        let response = state
            .http
            .post(format!("{}/core/address", state.config.account_api_url))
            .bearer_auth(key)
            .json(&json!({ "address": state.config.public_url }))
            .send()
            .await;
        match response {
            Ok(response) if response.status().is_success() => {
                info!(address = %state.config.public_url, "reported Core address");
                return Report::Accepted;
            }
            Ok(response) if response.status() == StatusCode::UNAUTHORIZED => {
                return Report::Rejected;
            }
            Ok(response) => {
                let reason = error_text(response).await;
                println!(
                    "\nAmberite did not accept this Core's address ({}): {reason}\nThe App may not find this Core until this is fixed. Retrying.\n",
                    state.config.public_url
                );
            }
            Err(error) => {
                warn!(%error, "could not reach Amberite to report this Core's address, retrying");
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(RETRY * 3) => {}
            _ = state.pairing_changed.notified() => return Report::Changed,
        }
    }
}

/// Show one pairing code until it is used or expires. The code is printed before it is
/// registered, so a backend problem never hides it; the problem is printed underneath.
async fn show_code(state: &AppState) {
    let code = generate_code();
    let expires_at = Instant::now() + CODE_TTL;
    *state.pairing_code.lock().unwrap() = Some(PairingCode {
        code: code.clone(),
        expires_at,
    });
    println!(
        "\nPairing code: {}-{}\nIn the Amberite App, choose \"Pair a Core\" and type this code. It works for 15 minutes.\n",
        code[..4].to_ascii_uppercase(),
        code[4..].to_ascii_uppercase()
    );

    let expired = tokio::time::sleep_until(expires_at.into());
    tokio::pin!(expired);
    let mut registered = false;
    let mut last_problem = String::new();
    loop {
        if !registered {
            match register_code(state, &code).await {
                Ok(()) => {
                    registered = true;
                    if !last_problem.is_empty() {
                        println!("\nThe pairing code is ready to use.\n");
                    }
                }
                // Another Core is showing the same code. Show a different one.
                Err(RegisterError::CodeTaken) => break,
                Err(RegisterError::Other(problem)) => {
                    if problem != last_problem {
                        println!(
                            "\nThe pairing code cannot be used yet: {problem}\nRetrying every {} seconds.\n",
                            RETRY.as_secs()
                        );
                        last_problem = problem;
                    }
                }
            }
        }
        tokio::select! {
            _ = &mut expired => {
                println!("\nThe pairing code expired.");
                break;
            }
            _ = state.pairing_changed.notified() => break,
            _ = tokio::time::sleep(RETRY), if !registered => {}
        }
    }

    let mut current = state.pairing_code.lock().unwrap();
    if current.as_ref().is_some_and(|shown| shown.code == code) {
        *current = None;
    }
}

enum RegisterError {
    CodeTaken,
    Other(String),
}

async fn register_code(
    state: &AppState,
    code: &str,
) -> Result<(), RegisterError> {
    let response = state
        .http
        .post(format!("{}/core/pairing", state.config.account_api_url))
        .json(&json!({
            "code": code,
            "core_id": state.core_id,
            "address": state.config.public_url,
        }))
        .send()
        .await
        .map_err(|error| {
            RegisterError::Other(format!(
                "Amberite could not be reached at {} ({error})",
                state.config.account_api_url
            ))
        })?;
    if response.status().is_success() {
        return Ok(());
    }
    let status = response.status();
    let body: BackendError = response.json().await.unwrap_or_default();
    if status == StatusCode::CONFLICT && body.error == "code_taken" {
        return Err(RegisterError::CodeTaken);
    }
    Err(RegisterError::Other(match body.error.as_str() {
        "already_paired" => "this Core is still listed under an account. Unpair it in the App under Settings, Cores.".into(),
        "invalid_address" => format!(
            "this Core's address {} is not allowed. {}",
            state.config.public_url, body.description
        ),
        _ => format!("Amberite answered {status}. {}", body.description),
    }))
}

/// Pair this Core to `account`, who sent `code` from the App.
pub async fn pair(
    state: &AppState,
    account: &Account,
    code: &str,
) -> Result<(), ApiError> {
    let owner = state.owner().await.map_err(internal)?;
    if let Some(owner) = &owner {
        if owner.user_id != account.id {
            return Err(ApiError::Conflict(
                "this Core is already paired to another account".into(),
            ));
        }
        if owner.backend_key.is_some() {
            return Ok(());
        }
    }

    let now = Instant::now();
    let wrong_guesses = state
        .pairing_guesses
        .get(&account.id)
        .filter(|guesses| guesses.1 + GUESS_WINDOW > now)
        .map_or(0, |guesses| guesses.0);
    if wrong_guesses >= MAX_WRONG_GUESSES {
        return Err(ApiError::TooManyRequests(
            "too many wrong codes, try again in a few minutes".into(),
        ));
    }

    let code: String = code
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect();
    let shown = state.pairing_code.lock().unwrap().clone();
    if !shown.is_some_and(|shown| shown.code == code && shown.expires_at > now)
    {
        state
            .pairing_guesses
            .retain(|_, guesses| guesses.1 + GUESS_WINDOW > now);
        let mut guesses = state
            .pairing_guesses
            .entry(account.id.clone())
            .or_insert((0, now));
        guesses.0 += 1;
        return Err(ApiError::NotFound(
            "that code does not match this Core".into(),
        ));
    }

    // The claimer is not trusted: the backend says whether this account claimed this code.
    let response = state
        .http
        .post(format!(
            "{}/core/pairing/confirm",
            state.config.account_api_url
        ))
        .json(&json!({
            "code": code,
            "core_id": state.core_id,
            "user_id": account.id,
            "name": core_name(),
            "address": state.config.public_url,
        }))
        .send()
        .await
        .map_err(|error| {
            ApiError::ServiceUnavailable(format!(
                "account service unavailable: {error}"
            ))
        })?;
    let status = response.status();
    if !status.is_success() {
        let reason = error_text(response).await;
        return Err(match status {
            StatusCode::FORBIDDEN => ApiError::Forbidden(reason),
            StatusCode::CONFLICT => ApiError::Conflict(reason),
            _ => ApiError::ServiceUnavailable(reason),
        });
    }
    let confirmed: Confirmed = response.json().await.map_err(|error| {
        ApiError::ServiceUnavailable(format!(
            "account service unavailable: {error}"
        ))
    })?;

    state
        .set_owner(&account.id, &confirmed.key)
        .await
        .map_err(internal)?;
    state.pairing_guesses.clear();
    println!("\nPaired to {}.\n", account.username);
    Ok(())
}

#[derive(Deserialize)]
struct Confirmed {
    key: String,
}

#[derive(Default, Deserialize)]
struct BackendError {
    #[serde(default)]
    error: String,
    #[serde(default)]
    description: String,
}

async fn error_text(response: reqwest::Response) -> String {
    let status = response.status();
    let body: BackendError = response.json().await.unwrap_or_default();
    if body.description.is_empty() {
        format!("Amberite answered {status}")
    } else {
        body.description
    }
}

fn internal(error: sqlx::Error) -> ApiError {
    ApiError::Internal(error.to_string())
}

/// Name shown for this Core in the App: the machine's name.
fn core_name() -> String {
    sysinfo::System::host_name()
        .filter(|name| !name.trim().is_empty())
        .map(|name| name.chars().take(64).collect())
        .unwrap_or_else(|| "Core".into())
}

fn generate_code() -> String {
    // Version 4 UUIDs are random except for six fixed bits in bytes 6 and 8.
    Uuid::new_v4()
        .as_bytes()
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 6 && *index != 8)
        .take(8)
        .map(|(_, byte)| ALPHABET[usize::from(byte & 31)] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{generate_code, ALPHABET};

    #[test]
    fn codes_are_eight_characters_from_the_alphabet() {
        let code = generate_code();
        assert_eq!(code.len(), 8);
        assert!(code.bytes().all(|byte| ALPHABET.contains(&byte)));
    }
}
