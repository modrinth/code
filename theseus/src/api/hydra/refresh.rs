use std::collections::HashMap;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    hydra::{MicrosoftError, MICROSOFT_CLIENT_ID},
    util::fetch::REQWEST_CLIENT,
};

use super::stages::auth_retry;

#[derive(Debug, Deserialize)]
pub struct OauthSuccess {
    pub token_type: String,
    pub scope: String,
    pub expires_in: i64,
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn refresh(refresh_token: String) -> crate::Result<OauthSuccess> {
    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("client_id", MICROSOFT_CLIENT_ID);
    params.insert("refresh_token", &refresh_token);
    params.insert(
        "redirect_uri",
        "https://login.microsoftonline.com/common/oauth2/nativeclient",
    );

    // Poll the URL in a loop until we are successful.
    // On an authorization_pending response, wait 5 seconds and try again.
    let resp =
        auth_retry(|| {
            REQWEST_CLIENT
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        })
        .await?;

    match resp.status() {
        StatusCode::OK => {
            let oauth = resp.json::<OauthSuccess>().await.map_err(|err| {
                crate::ErrorKind::HydraError(format!(
                    "Could not decipher successful response: {}",
                    err
                ))
            })?;
            Ok(oauth)
        }
        _ => {
            let failure =
                resp.json::<MicrosoftError>().await.map_err(|err| {
                    crate::ErrorKind::HydraError(format!(
                        "Could not decipher failure response: {}",
                        err
                    ))
                })?;
            Err(crate::ErrorKind::HydraError(format!(
                "Error refreshing token: {}",
                failure.error
            ))
            .as_error())
        }
    }
}
