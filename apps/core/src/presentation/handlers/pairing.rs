//! Pairing routes. `POST /pairing` is the only route an unowned Core answers.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    application::{pairing_service, state::AppState},
    presentation::{
        error::ApiError,
        extractors::{account, bearer_token, AuthUser},
    },
};

#[derive(Deserialize)]
pub struct PairRequest {
    code: String,
}

/// The App sends the code its user typed, with their account token.
pub async fn pair(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PairRequest>,
) -> Result<Json<Value>, ApiError> {
    let token = bearer_token(&headers)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| {
            ApiError::Unauthorized("missing Authorization header".into())
        })?;
    let account = account(&state, token).await?;
    pairing_service::pair(&state, &account, &request.code).await?;
    Ok(Json(json!({ "core_id": state.core_id })))
}

/// The owner unpairs this Core. It keeps its servers and files and shows a new code.
pub async fn unpair(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    state
        .forget_owner()
        .await
        .map_err(|error| ApiError::Internal(error.to_string()))?;
    state.pairing_changed.notify_one();
    Ok(StatusCode::NO_CONTENT)
}
