use std::time::{Duration, Instant};

use eyre::{Result, eyre};
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
    state: Mutex<TiltifyState>,
}

#[derive(Debug)]
struct TiltifyState {
    token: Option<TiltifyAccessToken>,
    rate_limited_until: Instant,
    rate_limit_backoff: Duration,
}

impl TiltifyState {
    fn set_fetch_backoff(&mut self) {
        self.rate_limited_until = Instant::now() + self.rate_limit_backoff;
        self.rate_limit_backoff = self
            .rate_limit_backoff
            .saturating_mul(2)
            .min(TILTIFY_MAX_RATE_LIMIT_BACKOFF);
    }
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

const TILTIFY_INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(60);
const TILTIFY_MAX_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(15 * 60);

impl TiltifyClient {
    pub fn new(http: HttpClient) -> Self {
        Self {
            http,
            state: Mutex::new(TiltifyState {
                token: None,
                rate_limited_until: Instant::now(),
                rate_limit_backoff: TILTIFY_INITIAL_RATE_LIMIT_BACKOFF,
            }),
        }
    }

    pub async fn access_token(&self) -> Result<String> {
        let mut state = self.state.lock().await;

        if let Some(token) = state.token.as_ref()
            && token.expires_at > Instant::now()
        {
            return Ok(token.access_token.clone());
        }

        if state.rate_limited_until > Instant::now() {
            return Err(eyre!(
                "waiting for rate limit to reset at {:.0?} (backoff {:.0?})",
                state.rate_limited_until,
                state.rate_limit_backoff
            ));
        }

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
            .inspect_err(|_| state.set_fetch_backoff())
            .wrap_err("fetching OAuth token")?;

        let response = match response.error_for_status() {
            Ok(response) => response,
            Err(error) => {
                state.set_fetch_backoff();
                return Err(error).wrap_err("fetching OAuth token");
            }
        };

        let response = response
            .json::<TiltifyTokenResponse>()
            .await
            .wrap_err("parsing OAuth token response")?;

        let expires_at = Instant::now()
            + Duration::from_secs(response.expires_in)
                .saturating_sub(Duration::from_secs(60));
        let access_token = response.access_token;

        state.token = Some(TiltifyAccessToken {
            access_token: access_token.clone(),
            expires_at,
        });
        state.rate_limit_backoff = TILTIFY_INITIAL_RATE_LIMIT_BACKOFF;

        Ok(access_token)
    }
}
