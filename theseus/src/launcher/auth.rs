//! Authentication flow based on Hydra
use crate::util::fetch::{fetch_advanced, fetch_json, FetchSemaphore};
use async_tungstenite as ws;
use chrono::{prelude::*, Duration};
use futures::prelude::*;
use lazy_static::lazy_static;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use url::Url;

lazy_static! {
    static ref HYDRA_URL: Url = Url::parse("https://hydra.modrinth.com")
        .expect("Hydra URL parse failed");
}

// Socket messages
#[derive(Deserialize)]
struct ErrorJSON {
    error: String,
}

impl ErrorJSON {
    pub fn unwrap<'a, T: Deserialize<'a>>(data: &'a [u8]) -> crate::Result<T> {
        if let Ok(err) = serde_json::from_slice::<Self>(data) {
            Err(crate::ErrorKind::HydraError(err.error).as_error())
        } else {
            Ok(serde_json::from_slice::<T>(data)?)
        }
    }
}

#[derive(Deserialize)]
struct LoginCodeJSON {
    login_code: String,
}

#[derive(Deserialize)]
struct TokenJSON {
    token: String,
    refresh_token: String,
    expires_after: u32,
}

#[derive(Deserialize)]
struct ProfileInfoJSON {
    id: uuid::Uuid,
    name: String,
}

// Login information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Credentials {
    pub id: uuid::Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
    _ctor_scope: std::marker::PhantomData<()>,
}

// Implementation
pub struct HydraAuthFlow<S: AsyncRead + AsyncWrite + Unpin> {
    socket: ws::WebSocketStream<S>,
}

impl HydraAuthFlow<ws::tokio::ConnectStream> {
    pub async fn new() -> crate::Result<Self> {
        let sock_url = wrap_ref_builder!(
            it = HYDRA_URL.clone() =>
            { it.set_scheme("wss").ok() }
        );
        let (socket, _) = ws::tokio::connect_async(sock_url.clone()).await?;
        Ok(Self { socket })
    }

    pub async fn prepare_login_url(&mut self) -> crate::Result<Url> {
        let code_resp = self
            .socket
            .try_next()
            .await?
            .ok_or(
                crate::ErrorKind::WSClosedError(String::from(
                    "login socket ID",
                ))
                .as_error(),
            )?
            .into_data();
        let code = ErrorJSON::unwrap::<LoginCodeJSON>(&code_resp)?;
        Ok(wrap_ref_builder!(
            it = HYDRA_URL.join("login")? =>
            { it.query_pairs_mut().append_pair("id", &code.login_code); }
        ))
    }

    pub async fn extract_credentials(
        &mut self,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<Credentials> {
        // Minecraft bearer token
        let token_resp = self
            .socket
            .try_next()
            .await?
            .ok_or(
                crate::ErrorKind::WSClosedError(String::from(
                    "login socket ID",
                ))
                .as_error(),
            )?
            .into_data();
        let token = ErrorJSON::unwrap::<TokenJSON>(&token_resp)?;
        let expires =
            Utc::now() + Duration::seconds(token.expires_after.into());

        // Get account credentials
        let info = fetch_info(&token.token, semaphore).await?;

        // Return structure from response
        Ok(Credentials {
            username: info.name,
            id: info.id,
            refresh_token: token.refresh_token,
            access_token: token.token,
            expires,
            _ctor_scope: std::marker::PhantomData,
        })
    }
}

pub async fn refresh_credentials(
    credentials: &mut Credentials,
    semaphore: &FetchSemaphore,
) -> crate::Result<()> {
    let resp = fetch_json::<TokenJSON>(
        Method::POST,
        HYDRA_URL.join("/refresh")?.as_str(),
        None,
        Some(serde_json::json!({ "refresh_token": credentials.refresh_token })),
        semaphore,
    )
    .await?;

    credentials.access_token = resp.token;
    credentials.refresh_token = resp.refresh_token;
    credentials.expires =
        Utc::now() + Duration::seconds(resp.expires_after.into());

    Ok(())
}

// Helpers
async fn fetch_info(
    token: &str,
    semaphore: &FetchSemaphore,
) -> crate::Result<ProfileInfoJSON> {
    let result = fetch_advanced(
        Method::GET,
        "https://api.minecraftservices.com/minecraft/profile",
        None,
        None,
        Some(("Authorization", &format!("Bearer {token}"))),
        None,
        semaphore,
    )
    .await?;
    let value = serde_json::from_slice(&result)?;

    Ok(value)
}
