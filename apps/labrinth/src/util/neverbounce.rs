use std::time::Instant;

use eyre::{WrapErr, eyre};
use neverbounce::{
    ResponseStatus, SingleCheckParams, SingleCheckResponse, VerificationResult,
};
use tracing::{debug, error};

use crate::env::ENV;
use crate::util::http::HTTP_CLIENT;

pub async fn check_email(email: &str) -> eyre::Result<VerificationResult> {
    if ENV.NEVERBOUNCE_API_KEY.is_empty() {
        debug!(
            result = "unknown",
            "NeverBounce email check skipped because API key is not set",
        );
        return Ok(VerificationResult::Unknown);
    }

    let params = SingleCheckParams::new(&ENV.NEVERBOUNCE_API_KEY, email)
        .with_api_url(&ENV.NEVERBOUNCE_BASE_URL);

    let check_time_start = Instant::now();

    let response = match neverbounce::single_check(&HTTP_CLIENT, &params).await
    {
        Ok(response) => response,
        Err(source) => {
            error!(
                result = "unknown",
                error = ?source,
                "NeverBounce email check failed",
            );
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
            Ok(result)
        }
        failure_type => {
            let result = result.unwrap_or(VerificationResult::Unknown);
            error!(
                failure_type = response_failure_type(&failure_type),
                result = result.as_str(),
                request.time_ms = check_time.as_millis(),
                "NeverBounce email check failed",
            );
            Err(email_check_error_generic())
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
