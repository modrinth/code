//! Microsoft account auth — Path A device-code + Xbox + Minecraft when
//! `MICROSOFT_CLIENT_ID` is set; otherwise `unconfigured`.
//!
//! Flow (Azure AD public client / Minecraft launcher-compatible):
//! 1. OAuth2 device code (`consumers` tenant)
//! 2. Poll token endpoint
//! 3. Xbox Live authenticate → XSTS → Minecraft Services login → profile
//!
//! Tokens are stored as an opaque `ms1:` base64 JSON blob on the profile
//! (machine-local file). Windows DPAPI wrapping remains a follow-up.
//! Never invent fake user codes when exchange is unavailable.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";
const XBOX_USER_AUTH: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";
const USER_AGENT: &str = concat!("owyx/", env!("CARGO_PKG_VERSION"), " (Microsoft auth)");

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsAuthStatus {
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamertag: Option<String>,
    /// Opaque auth blob for profile persistence (Path A success only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_blob: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingSession {
    client_id: String,
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: Duration,
    expires_at: Instant,
    last_poll: Option<Instant>,
    success: Option<SuccessPayload>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct SuccessPayload {
    gamertag: String,
    auth_blob: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredMsTokens {
    v: u8,
    gamertag: String,
    uuid: String,
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    obtained_at: u64,
}

static STATE: Mutex<Option<PendingSession>> = Mutex::new(None);

fn client_id() -> Option<String> {
    std::env::var("MICROSOFT_CLIENT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn http() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn encode_auth_blob(tokens: &StoredMsTokens) -> Result<String, String> {
    let json = serde_json::to_vec(tokens).map_err(|e| format!("Serialize tokens: {e}"))?;
    Ok(format!("ms1:{}", b64_encode(&json)))
}

fn b64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    expires_in: u64,
    #[serde(default)]
    interval: u64,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XboxAuthResponse {
    #[serde(default, rename = "Token")]
    token: Option<String>,
    #[serde(default, rename = "DisplayClaims")]
    display_claims: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct McLoginResponse {
    #[serde(default)]
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McProfile {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

pub async fn start() -> Result<MsAuthStatus, String> {
    let Some(id) = client_id() else {
        return Ok(MsAuthStatus {
            phase: "unconfigured".into(),
            message: Some(
                "MICROSOFT_CLIENT_ID is not set. See Launcher/apps/launcher/docs/MICROSOFT_AUTH.md"
                    .into(),
            ),
            user_code: None,
            verification_uri: None,
            gamertag: None,
            auth_blob: None,
        });
    };

    let client = http()?;
    let res = client
        .post(DEVICE_CODE_URL)
        .form(&[("client_id", id.as_str()), ("scope", SCOPE)])
        .send()
        .await
        .map_err(|e| format!("Device code request failed: {e}"))?;
    let status = res.status();
    let body: DeviceCodeResponse = res
        .json()
        .await
        .map_err(|e| format!("Device code JSON: {e}"))?;
    if !status.is_success() || body.error.is_some() {
        let msg = body
            .error_description
            .or(body.error)
            .unwrap_or_else(|| format!("Device code HTTP {status}"));
        // Common when Azure app lacks public-client / device-code: surface clearly.
        return Ok(MsAuthStatus {
            phase: "error".into(),
            message: Some(format!(
                "{msg} — ensure the Azure app allows public client device code and Xbox Live sign-in. See docs/MICROSOFT_AUTH.md"
            )),
            user_code: None,
            verification_uri: None,
            gamertag: None,
            auth_blob: None,
        });
    }
    if body.device_code.is_empty() || body.user_code.is_empty() {
        return Ok(MsAuthStatus {
            phase: "error".into(),
            message: Some("Microsoft returned an empty device code".into()),
            user_code: None,
            verification_uri: None,
            gamertag: None,
            auth_blob: None,
        });
    }

    let expires = if body.expires_in == 0 {
        900
    } else {
        body.expires_in
    };
    let interval = if body.interval == 0 { 5 } else { body.interval };
    let session = PendingSession {
        client_id: id,
        device_code: body.device_code,
        user_code: body.user_code.clone(),
        verification_uri: if body.verification_uri.is_empty() {
            "https://www.microsoft.com/link".into()
        } else {
            body.verification_uri.clone()
        },
        interval: Duration::from_secs(interval),
        expires_at: Instant::now() + Duration::from_secs(expires),
        last_poll: None,
        success: None,
        error: None,
    };
    let mut guard = STATE.lock().map_err(|e| e.to_string())?;
    *guard = Some(session);

    Ok(MsAuthStatus {
        phase: "waiting".into(),
        message: body.message.or(Some("Enter the code at the verification URL".into())),
        user_code: Some(body.user_code),
        verification_uri: Some(if body.verification_uri.is_empty() {
            "https://www.microsoft.com/link".into()
        } else {
            body.verification_uri
        }),
        gamertag: None,
        auth_blob: None,
    })
}

pub async fn poll() -> Result<MsAuthStatus, String> {
    if client_id().is_none() {
        return Ok(MsAuthStatus {
            phase: "unconfigured".into(),
            message: Some("MICROSOFT_CLIENT_ID is not set".into()),
            user_code: None,
            verification_uri: None,
            gamertag: None,
            auth_blob: None,
        });
    }

    // Snapshot session fields under the lock, then release before network I/O.
    let (client_id, device_code, user_code, verification_uri, interval, expired) = {
        let mut guard = STATE.lock().map_err(|e| e.to_string())?;
        let Some(session) = guard.as_mut() else {
            return Ok(MsAuthStatus {
                phase: "error".into(),
                message: Some("No active Microsoft sign-in session".into()),
                user_code: None,
                verification_uri: None,
                gamertag: None,
                auth_blob: None,
            });
        };
        if let Some(ok) = session.success.clone() {
            return Ok(MsAuthStatus {
                phase: "success".into(),
                message: Some("Signed in".into()),
                user_code: None,
                verification_uri: None,
                gamertag: Some(ok.gamertag),
                auth_blob: Some(ok.auth_blob),
            });
        }
        if let Some(err) = session.error.clone() {
            return Ok(MsAuthStatus {
                phase: "error".into(),
                message: Some(err),
                user_code: None,
                verification_uri: None,
                gamertag: None,
                auth_blob: None,
            });
        }
        if Instant::now() >= session.expires_at {
            *guard = None;
            return Ok(MsAuthStatus {
                phase: "error".into(),
                message: Some("Microsoft sign-in timed out".into()),
                user_code: None,
                verification_uri: None,
                gamertag: None,
                auth_blob: None,
            });
        }
        if let Some(last) = session.last_poll {
            if last.elapsed() < session.interval {
                return Ok(MsAuthStatus {
                    phase: "waiting".into(),
                    message: Some("Waiting for Microsoft…".into()),
                    user_code: Some(session.user_code.clone()),
                    verification_uri: Some(session.verification_uri.clone()),
                    gamertag: None,
                    auth_blob: None,
                });
            }
        }
        session.last_poll = Some(Instant::now());
        (
            session.client_id.clone(),
            session.device_code.clone(),
            session.user_code.clone(),
            session.verification_uri.clone(),
            session.interval,
            false,
        )
    };
    let _ = (interval, expired);

    let client = http()?;
    let res = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id.as_str()),
            ("device_code", device_code.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Token poll failed: {e}"))?;
    let token_body: TokenResponse = res
        .json()
        .await
        .map_err(|e| format!("Token JSON: {e}"))?;

    if let Some(err) = token_body.error.as_deref() {
        match err {
            "authorization_pending" => {
                return Ok(MsAuthStatus {
                    phase: "waiting".into(),
                    message: Some("Waiting for Microsoft…".into()),
                    user_code: Some(user_code),
                    verification_uri: Some(verification_uri),
                    gamertag: None,
                    auth_blob: None,
                });
            }
            "slow_down" => {
                // Azure asks clients to back off (~+5s).
                {
                    let mut guard = STATE.lock().map_err(|e| e.to_string())?;
                    if let Some(s) = guard.as_mut() {
                        s.interval = s.interval.saturating_add(Duration::from_secs(5));
                    }
                }
                return Ok(MsAuthStatus {
                    phase: "waiting".into(),
                    message: Some("Waiting for Microsoft…".into()),
                    user_code: Some(user_code),
                    verification_uri: Some(verification_uri),
                    gamertag: None,
                    auth_blob: None,
                });
            }
            "expired_token" | "bad_verification_code" => {
                let mut guard = STATE.lock().map_err(|e| e.to_string())?;
                *guard = None;
                return Ok(MsAuthStatus {
                    phase: "error".into(),
                    message: Some(
                        token_body
                            .error_description
                            .unwrap_or_else(|| err.to_string()),
                    ),
                    user_code: None,
                    verification_uri: None,
                    gamertag: None,
                    auth_blob: None,
                });
            }
            other => {
                let msg = token_body
                    .error_description
                    .unwrap_or_else(|| other.to_string());
                let mut guard = STATE.lock().map_err(|e| e.to_string())?;
                if let Some(s) = guard.as_mut() {
                    s.error = Some(msg.clone());
                }
                return Ok(MsAuthStatus {
                    phase: "error".into(),
                    message: Some(msg),
                    user_code: None,
                    verification_uri: None,
                    gamertag: None,
                    auth_blob: None,
                });
            }
        }
    }

    let ms_access = token_body
        .access_token
        .ok_or_else(|| "Microsoft token response missing access_token".to_string())?;
    let refresh = token_body.refresh_token;

    match exchange_for_minecraft(&client, &ms_access, refresh.as_deref()).await {
        Ok(payload) => {
            let mut guard = STATE.lock().map_err(|e| e.to_string())?;
            if let Some(s) = guard.as_mut() {
                s.success = Some(payload.clone());
            }
            Ok(MsAuthStatus {
                phase: "success".into(),
                message: Some("Signed in".into()),
                user_code: None,
                verification_uri: None,
                gamertag: Some(payload.gamertag),
                auth_blob: Some(payload.auth_blob),
            })
        }
        Err(err) => {
            let mut guard = STATE.lock().map_err(|e| e.to_string())?;
            if let Some(s) = guard.as_mut() {
                s.error = Some(err.clone());
            }
            Ok(MsAuthStatus {
                phase: "error".into(),
                message: Some(err),
                user_code: None,
                verification_uri: None,
                gamertag: None,
                auth_blob: None,
            })
        }
    }
}

async fn exchange_for_minecraft(
    client: &reqwest::Client,
    ms_access: &str,
    refresh: Option<&str>,
) -> Result<SuccessPayload, String> {
    // Xbox Live user token
    let xbox_body = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={ms_access}")
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    let xbox_res = client
        .post(XBOX_USER_AUTH)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&xbox_body)
        .send()
        .await
        .map_err(|e| format!("Xbox auth: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Xbox auth HTTP: {e}"))?;
    let xbox: XboxAuthResponse = xbox_res
        .json()
        .await
        .map_err(|e| format!("Xbox auth JSON: {e}"))?;
    let xbox_token = xbox
        .token
        .ok_or_else(|| "Xbox auth missing Token".to_string())?;
    let uhs = xbox
        .display_claims
        .as_ref()
        .and_then(|v| v.get("xui"))
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("uhs"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Xbox auth missing user hash".to_string())?
        .to_string();

    // XSTS
    let xsts_body = json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbox_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });
    let xsts_res = client
        .post(XSTS_AUTH)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&xsts_body)
        .send()
        .await
        .map_err(|e| format!("XSTS: {e}"))?;
    if !xsts_res.status().is_success() {
        let status = xsts_res.status();
        let text = xsts_res.text().await.unwrap_or_default();
        return Err(format!(
            "XSTS HTTP {status}: {text}. The Azure app may lack Xbox Live permission, or the Microsoft account has no Xbox profile."
        ));
    }
    let xsts: XboxAuthResponse = xsts_res
        .json()
        .await
        .map_err(|e| format!("XSTS JSON: {e}"))?;
    let xsts_token = xsts
        .token
        .ok_or_else(|| "XSTS missing Token".to_string())?;

    // Minecraft login
    let mc_body = json!({
        "identityToken": format!("XBL3.0 x={uhs};{xsts_token}")
    });
    let mc_res = client
        .post(MC_LOGIN)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&mc_body)
        .send()
        .await
        .map_err(|e| format!("Minecraft login: {e}"))?
        .error_for_status()
        .map_err(|e| {
            format!(
                "Minecraft login HTTP: {e}. Account may not own Minecraft Java."
            )
        })?;
    let mc_login: McLoginResponse = mc_res
        .json()
        .await
        .map_err(|e| format!("Minecraft login JSON: {e}"))?;
    let mc_token = mc_login
        .access_token
        .ok_or_else(|| "Minecraft login missing access_token".to_string())?;

    let profile_res = client
        .get(MC_PROFILE)
        .header("Authorization", format!("Bearer {mc_token}"))
        .send()
        .await
        .map_err(|e| format!("Minecraft profile: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Minecraft profile HTTP: {e}"))?;
    let profile: McProfile = profile_res
        .json()
        .await
        .map_err(|e| format!("Minecraft profile JSON: {e}"))?;
    let gamertag = profile
        .name
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Minecraft profile missing name".to_string())?;
    let uuid = profile.id.unwrap_or_default();

    let stored = StoredMsTokens {
        v: 1,
        gamertag: gamertag.clone(),
        uuid,
        access_token: mc_token,
        refresh_token: refresh.map(|s| s.to_string()),
        obtained_at: now_secs(),
    };
    let auth_blob = encode_auth_blob(&stored)?;
    Ok(SuccessPayload {
        gamertag,
        auth_blob,
    })
}

pub fn cancel() -> Result<(), String> {
    let mut guard = STATE.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_blob_roundtrip_prefix() {
        let stored = StoredMsTokens {
            v: 1,
            gamertag: "Test".into(),
            uuid: "abc".into(),
            access_token: "tok".into(),
            refresh_token: None,
            obtained_at: 1,
        };
        let blob = encode_auth_blob(&stored).unwrap();
        assert!(blob.starts_with("ms1:"));
    }
}
