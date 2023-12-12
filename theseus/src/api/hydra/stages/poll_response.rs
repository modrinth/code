use std::collections::HashMap;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    hydra::{MicrosoftError, MICROSOFT_CLIENT_ID},
    util::fetch::REQWEST_CLIENT,
};

use super::auth_retry;

#[derive(Debug, Deserialize)]
pub struct OauthSuccess {
    pub token_type: String,
    pub scope: String,
    pub expires_in: i64,
    pub access_token: String,
    pub refresh_token: String,
}

#[tracing::instrument]
pub async fn poll_response(device_code: String) -> crate::Result<OauthSuccess> {
    let mut params = HashMap::new();
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
    params.insert("client_id", MICROSOFT_CLIENT_ID);
    params.insert("device_code", &device_code);
    params.insert(
        "scope",
        "XboxLive.signin XboxLive.offline_access profile openid email",
    );

    // Poll the URL in a loop until we are successful.
    // On an authorization_pending response, wait 5 seconds and try again.
    loop {
        let resp = auth_retry(|| {
            REQWEST_CLIENT
            .post(
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/token",
            )
            .form(&params)
            .send()
        })
        .await?;

        match resp.status() {
            StatusCode::OK => {
                let oauth =
                    resp.json::<OauthSuccess>().await.map_err(|err| {
                        crate::ErrorKind::HydraError(format!(
                            "Could not decipher successful response: {}",
                            err
                        ))
                    })?;
                return Ok(oauth);
            }
            _ => {
                let failure =
                    resp.json::<MicrosoftError>().await.map_err(|err| {
                        crate::ErrorKind::HydraError(format!(
                            "Could not decipher failure response: {}",
                            err
                        ))
                    })?;
                match failure.error.as_str() {
                    "authorization_pending" => {
                        tokio::time::sleep(std::time::Duration::from_secs(2))
                            .await;
                    }
                    "authorization_declined" => {
                        return Err(crate::ErrorKind::HydraError(
                            "Authorization declined".to_string(),
                        )
                        .as_error());
                    }
                    "expired_token" => {
                        return Err(crate::ErrorKind::HydraError(
                            "Device code expired".to_string(),
                        )
                        .as_error());
                    }
                    "bad_verification_code" => {
                        return Err(crate::ErrorKind::HydraError(
                            "Invalid device code".to_string(),
                        )
                        .as_error());
                    }
                    _ => {
                        return Err(crate::ErrorKind::HydraError(format!(
                            "Unknown error: {}",
                            failure.error
                        ))
                        .as_error());
                    }
                }
            }
        }
    }
}
