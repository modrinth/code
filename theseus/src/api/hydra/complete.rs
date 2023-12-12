//! Main authentication flow for Hydra

use serde::Deserialize;

use crate::prelude::Credentials;

use super::stages::{
    bearer_token, player_info, poll_response, xbl_signin, xsts_token,
};

#[derive(Debug, Deserialize)]
pub struct OauthFailure {
    pub error: String,
}

pub struct SuccessfulLogin {
    pub name: String,
    pub icon: String,
    pub token: String,
    pub refresh_token: String,
    pub expires_after: i64,
}

pub async fn wait_finish(device_code: String) -> crate::Result<Credentials> {
    // Loop, polling for response from Microsoft
    let oauth = poll_response::poll_response(device_code).await?;

    // Get xbl token from oauth token
    let xbl_token = xbl_signin::login_xbl(&oauth.access_token).await?;

    // Get xsts token from xbl token
    let xsts_response = xsts_token::fetch_token(&xbl_token.token).await?;

    match xsts_response {
        xsts_token::XSTSResponse::Unauthorized(err) => {
            Err(crate::ErrorKind::HydraError(format!(
                "Error getting XBox Live token: {}",
                err
            ))
            .as_error())
        }
        xsts_token::XSTSResponse::Success { token: xsts_token } => {
            // Get xsts bearer token from xsts token
            let (bearer_token, expires_in) =
                bearer_token::fetch_bearer(&xsts_token, &xbl_token.uhs)
                    .await
                    .map_err(|err| {
                        crate::ErrorKind::HydraError(format!(
                            "Error getting bearer token: {}",
                            err
                        ))
                    })?;

            // Get player info from bearer token
            let player_info = player_info::fetch_info(&bearer_token).await.map_err(|_err| {
                crate::ErrorKind::HydraError("No Minecraft account for profile. Make sure you own the game and have set a username through the official Minecraft launcher."
            .to_string())
            })?;

            // Create credentials
            let credentials = Credentials::new(
                uuid::Uuid::parse_str(&player_info.id)?, // get uuid from player_info.id which is a String
                player_info.name,
                bearer_token,
                oauth.refresh_token,
                chrono::Utc::now() + chrono::Duration::seconds(expires_in),
            );

            // Put credentials into state
            let state = crate::State::get().await?;
            {
                let mut users = state.users.write().await;
                users.insert(&credentials).await?;
            }

            if state.settings.read().await.default_user.is_none() {
                let mut settings = state.settings.write().await;
                settings.default_user = Some(credentials.id);
            }

            Ok(credentials)
        }
    }
}
