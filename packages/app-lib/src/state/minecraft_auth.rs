use crate::data::DirectoryInfo;
use crate::util::fetch::{read_json, write, IoSemaphore, REQWEST_CLIENT};
use crate::{ErrorKind, State};
use base64::prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD};
use base64::Engine;
use byteorder::BigEndian;
use chrono::{DateTime, Duration, Utc};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding};
use rand::rngs::OsRng;
use rand::Rng;
use reqwest::header::HeaderMap;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use std::collections::HashMap;
use std::future::Future;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum MinecraftAuthStep {
    GetDeviceToken,
    SisuAuthenicate,
    GetOAuthToken,
    RefreshOAuthToken,
    SisuAuthorize,
    XstsAuthorize,
    MinecraftToken,
    MinecraftEntitlements,
    MinecraftProfile,
}

#[derive(thiserror::Error, Debug)]
pub enum MinecraftAuthenticationError {
    #[error("Error reading public key during generation")]
    ReadingPublicKey,
    #[error("Failed to serialize private key to PEM: {0}")]
    PEMSerialize(#[from] p256::pkcs8::Error),
    #[error("Failed to serialize body to JSON during step {step:?}: {source}")]
    SerializeBody {
        step: MinecraftAuthStep,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "Failed to deserialize response to JSON during step {step:?}: {source}. Status Code: {status_code} Body: {raw}"
    )]
    DeserializeResponse {
        step: MinecraftAuthStep,
        raw: String,
        #[source]
        source: serde_json::Error,
        status_code: reqwest::StatusCode,
    },
    #[error("Request failed during step {step:?}: {source}")]
    Request {
        step: MinecraftAuthStep,
        #[source]
        source: reqwest::Error,
    },
    #[error("Error creating signed request buffer {step:?}: {source}")]
    ConstructingSignedRequest {
        step: MinecraftAuthStep,
        #[source]
        source: std::io::Error,
    },
    #[error("Error reading XBOX Session ID header")]
    NoSessionId,
    #[error("Error reading user hash")]
    NoUserHash,
}

const AUTH_JSON: &str = "minecraft_auth.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveDeviceToken {
    pub id: String,
    pub private_key: String,
    pub x: String,
    pub y: String,
    pub token: DeviceToken,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftLoginFlow {
    pub verifier: String,
    pub challenge: String,
    pub session_id: String,
    pub redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftAuthStore {
    pub users: HashMap<Uuid, Credentials>,
    pub token: Option<SaveDeviceToken>,
    pub default_user: Option<Uuid>,
}

impl MinecraftAuthStore {
    #[tracing::instrument]
    pub async fn init(
        dirs: &DirectoryInfo,
        io_semaphore: &IoSemaphore,
    ) -> crate::Result<Self> {
        let auth_path = dirs.caches_meta_dir().await.join(AUTH_JSON);
        let store = read_json(&auth_path, io_semaphore).await.ok();

        if let Some(store) = store {
            Ok(store)
        } else {
            Ok(Self {
                users: HashMap::new(),
                token: None,
                default_user: None,
            })
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn save(&self) -> crate::Result<()> {
        let state = State::get().await?;
        let auth_path =
            state.directories.caches_meta_dir().await.join(AUTH_JSON);

        write(&auth_path, &serde_json::to_vec(&self)?, &state.io_semaphore)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn refresh_and_get_device_token(
        &mut self,
        current_date: DateTime<Utc>,
        force_generate: bool,
    ) -> crate::Result<(DeviceTokenKey, DeviceToken, DateTime<Utc>, bool)> {
        macro_rules! generate_key {
            ($self:ident, $generate_key:expr, $device_token:expr, $SaveDeviceToken:path) => {{
                let key = generate_key()?;
                let res = device_token(&key, current_date).await?;

                self.token = Some(SaveDeviceToken {
                    id: key.id.clone(),
                    private_key: key
                        .key
                        .to_pkcs8_pem(LineEnding::default())
                        .map_err(|err| {
                            MinecraftAuthenticationError::PEMSerialize(err)
                        })?
                        .to_string(),
                    x: key.x.clone(),
                    y: key.y.clone(),
                    token: res.value.clone(),
                });
                self.save().await?;

                (key, res.value, res.date, true)
            }};
        }

        let (key, token, date, valid_date) = if let Some(ref token) = self.token
        {
            if let Ok(private_key) =
                SigningKey::from_pkcs8_pem(&token.private_key)
            {
                if token.token.not_after > Utc::now() && !force_generate {
                    (
                        DeviceTokenKey {
                            id: token.id.clone(),
                            key: private_key,
                            x: token.x.clone(),
                            y: token.y.clone(),
                        },
                        token.token.clone(),
                        current_date,
                        false,
                    )
                } else {
                    let key = DeviceTokenKey {
                        id: token.id.clone(),
                        key: private_key,
                        x: token.x.clone(),
                        y: token.y.clone(),
                    };

                    let res = device_token(&key, current_date).await?;

                    (key, res.value, res.date, true)
                }
            } else {
                generate_key!(self, generate_key, device_token, SaveDeviceToken)
            }
        } else {
            generate_key!(self, generate_key, device_token, SaveDeviceToken)
        };

        Ok((key, token, date, valid_date))
    }

    #[tracing::instrument(skip(self))]
    pub async fn login_begin(&mut self) -> crate::Result<MinecraftLoginFlow> {
        let (key, token, current_date, valid_date) =
            self.refresh_and_get_device_token(Utc::now(), false).await?;

        let verifier = generate_oauth_challenge();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&verifier);
        let result = hasher.finalize();
        let challenge = BASE64_URL_SAFE_NO_PAD.encode(result);

        match sisu_authenticate(&token.token, &challenge, &key, current_date)
            .await
        {
            Ok((session_id, redirect_uri)) => Ok(MinecraftLoginFlow {
                verifier,
                challenge,
                session_id,
                redirect_uri: redirect_uri.value.msa_oauth_redirect,
            }),
            Err(err) => {
                if !valid_date {
                    let (key, token, current_date, _) = self
                        .refresh_and_get_device_token(Utc::now(), false)
                        .await?;

                    let verifier = generate_oauth_challenge();
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(&verifier);
                    let result = hasher.finalize();
                    let challenge = BASE64_URL_SAFE_NO_PAD.encode(result);

                    let (session_id, redirect_uri) = sisu_authenticate(
                        &token.token,
                        &challenge,
                        &key,
                        current_date,
                    )
                    .await?;

                    Ok(MinecraftLoginFlow {
                        verifier,
                        challenge,
                        session_id,
                        redirect_uri: redirect_uri.value.msa_oauth_redirect,
                    })
                } else {
                    Err(crate::ErrorKind::from(err).into())
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn login_finish(
        &mut self,
        code: &str,
        flow: MinecraftLoginFlow,
    ) -> crate::Result<Credentials> {
        let (key, token, _, _) =
            self.refresh_and_get_device_token(Utc::now(), false).await?;

        let oauth_token = oauth_token(code, &flow.verifier).await?;
        let sisu_authorize = sisu_authorize(
            Some(&flow.session_id),
            &oauth_token.value.access_token,
            &token.token,
            &key,
            oauth_token.date,
        )
        .await?;

        let xbox_token = xsts_authorize(
            sisu_authorize.value,
            &token.token,
            &key,
            sisu_authorize.date,
        )
        .await?;
        let minecraft_token = minecraft_token(xbox_token.value).await?;

        minecraft_entitlements(&minecraft_token.access_token).await?;

        let profile = minecraft_profile(&minecraft_token.access_token).await?;

        let profile_id = profile.id.unwrap_or_default();

        let credentials = Credentials {
            id: profile_id,
            username: profile.name,
            access_token: minecraft_token.access_token,
            refresh_token: oauth_token.value.refresh_token,
            expires: oauth_token.date
                + Duration::seconds(oauth_token.value.expires_in as i64),
        };

        self.users.insert(profile_id, credentials.clone());

        if self.default_user.is_none() {
            self.default_user = Some(profile_id);
        }

        self.save().await?;

        Ok(credentials)
    }

    async fn refresh_token(
        &mut self,
        creds: &Credentials,
    ) -> crate::Result<Option<Credentials>> {
        let cred_id = creds.id;
        let profile_name = creds.username.clone();

        let oauth_token = oauth_refresh(&creds.refresh_token).await?;
        let (key, token, current_date, _) = self
            .refresh_and_get_device_token(oauth_token.date, false)
            .await?;

        let sisu_authorize = sisu_authorize(
            None,
            &oauth_token.value.access_token,
            &token.token,
            &key,
            current_date,
        )
        .await?;

        let xbox_token = xsts_authorize(
            sisu_authorize.value,
            &token.token,
            &key,
            sisu_authorize.date,
        )
        .await?;

        let minecraft_token = minecraft_token(xbox_token.value).await?;

        let val = Credentials {
            id: cred_id,
            username: profile_name,
            access_token: minecraft_token.access_token,
            refresh_token: oauth_token.value.refresh_token,
            expires: oauth_token.date
                + Duration::seconds(oauth_token.value.expires_in as i64),
        };

        self.users.insert(val.id, val.clone());
        self.save().await?;

        Ok(Some(val))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_default_credential(
        &mut self,
    ) -> crate::Result<Option<Credentials>> {
        let credentials = if let Some(default_user) = self.default_user {
            if let Some(creds) = self.users.get(&default_user) {
                Some(creds)
            } else {
                self.users.values().next()
            }
        } else {
            self.users.values().next()
        };

        if let Some(creds) = credentials {
            if self.default_user != Some(creds.id) {
                self.default_user = Some(creds.id);
                self.save().await?;
            }

            if creds.expires < Utc::now() {
                let old_credentials = creds.clone();

                let res = self.refresh_token(&old_credentials).await;

                match res {
                    Ok(val) => Ok(val),
                    Err(err) => {
                        if let ErrorKind::MinecraftAuthenticationError(
                            MinecraftAuthenticationError::Request {
                                ref source,
                                ..
                            },
                        ) = *err.raw
                        {
                            if source.is_connect() || source.is_timeout() {
                                return Ok(Some(old_credentials));
                            }
                        }

                        Err(err)
                    }
                }
            } else {
                Ok(Some(creds.clone()))
            }
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove(
        &mut self,
        id: Uuid,
    ) -> crate::Result<Option<Credentials>> {
        let val = self.users.remove(&id);
        self.save().await?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Credentials {
    pub id: Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
}

const MICROSOFT_CLIENT_ID: &str = "00000000402b5328";
const REDIRECT_URL: &str = "https://login.live.com/oauth20_desktop.srf";
const REQUESTED_SCOPES: &str = "service::user.auth.xboxlive.com::MBI_SSL";

struct RequestWithDate<T> {
    pub date: DateTime<Utc>,
    pub value: T,
}

// flow steps
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceToken {
    pub issue_instant: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub token: String,
    pub display_claims: HashMap<String, serde_json::Value>,
}

#[tracing::instrument(skip(key))]
pub async fn device_token(
    key: &DeviceTokenKey,
    current_date: DateTime<Utc>,
) -> Result<RequestWithDate<DeviceToken>, MinecraftAuthenticationError> {
    let res = send_signed_request(
        None,
        "https://device.auth.xboxlive.com/device/authenticate",
        "/device/authenticate",
        json!({
            "Properties": {
                "AuthMethod": "ProofOfPossession",
                "Id": format!("{{{}}}", key.id),
                "DeviceType": "Win32",
                "Version": "10.16.0",
                "ProofKey": {
                    "kty": "EC",
                    "x": key.x,
                    "y": key.y,
                    "crv": "P-256",
                    "alg": "ES256",
                    "use": "sig"
                }
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"

        }),
        key,
        MinecraftAuthStep::GetDeviceToken,
        current_date,
    )
    .await?;

    Ok(RequestWithDate {
        date: res.current_date,
        value: res.body,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RedirectUri {
    pub msa_oauth_redirect: String,
}

#[tracing::instrument(skip(key))]
async fn sisu_authenticate(
    token: &str,
    challenge: &str,
    key: &DeviceTokenKey,
    current_date: DateTime<Utc>,
) -> Result<(String, RequestWithDate<RedirectUri>), MinecraftAuthenticationError>
{
    let res = send_signed_request::<RedirectUri>(
        None,
        "https://sisu.xboxlive.com/authenticate",
        "/authenticate",
        json!({
          "AppId": MICROSOFT_CLIENT_ID,
          "DeviceToken": token,
          "Offers": [
            REQUESTED_SCOPES
          ],
          "Query": {
            "code_challenge": challenge,
            "code_challenge_method": "S256",
            "state": generate_oauth_challenge(),
            "prompt": "select_account"
          },
          "RedirectUri": REDIRECT_URL,
          "Sandbox": "RETAIL",
          "TokenType": "code",
          "TitleId": "1794566092",
        }),
        key,
        MinecraftAuthStep::SisuAuthenicate,
        current_date,
    )
    .await?;

    let session_id = res
        .headers
        .get("X-SessionId")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| MinecraftAuthenticationError::NoSessionId)?
        .to_string();

    Ok((
        session_id,
        RequestWithDate {
            date: res.current_date,
            value: res.body,
        },
    ))
}

#[derive(Deserialize)]
struct OAuthToken {
    // pub token_type: String,
    pub expires_in: u64,
    // pub scope: String,
    pub access_token: String,
    pub refresh_token: String,
    // pub user_id: String,
    // pub foci: String,
}

#[tracing::instrument]
async fn oauth_token(
    code: &str,
    verifier: &str,
) -> Result<RequestWithDate<OAuthToken>, MinecraftAuthenticationError> {
    let mut query = HashMap::new();
    query.insert("client_id", "00000000402b5328");
    query.insert("code", code);
    query.insert("code_verifier", verifier);
    query.insert("grant_type", "authorization_code");
    query.insert("redirect_uri", "https://login.live.com/oauth20_desktop.srf");
    query.insert("scope", "service::user.auth.xboxlive.com::MBI_SSL");

    let res = auth_retry(|| {
        REQWEST_CLIENT
            .post("https://login.live.com/oauth20_token.srf")
            .header("Accept", "application/json")
            .form(&query)
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::GetOAuthToken,
    })?;

    let status = res.status();
    let current_date = get_date_header(res.headers());
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::GetOAuthToken,
        }
    })?;

    let body = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::GetOAuthToken,
            status_code: status,
        }
    })?;

    Ok(RequestWithDate {
        date: current_date,
        value: body,
    })
}

#[tracing::instrument]
async fn oauth_refresh(
    refresh_token: &str,
) -> Result<RequestWithDate<OAuthToken>, MinecraftAuthenticationError> {
    let mut query = HashMap::new();
    query.insert("client_id", "00000000402b5328");
    query.insert("refresh_token", refresh_token);
    query.insert("grant_type", "refresh_token");
    query.insert("redirect_uri", "https://login.live.com/oauth20_desktop.srf");
    query.insert("scope", "service::user.auth.xboxlive.com::MBI_SSL");

    let res = auth_retry(|| {
        REQWEST_CLIENT
            .post("https://login.live.com/oauth20_token.srf")
            .header("Accept", "application/json")
            .form(&query)
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::RefreshOAuthToken,
    })?;

    let status = res.status();
    let current_date = get_date_header(res.headers());
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::RefreshOAuthToken,
        }
    })?;

    let body = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::RefreshOAuthToken,
            status_code: status,
        }
    })?;

    Ok(RequestWithDate {
        date: current_date,
        value: body,
    })
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SisuAuthorize {
    // pub authorization_token: DeviceToken,
    // pub device_token: String,
    // pub sandbox: String,
    pub title_token: DeviceToken,
    pub user_token: DeviceToken,
    // pub web_page: String,
}

#[tracing::instrument(skip(key))]
async fn sisu_authorize(
    session_id: Option<&str>,
    access_token: &str,
    device_token: &str,
    key: &DeviceTokenKey,
    current_date: DateTime<Utc>,
) -> Result<RequestWithDate<SisuAuthorize>, MinecraftAuthenticationError> {
    let res = send_signed_request(
        None,
        "https://sisu.xboxlive.com/authorize",
        "/authorize",
        json!({
            "AccessToken": format!("t={access_token}"),
            "AppId": "00000000402b5328",
            "DeviceToken": device_token,
            "ProofKey": {
                "kty": "EC",
                "x": key.x,
                "y": key.y,
                "crv": "P-256",
                "alg": "ES256",
                "use": "sig"
            },
            "Sandbox": "RETAIL",
            "SessionId": session_id,
            "SiteName": "user.auth.xboxlive.com",
            "RelyingParty": "http://xboxlive.com",
            "UseModernGamertag": true
        }),
        key,
        MinecraftAuthStep::SisuAuthorize,
        current_date,
    )
    .await?;

    Ok(RequestWithDate {
        date: res.current_date,
        value: res.body,
    })
}

#[tracing::instrument(skip(key))]
async fn xsts_authorize(
    authorize: SisuAuthorize,
    device_token: &str,
    key: &DeviceTokenKey,
    current_date: DateTime<Utc>,
) -> Result<RequestWithDate<DeviceToken>, MinecraftAuthenticationError> {
    let res = send_signed_request(
        None,
        "https://xsts.auth.xboxlive.com/xsts/authorize",
        "/xsts/authorize",
        json!({
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [authorize.user_token.token],
                "DeviceToken": device_token,
                "TitleToken": authorize.title_token.token,
            },
        }),
        key,
        MinecraftAuthStep::XstsAuthorize,
        current_date,
    )
    .await?;

    Ok(RequestWithDate {
        date: res.current_date,
        value: res.body,
    })
}

#[derive(Deserialize)]
struct MinecraftToken {
    // pub username: String,
    pub access_token: String,
    // pub token_type: String,
    // pub expires_in: u64,
}

#[tracing::instrument]
async fn minecraft_token(
    token: DeviceToken,
) -> Result<MinecraftToken, MinecraftAuthenticationError> {
    let uhs = token
        .display_claims
        .get("xui")
        .and_then(|x| x.get(0))
        .and_then(|x| x.get("uhs"))
        .and_then(|x| x.as_str().map(String::from))
        .ok_or_else(|| MinecraftAuthenticationError::NoUserHash)?;

    let token = token.token;

    let res = auth_retry(|| {
        REQWEST_CLIENT
            .post("https://api.minecraftservices.com/launcher/login")
            .header("Accept", "application/json")
            .json(&json!({
                "platform": "PC_LAUNCHER",
                "xtoken": format!("XBL3.0 x={uhs};{token}"),
            }))
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::MinecraftToken,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::MinecraftToken,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::MinecraftToken,
            status_code: status,
        }
    })
}

#[derive(Deserialize)]
struct MinecraftProfile {
    pub id: Option<Uuid>,
    pub name: String,
}

#[tracing::instrument]
async fn minecraft_profile(
    token: &str,
) -> Result<MinecraftProfile, MinecraftAuthenticationError> {
    let res = auth_retry(|| {
        REQWEST_CLIENT
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Accept", "application/json")
            .bearer_auth(token)
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::MinecraftProfile,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::MinecraftProfile,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::MinecraftProfile,
            status_code: status,
        }
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinecraftEntitlements {}

#[tracing::instrument]
async fn minecraft_entitlements(
    token: &str,
) -> Result<MinecraftEntitlements, MinecraftAuthenticationError> {
    let res = auth_retry(|| {
        REQWEST_CLIENT
            .get(format!("https://api.minecraftservices.com/entitlements/license?requestId={}", Uuid::new_v4()))
            .header("Accept", "application/json")
            .bearer_auth(token)
            .send()
    })
        .await.map_err(|source| MinecraftAuthenticationError::Request { source, step: MinecraftAuthStep::MinecraftEntitlements })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::MinecraftEntitlements,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::MinecraftEntitlements,
            status_code: status,
        }
    })
}

// auth utils
#[tracing::instrument(skip(reqwest_request))]
async fn auth_retry<F>(
    reqwest_request: impl Fn() -> F,
) -> Result<reqwest::Response, reqwest::Error>
where
    F: Future<Output = Result<Response, reqwest::Error>>,
{
    const RETRY_COUNT: usize = 5; // Does command 9 times
    const RETRY_WAIT: std::time::Duration =
        std::time::Duration::from_millis(250);

    let mut resp = reqwest_request().await;
    for i in 0..RETRY_COUNT {
        match &resp {
            Ok(_) => {
                break;
            }
            Err(err) => {
                if err.is_connect() || err.is_timeout() {
                    if i < RETRY_COUNT - 1 {
                        tracing::debug!(
                            "Request failed with connect error, retrying...",
                        );
                        tokio::time::sleep(RETRY_WAIT).await;
                        resp = reqwest_request().await;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    resp
}

pub struct DeviceTokenKey {
    pub id: String,
    pub key: SigningKey,
    pub x: String,
    pub y: String,
}

#[tracing::instrument]
fn generate_key() -> Result<DeviceTokenKey, MinecraftAuthenticationError> {
    let id = Uuid::new_v4().to_string().to_uppercase();

    let signing_key = SigningKey::random(&mut OsRng);
    let public_key = VerifyingKey::from(&signing_key);

    let encoded_point = public_key.to_encoded_point(false);

    Ok(DeviceTokenKey {
        id,
        key: signing_key,
        x: BASE64_URL_SAFE_NO_PAD.encode(
            encoded_point.x().ok_or_else(|| {
                MinecraftAuthenticationError::ReadingPublicKey
            })?,
        ),
        y: BASE64_URL_SAFE_NO_PAD.encode(
            encoded_point.y().ok_or_else(|| {
                MinecraftAuthenticationError::ReadingPublicKey
            })?,
        ),
    })
}

struct SignedRequestResponse<T> {
    pub headers: HeaderMap,
    pub current_date: DateTime<Utc>,
    pub body: T,
}

#[tracing::instrument(skip(key))]
async fn send_signed_request<T: DeserializeOwned>(
    authorization: Option<&str>,
    url: &str,
    url_path: &str,
    raw_body: serde_json::Value,
    key: &DeviceTokenKey,
    step: MinecraftAuthStep,
    current_date: DateTime<Utc>,
) -> Result<SignedRequestResponse<T>, MinecraftAuthenticationError> {
    let auth = authorization.map_or(Vec::new(), |v| v.as_bytes().to_vec());

    let body = serde_json::to_vec(&raw_body).map_err(|source| {
        MinecraftAuthenticationError::SerializeBody { source, step }
    })?;
    let time: u128 =
        { ((current_date.timestamp() as u128) + 11644473600) * 10000000 };

    use byteorder::WriteBytesExt;
    let mut buffer = Vec::new();
    buffer.write_u32::<BigEndian>(1).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    buffer.write_u8(0).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    buffer
        .write_u64::<BigEndian>(time as u64)
        .map_err(|source| {
            MinecraftAuthenticationError::ConstructingSignedRequest {
                source,
                step,
            }
        })?;
    buffer.write_u8(0).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    buffer.extend_from_slice("POST".as_bytes());
    buffer.write_u8(0).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    buffer.extend_from_slice(url_path.as_bytes());
    buffer.write_u8(0).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    buffer.extend_from_slice(&auth);
    buffer.write_u8(0).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    buffer.extend_from_slice(&body);
    buffer.write_u8(0).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;

    let ecdsa_sig: Signature = key.key.sign(&buffer);

    let mut sig_buffer = Vec::new();
    sig_buffer.write_i32::<BigEndian>(1).map_err(|source| {
        MinecraftAuthenticationError::ConstructingSignedRequest { source, step }
    })?;
    sig_buffer
        .write_u64::<BigEndian>(time as u64)
        .map_err(|source| {
            MinecraftAuthenticationError::ConstructingSignedRequest {
                source,
                step,
            }
        })?;
    sig_buffer.extend_from_slice(&ecdsa_sig.r().to_bytes());
    sig_buffer.extend_from_slice(&ecdsa_sig.s().to_bytes());

    let signature = BASE64_STANDARD.encode(&sig_buffer);

    let res = auth_retry(|| {
        let mut request = REQWEST_CLIENT
            .post(url)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Accept", "application/json")
            .header("Signature", &signature);

        if url != "https://sisu.xboxlive.com/authorize" {
            request = request.header("x-xbl-contract-version", "1");
        }

        if let Some(auth) = authorization {
            request = request.header("Authorization", auth);
        }

        request.body(body.clone()).send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request { source, step })?;

    let status = res.status();
    let headers = res.headers().clone();

    let current_date = get_date_header(&headers);

    let body = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request { source, step }
    })?;

    let body = serde_json::from_str(&body).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: body,
            step,
            status_code: status,
        }
    })?;
    Ok(SignedRequestResponse {
        headers,
        current_date,
        body,
    })
}

#[tracing::instrument]
fn get_date_header(headers: &HeaderMap) -> DateTime<Utc> {
    headers
        .get(reqwest::header::DATE)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| DateTime::parse_from_rfc2822(x).ok())
        .map(|x| x.with_timezone(&Utc))
        .unwrap_or(Utc::now())
}

#[tracing::instrument]
#[allow(clippy::format_collect)]
fn generate_oauth_challenge() -> String {
    let mut rng = rand::thread_rng();

    let bytes: Vec<u8> = (0..64).map(|_| rng.gen::<u8>()).collect();
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}
