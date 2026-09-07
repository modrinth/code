use std::time::{Duration, Instant};

use eyre::{WrapErr, eyre};
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::{debug, error, warn};

use xredis::RedisPool;

use crate::env::ENV;
use crate::util::http::HTTP_CLIENT;
use crate::util::neverbounce::email_check_error_generic;

pub const DEFAULT_API_URL: &str = "https://api.usercheck.com";
const TIMEOUT: Duration = Duration::from_secs(5);
const CACHE_NAMESPACE: &str = "usercheck_gate:v1";
const CACHE_EXPIRY_SECONDS: i64 = 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionAction {
    Allow,
    Block,
    Challenge,
    Unrecognized(String),
}

impl DecisionAction {
    fn from_api_value(value: &str) -> Self {
        match value {
            "allow" => Self::Allow,
            "block" => Self::Block,
            "challenge" => Self::Challenge,
            value => Self::Unrecognized(value.to_owned()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            DecisionAction::Allow => "allow",
            DecisionAction::Block => "block",
            DecisionAction::Challenge => "challenge",
            DecisionAction::Unrecognized(other) => other,
        }
    }
}

impl<'de> Deserialize<'de> for DecisionAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_api_value(&String::deserialize(deserializer)?))
    }
}

#[derive(Deserialize)]
struct DecisionResponse {
    decision: Decision,
    #[serde(default)]
    meta: Option<ResponseMeta>,
}

#[derive(Deserialize)]
struct Decision {
    action: DecisionAction,
    #[serde(default)]
    matched_rule: Option<MatchedRule>,
}

#[derive(Deserialize)]
struct MatchedRule {
    id: String,
    name: String,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct ResponseMeta {
    #[serde(default)]
    request_id: Option<String>,
}

#[derive(Serialize)]
struct DecisionRequest<'a> {
    email: &'a str,
}

/// Asks the configured UserCheck gate whether a signup should proceed.
///
/// Failure handling mirrors [`crate::util::neverbounce::check_email`]: a
/// transient failure resolves to `Allow` so an outage cannot block every
/// signup, while anything else is an error that rejects the signup.
/// `Challenge` resolves to `Allow` because these flows have no step-up
/// mechanism past the captcha that already ran.
///
/// Verdicts are cached in Redis for an hour, keyed by email address.
pub async fn check_email_gate(
    redis: &RedisPool,
    email: &str,
) -> eyre::Result<DecisionAction> {
    if ENV.USERCHECK_API_KEY.is_empty() || ENV.USERCHECK_GATE_ID.is_empty() {
        debug!(
            action = "allow",
            "UserCheck gate skipped because the API key or gate ID is not set",
        );
        return Ok(DecisionAction::Allow);
    }

    let cache_key = {
        let mut redis = redis.connect().await?;
        let key = redis
            .key()
            .entity(CACHE_NAMESPACE, email.to_ascii_lowercase());

        if let Some(cached) = redis.get(&key).await? {
            let action = DecisionAction::from_api_value(&cached);

            debug!(
                action = action.as_str(),
                "UserCheck gate decision served from cache",
            );

            return Ok(action);
        }

        key
    };

    let decision_time_start = Instant::now();
    let response = request_decision(email).await;
    let decision_time = decision_time_start.elapsed();

    let response = match response {
        Ok(response) => response,
        Err(source) => {
            let is_transient = is_transient(&source);

            error!(
                action = if is_transient { "allow" } else { "block" },
                request.transient = is_transient,
                request.time_ms = decision_time.as_millis(),
                error = ?source,
                "UserCheck gate decision failed",
            );

            if is_transient {
                return Ok(DecisionAction::Allow);
            }

            return Err(eyre!(source)).wrap_err("failed to check email");
        }
    };

    let DecisionResponse { decision, meta } = response;
    let Decision {
        action,
        matched_rule,
    } = decision;

    let rule_id = matched_rule.as_ref().map(|rule| rule.id.as_str());
    let rule_name = matched_rule.as_ref().map(|rule| rule.name.as_str());
    let rule_message = matched_rule
        .as_ref()
        .and_then(|rule| rule.message.as_deref());
    let request_id = meta.and_then(|meta| meta.request_id);
    let time_ms = decision_time.as_millis();

    match action {
        DecisionAction::Unrecognized(ref value) => {
            error!(
                action = value.as_str(),
                rule.id = rule_id,
                rule.name = rule_name,
                rule.message = rule_message,
                request.id = request_id,
                request.time_ms = time_ms,
                "UserCheck gate returned an unrecognized action",
            );
            return Err(email_check_error_generic());
        }
        DecisionAction::Challenge => warn!(
            action = action.as_str(),
            rule.id = rule_id,
            rule.name = rule_name,
            rule.message = rule_message,
            request.id = request_id,
            request.time_ms = time_ms,
            "UserCheck gate returned a challenge, allowing",
        ),
        _ => debug!(
            action = action.as_str(),
            rule.id = rule_id,
            rule.name = rule_name,
            rule.message = rule_message,
            request.id = request_id,
            request.time_ms = time_ms,
            "UserCheck gate decision succeeded",
        ),
    }

    redis
        .connect()
        .await?
        .set(&cache_key, action.as_str(), Some(CACHE_EXPIRY_SECONDS))
        .await?;

    Ok(action)
}

pub fn gate_block_error() -> eyre::Error {
    eyre!(
        "Please try a different email address, or turn off any VPN or proxy services!"
    )
}

async fn request_decision(email: &str) -> reqwest::Result<DecisionResponse> {
    HTTP_CLIENT
        .post(format!(
            "{}/v0/gates/{}/decisions",
            ENV.USERCHECK_BASE_URL.trim_end_matches('/'),
            ENV.USERCHECK_GATE_ID,
        ))
        .bearer_auth(&ENV.USERCHECK_API_KEY)
        .timeout(TIMEOUT)
        .json(&DecisionRequest { email })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

fn is_transient(error: &reqwest::Error) -> bool {
    if let Some(status) = error.status() {
        return status.is_server_error()
            || matches!(
                status,
                StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS
            );
    }

    error.is_timeout()
        || error.is_connect()
        || error.is_request()
        || error.is_body()
}
