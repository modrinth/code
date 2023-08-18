//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{hydra::MicrosoftError, util::fetch::REQWEST_CLIENT};

use super::MICROSOFT_CLIENT_ID;

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
    let client_id = MICROSOFT_CLIENT_ID;

    // Get device code
    // Define the parameters
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("scope", "XboxLive.signin offline_access");

    // urlencoding::encode("XboxLive.signin offline_access"));
    let req = REQWEST_CLIENT.post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
    .header("Content-Type", "application/x-www-form-urlencoded").form(&params).send().await?;

    match req.status() {
        reqwest::StatusCode::OK => Ok(req.json().await?),
        _ => {
            let microsoft_error = req.json::<MicrosoftError>().await?;
            Err(crate::ErrorKind::HydraError(format!(
                "Error from Microsoft: {:?}",
                microsoft_error.error_description
            ))
            .into())
        }
    }
}
