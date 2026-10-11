//! Account login for Core HTTP routes.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
};
use serde::Deserialize;

use crate::{application::state::AppState, presentation::error::ApiError};

/// How long the backend's answer for an account token is reused.
const TOKEN_CACHE: Duration = Duration::from_secs(60);

/// Axum extractor that accepts only the Core owner's account token.
pub struct AuthUser(pub Account);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                ApiError::Unauthorized("missing Authorization header".into())
            })?;
        let account = account(state, token).await?;

        // An unowned Core refuses everything except pairing.
        let owner = state
            .owner()
            .await
            .map_err(|error| ApiError::Internal(error.to_string()))?
            .ok_or_else(|| {
                ApiError::Forbidden("this Core is not paired".into())
            })?;
        if owner.user_id != account.id {
            return Err(ApiError::Forbidden(
                "this Core belongs to another account".into(),
            ));
        }

        Ok(Self(account))
    }
}

pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let val = headers.get("authorization")?.to_str().ok()?;
    val.strip_prefix("Bearer ")
}

/// The signed-in account making a request.
#[derive(Clone, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
}

/// Resolve an account token to its account by asking the backend (`GET /user`).
pub async fn account(
    state: &AppState,
    token: &str,
) -> Result<Account, ApiError> {
    let now = Instant::now();
    if let Some(cached) = state.account_tokens.get(token) {
        if cached.1 > now {
            return Ok(cached.0.clone());
        }
    }

    let unavailable = |error: reqwest::Error| {
        ApiError::ServiceUnavailable(format!(
            "account service unavailable: {error}"
        ))
    };
    let response = state
        .http
        .get(format!("{}/user", state.config.account_api_url))
        .bearer_auth(token)
        .send()
        .await
        .map_err(unavailable)?;
    if response.status() == StatusCode::UNAUTHORIZED {
        return Err(ApiError::Unauthorized("invalid account token".into()));
    }
    let account: Account = response
        .error_for_status()
        .map_err(unavailable)?
        .json()
        .await
        .map_err(unavailable)?;

    state.account_tokens.retain(|_, cached| cached.1 > now);
    state
        .account_tokens
        .insert(token.to_string(), (account.clone(), now + TOKEN_CACHE));
    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::bearer_token;
    use axum::http::{header, HeaderMap, HeaderValue};

    fn with_auth(value: &str) -> HeaderMap {
        let mut m = HeaderMap::new();
        m.insert(header::AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        m
    }

    #[test]
    fn valid_bearer_token() {
        let headers = with_auth("Bearer abc123");
        assert_eq!(bearer_token(&headers), Some("abc123"));
    }

    #[test]
    fn missing_authorization_header() {
        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }

    #[test]
    fn wrong_scheme_returns_none() {
        let headers = with_auth("Basic abc123");
        assert_eq!(bearer_token(&headers), None);
    }
}
