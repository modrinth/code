//! MSA authentication stages

use futures::Future;
use reqwest::Response;

const RETRY_COUNT: usize = 9; // Does command 3 times
const RETRY_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

pub mod bearer_token;
pub mod player_info;
pub mod poll_response;
pub mod xbl_signin;
pub mod xsts_token;

#[tracing::instrument(skip(reqwest_request))]
pub async fn auth_retry<F>(
    reqwest_request: impl Fn() -> F,
) -> crate::Result<reqwest::Response>
where
    F: Future<Output = Result<Response, reqwest::Error>>,
{
    let mut resp = reqwest_request().await?;
    for i in 0..RETRY_COUNT {
        if resp.status().is_success() {
            break;
        }
        tracing::debug!(
            "Request failed with status code {}, retrying...",
            resp.status()
        );
        if i < RETRY_COUNT - 1 {
            tokio::time::sleep(RETRY_WAIT).await;
        }
        resp = reqwest_request().await?;
    }
    Ok(resp)
}
