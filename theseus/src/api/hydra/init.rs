//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use serde::{Deserialize, Serialize};

use crate::{hydra::MicrosoftError, util::fetch::REQWEST_CLIENT};

use super::{stages::auth_retry, MICROSOFT_CLIENT_ID};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceLoginSuccess {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

pub async fn init() -> crate::Result<DeviceLoginSuccess> {
    // Get the initial URL
    // Get device code
    // Define the parameters

    // urlencoding::encode("XboxLive.signin offline_access"));
    let resp = auth_retry(|| REQWEST_CLIENT.get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .header("Content-Length", "0")
        .query(&[
            ("client_id", MICROSOFT_CLIENT_ID),
            (
                "scope",
                "XboxLive.signin XboxLive.offline_access profile openid email",
            ),
        ])
        .send()
    ).await?;

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.json().await?),
        _ => {
            let microsoft_error = resp.json::<MicrosoftError>().await?;
            Err(crate::ErrorKind::HydraError(format!(
                "Error from Microsoft: {:?}",
                microsoft_error.error_description
            ))
            .into())
        }
    }
}
