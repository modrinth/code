use std::time::Instant;

use eyre::{WrapErr, eyre};
use neverbounce::{
    ReqwestErrorReason, ResponseStatus, SingleCheckParams, SingleCheckResponse,
    VerificationResult,
};
use tracing::{debug, error};
use xredis::RedisPool;

use crate::env::ENV;
use crate::util::http::HTTP_CLIENT;

const CACHE_NAMESPACE: &str = "neverbounce:v1";
const CACHE_EXPIRY_SECONDS: i64 = 60 * 60;

/// Verdicts are cached in Redis for an hour, keyed by address. Only verdicts
/// NeverBounce actually returned are cached; the `Unknown` we fall back to when
/// the API is unreachable is not, so an outage cannot pin an address for an
/// hour.
pub async fn check_email(
    redis: &RedisPool,
    email: &str,
) -> eyre::Result<VerificationResult> {
    if ENV.NEVERBOUNCE_API_KEY.is_empty() {
        debug!(
            result = "unknown",
            "NeverBounce email check skipped because API key is not set",
        );
        return Ok(VerificationResult::Unknown);
    }

    let cache_key = {
        let mut redis = redis.connect().await?;
        let key = redis
            .key()
            .entity(CACHE_NAMESPACE, email.to_ascii_lowercase());

        if let Some(cached) = redis.get(&key).await? {
            let result = VerificationResult::from_api_value(&cached);

            debug!(
                result = result.as_str(),
                "NeverBounce email check served from cache",
            );

            return Ok(result);
        }

        key
    };

    let params = SingleCheckParams::new(&ENV.NEVERBOUNCE_API_KEY, email)
        .with_api_url(&ENV.NEVERBOUNCE_BASE_URL);

    let check_time_start = Instant::now();

    let response = match neverbounce::single_check(&HTTP_CLIENT, &params).await
    {
        Ok(response) => response,
        Err(source) => {
            let reason = ReqwestErrorReason::from(&source);
            let is_transient = reason.is_transient();

            error!(
                result = "unknown",
                request.error_reason = ?reason,
                request.time_ms = check_time_start.elapsed().as_millis(),
                error = ?source,
                "NeverBounce email check failed",
            );

            if is_transient {
                return Ok(VerificationResult::Unknown);
            }

            return Err(eyre!(source)).wrap_err("failed to check email");
        }
    };

    let SingleCheckResponse { status, result, .. } = response;

    let check_time = check_time_start.elapsed();

    match status {
        ResponseStatus::Success => {
            let result = result.ok_or_else(|| {
                error!(result = "unknown", "NeverBounce email check failed",);
                eyre!("")
            })?;

            if matches!(result, VerificationResult::Unrecognized(_)) {
                error!(
                    result = result.as_str(),
                    request.time_ms = check_time.as_millis(),
                    "NeverBounce email check failed",
                );
                return Err(email_check_error_generic());
            }

            debug!(
                result = result.as_str(),
                request.time_ms = check_time.as_millis(),
                "NeverBounce email check succeeded",
            );

            redis
                .connect()
                .await?
                .set(&cache_key, result.as_str(), Some(CACHE_EXPIRY_SECONDS))
                .await?;

            Ok(result)
        }
        failure_type => {
            let result = result.unwrap_or(VerificationResult::Unknown);
            let is_transient = failure_type.is_transient();
            error!(
                failure_type = response_failure_type(&failure_type),
                result = result.as_str(),
                request.time_ms = check_time.as_millis(),
                "NeverBounce email check failed",
            );

            if is_transient {
                Ok(VerificationResult::Unknown)
            } else {
                Err(email_check_error_generic())
            }
        }
    }
}

pub fn email_check_error_generic() -> eyre::Error {
    eyre!("Please try a different email address!")
}

fn response_failure_type(status: &ResponseStatus) -> &str {
    match status {
        ResponseStatus::Success => "success",
        ResponseStatus::GeneralFailure => "general_failure",
        ResponseStatus::AuthFailure => "auth_failure",
        ResponseStatus::TemporarilyUnavailable => "temp_unavail",
        ResponseStatus::ThrottleTriggered => "throttle_triggered",
        ResponseStatus::BadReferrer => "bad_referrer",
        ResponseStatus::Unrecognized(status) => status,
        _ => "unrecognized",
    }
}
