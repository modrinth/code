//! Authentication flow based on Hydra
use async_tungstenite as ws;
use bincode::{Decode, Encode};
use chrono::{prelude::*, Duration};
use futures::prelude::*;
use once_cell::sync::*;
use serde::Deserialize;
use url::Url;

pub const HYDRA_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://hydra.modrinth.com").unwrap());

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
#[derive(Encode, Decode)]
pub struct Credentials {
    #[bincode(with_serde)]
    pub id: uuid::Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    #[bincode(with_serde)]
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
            it = HYDRA_URL =>
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

    pub async fn extract_credentials(&mut self) -> crate::Result<Credentials> {
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
        let info = fetch_info(&token.token).await?;

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
) -> crate::Result<()> {
    let resp = crate::config::REQWEST_CLIENT
        .post(HYDRA_URL.join("/refresh")?)
        .json(
            &serde_json::json!({ "refresh_token": credentials.refresh_token }),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<TokenJSON>()
        .await?;

    credentials.access_token = resp.token;
    credentials.refresh_token = resp.refresh_token;
    credentials.expires =
        Utc::now() + Duration::seconds(resp.expires_after.into());

    Ok(())
}

pub async fn refresh_username(
    credentials: &mut Credentials,
) -> crate::Result<()> {
    let info = fetch_info(&credentials.access_token).await?;
    credentials.username = info.name;
    Ok(())
}

// Helpers
async fn fetch_info(token: &str) -> crate::Result<ProfileInfoJSON> {
    let url =
        Url::parse("https://api.minecraftservices.com/minecraft/profile")?;
    Ok(crate::config::REQWEST_CLIENT
        .get(url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .error_for_status()?
        .json::<ProfileInfoJSON>()
        .await?)
}
