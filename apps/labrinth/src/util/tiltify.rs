use std::time::{Duration, Instant};

use eyre::eyre;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    env::ENV,
    util::{error::Context, http::HttpClient},
};

#[derive(Debug)]
pub struct TiltifyClient {
    http: HttpClient,
    token: Mutex<Option<TiltifyAccessToken>>,
}

#[derive(Debug)]
struct TiltifyAccessToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Debug, Deserialize)]
struct TiltifyTokenResponse {
    access_token: String,
    expires_in: u64,
}

impl TiltifyClient {
    pub fn new(http: HttpClient) -> Self {
        Self {
            http,
            token: Mutex::new(None),
        }
    }

    pub async fn access_token(&self) -> eyre::Result<String> {
        let mut token = self.token.lock().await;

        if let Some(token) = token.as_ref()
            && token.expires_at > Instant::now()
        {
            return Ok(token.access_token.clone());
        }

        let response = self.fetch_access_token().await?;
        let expires_at = Instant::now()
            + Duration::from_secs(response.expires_in)
                .saturating_sub(Duration::from_secs(60));
        let access_token = response.access_token;

        *token = Some(TiltifyAccessToken {
            access_token: access_token.clone(),
            expires_at,
        });

        Ok(access_token)
    }

    async fn fetch_access_token(&self) -> eyre::Result<TiltifyTokenResponse> {
        if ENV.TILTIFY_CLIENT_ID.is_empty()
            || ENV.TILTIFY_CLIENT_SECRET.is_empty()
        {
            return Err(eyre!(
                "TILTIFY_CLIENT_ID and TILTIFY_CLIENT_SECRET must be set"
            ));
        }

        let response = self
            .http
            .post("https://v5api.tiltify.com/oauth/token")
            .json(&json!({
                "grant_type": "client_credentials",
                "client_id": &ENV.TILTIFY_CLIENT_ID,
                "client_secret": &ENV.TILTIFY_CLIENT_SECRET,
                "scope": "public",
            }))
            .send()
            .await
            .wrap_err("fetching OAuth token")?
            .error_for_status()
            .wrap_err("fetching OAuth token")?
            .json::<TiltifyTokenResponse>()
            .await
            .wrap_err("parsing OAuth token response")?;

        Ok(response)
    }
}
