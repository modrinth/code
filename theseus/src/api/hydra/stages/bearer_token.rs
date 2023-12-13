use serde::Deserialize;
use serde_json::json;

use super::auth_retry;

const MCSERVICES_AUTH_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";

#[derive(Deserialize)]
pub struct BearerTokenResponse {
    access_token: String,
    expires_in: i64,
}

#[tracing::instrument]
pub async fn fetch_bearer(
    token: &str,
    uhs: &str,
) -> crate::Result<(String, i64)> {
    let body = auth_retry(|| {
        let client = reqwest::Client::new();
        client
            .post(MCSERVICES_AUTH_URL)
            .header("Accept", "application/json")
            .json(&json!({
                "identityToken": format!("XBL3.0 x={};{}", uhs, token),
            }))
            .send()
    })
    .await?
    .text()
    .await?;

    serde_json::from_str::<BearerTokenResponse>(&body)
        .map(|x| (x.access_token, x.expires_in))
        .map_err(|_| {
            crate::ErrorKind::HydraError(format!(
                "Response didn't contain valid bearer token. body: {body}"
            ))
            .into()
        })
}
