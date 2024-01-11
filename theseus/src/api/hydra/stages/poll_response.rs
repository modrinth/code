use std::collections::HashMap;

use reqwest::StatusCode;
use serde::Deserialize;
use tiny_http::Response;

use crate::{
    hydra::{
        MicrosoftError, MICROSOFT_CLIENT_ID, MICROSOFT_CLIENT_SECRET,
        REDIRECT_URL, REQUESTED_SCOPES,
    },
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
pub async fn poll_response() -> crate::Result<OauthSuccess> {
    let state = crate::state::State::get().await?;
    let server = state.auth_flow.http_server.read().await;

    loop {
        // blocks until the next request is received
        let request = match server
            .as_ref()
            .ok_or_else(|| {
                crate::ErrorKind::HydraError(
                    "Could not aqquire HTTP server".to_string(),
                )
            })?
            .recv()
        {
            Ok(rq) => rq,
            Err(e) => {
                tracing::warn!("server request error: {}", e);
                continue;
            }
        };

        let url = match url::Url::parse(&format!(
            "http://localhost:20123{}",
            request.url()
        )) {
            Ok(val) => val,
            Err(err) => {
                tracing::warn!("error parsing uri: {err}");
                continue;
            }
        };

        if url.path() != "/theseus/callback" {
            tracing::warn!("wrong URI path: {}", url.path());

            continue;
        }

        let query = url
            .query_pairs()
            .collect::<HashMap<std::borrow::Cow<str>, std::borrow::Cow<str>>>();

        let code = match query.get("code") {
            Some(val) => val,
            None => {
                tracing::warn!("missing response code");
                continue;
            }
        };

        let mut map = HashMap::new();
        map.insert("client_id", MICROSOFT_CLIENT_ID);
        map.insert("client_secret", MICROSOFT_CLIENT_SECRET);
        map.insert("code", &**code);
        map.insert("grant_type", "authorization_code");
        map.insert("redirect_uri", REDIRECT_URL);
        map.insert("scope", REQUESTED_SCOPES);

        let resp = auth_retry(|| {
            REQWEST_CLIENT
                .post("https://login.live.com/oauth20_token.srf")
                .header(reqwest::header::ACCEPT, "application/json")
                .form(&map)
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

                let response = Response::from_string("Microsoft login succeeded. Loading your Minecraft account...");
                request.respond(response).ok();

                return Ok(oauth);
            }
            _ => {
                let response = Response::from_string(
                    "Authentication failed. Please try again.",
                );
                request.respond(response).ok();

                let failure =
                    resp.json::<MicrosoftError>().await.map_err(|err| {
                        crate::ErrorKind::HydraError(format!(
                            "Could not decipher failure response: {}",
                            err
                        ))
                    })?;
                match failure.error.as_str() {
                    "authorization_declined" => {
                        return Err(crate::ErrorKind::HydraError(
                            "Authorization declined".to_string(),
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
