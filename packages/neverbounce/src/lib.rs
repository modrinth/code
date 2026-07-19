use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;

pub const DEFAULT_API_URL: &str = "https://api.neverbounce.com";
pub const SINGLE_CHECK_PATH: &str = "/v4/single/check";
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// Authentication and email parameters for a single verification.
///
/// The API key is sent in the JSON request body.
#[derive(Clone, Copy)]
pub struct SingleCheckParams<'a> {
    pub api_url: &'a str,
    pub api_key: &'a str,
    pub email: &'a str,
}

impl<'a> SingleCheckParams<'a> {
    #[must_use]
    pub const fn new(api_key: &'a str, email: &'a str) -> Self {
        Self {
            api_url: DEFAULT_API_URL,
            api_key,
            email,
        }
    }

    #[must_use]
    pub const fn with_api_url(mut self, api_url: &'a str) -> Self {
        self.api_url = api_url;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct SingleCheckResponse {
    pub status: ResponseStatus,
    #[serde(default)]
    pub result: Option<VerificationResult>,
    #[serde(default)]
    pub flags: Vec<VerificationFlag>,
    #[serde(default)]
    pub suggested_correction: Option<String>,
    #[serde(default)]
    pub retry_token: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub execution_time: Option<u64>,
}

impl SingleCheckResponse {
    #[must_use]
    pub fn is_safe_to_send(&self) -> bool {
        matches!(self.status, ResponseStatus::Success)
            && matches!(self.result.as_ref(), Some(VerificationResult::Valid))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResponseStatus {
    Success,
    GeneralFailure,
    AuthFailure,
    TemporarilyUnavailable,
    ThrottleTriggered,
    BadReferrer,
    Unrecognized(String),
}

impl<'de> Deserialize<'de> for ResponseStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "success" => Self::Success,
            "general_failure" => Self::GeneralFailure,
            "auth_failure" => Self::AuthFailure,
            "temp_unavail" => Self::TemporarilyUnavailable,
            "throttle_triggered" => Self::ThrottleTriggered,
            "bad_referrer" => Self::BadReferrer,
            value => Self::Unrecognized(value.to_owned()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VerificationResult {
    Valid,
    Invalid,
    Disposable,
    CatchAll,
    Unknown,
    Unrecognized(String),
}

impl VerificationResult {
    pub fn as_str(&self) -> &str {
        match self {
            VerificationResult::Valid => "valid",
            VerificationResult::Invalid => "invalid",
            VerificationResult::Disposable => "disposable",
            VerificationResult::CatchAll => "catchall",
            VerificationResult::Unknown => "unknown",
            VerificationResult::Unrecognized(other) => other,
        }
    }
}

impl<'de> Deserialize<'de> for VerificationResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "valid" => Self::Valid,
            "invalid" => Self::Invalid,
            "disposable" => Self::Disposable,
            "catchall" => Self::CatchAll,
            "unknown" => Self::Unknown,
            value => Self::Unrecognized(value.to_owned()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VerificationFlag {
    HasDns,
    HasDnsMx,
    BadSyntax,
    FreeEmailHost,
    Profanity,
    RoleAccount,
    DisposableEmail,
    GovernmentHost,
    AcademicHost,
    MilitaryHost,
    InternationalHost,
    SquatterHost,
    SpellingMistake,
    BadDns,
    TemporaryDnsError,
    ConnectFails,
    AcceptsAll,
    ContainsAlias,
    ContainsSubdomain,
    SmtpConnectable,
    SpamtrapNetwork,
    HistoricalResponse,
    Unrecognized(String),
}

impl<'de> Deserialize<'de> for VerificationFlag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "has_dns" => Self::HasDns,
            "has_dns_mx" => Self::HasDnsMx,
            "bad_syntax" => Self::BadSyntax,
            "free_email_host" => Self::FreeEmailHost,
            "profanity" => Self::Profanity,
            "role_account" => Self::RoleAccount,
            "disposable_email" => Self::DisposableEmail,
            "government_host" => Self::GovernmentHost,
            "academic_host" => Self::AcademicHost,
            "military_host" => Self::MilitaryHost,
            "international_host" => Self::InternationalHost,
            "squatter_host" => Self::SquatterHost,
            "spelling_mistake" => Self::SpellingMistake,
            "bad_dns" => Self::BadDns,
            "temporary_dns_error" => Self::TemporaryDnsError,
            "connect_fails" => Self::ConnectFails,
            "accepts_all" => Self::AcceptsAll,
            "contains_alias" => Self::ContainsAlias,
            "contains_subdomain" => Self::ContainsSubdomain,
            "smtp_connectable" => Self::SmtpConnectable,
            "spamtrap_network" => Self::SpamtrapNetwork,
            "historical_response" => Self::HistoricalResponse,
            value => Self::Unrecognized(value.to_owned()),
        })
    }
}

#[derive(Serialize)]
struct SingleCheckRequest<'a> {
    key: &'a str,
    email: &'a str,
    timeout: u64,
}

/// Verifies one email address using NeverBounce's single-check endpoint.
///
/// This endpoint should only be called in response to an action such as a form
/// submission. Existing lists and databases must use NeverBounce's bulk API.
/// Both the server verification timeout and the complete HTTP request timeout
/// are ten seconds.
pub async fn single_check(
    client: &Client,
    params: &SingleCheckParams<'_>,
) -> reqwest::Result<SingleCheckResponse> {
    single_check_request(client, params)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

fn single_check_request(
    client: &Client,
    params: &SingleCheckParams<'_>,
) -> reqwest::RequestBuilder {
    client
        .post(format!(
            "{}{SINGLE_CHECK_PATH}",
            params.api_url.trim_end_matches('/')
        ))
        .timeout(TIMEOUT)
        .json(&SingleCheckRequest {
            key: params.api_key,
            email: params.email,
            timeout: TIMEOUT.as_secs(),
        })
}
