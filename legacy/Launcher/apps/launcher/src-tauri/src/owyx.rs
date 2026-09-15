// Owyx account login for the launcher.
//
// Talks to the Owyx site API (see owyxsite/LAUNCHER_SITE_CONTRACT.md):
//   POST /api/auth/login       -> { token }
//   GET  /api/launcher/me      -> { user, serverAccess, cosmetics, ... }
//
// The JWT is stored in the launcher data dir (~/owyx/owyx_auth.json), keyed by
// nickname. It is never returned to the UI or logged. Errors are returned as
// stable codes the frontend maps to localized messages:
//   "network" | "invalid_credentials" | "account_inactive" | "server"

use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_BASE: &str = "https://api.owyx.site";
const TIMEOUT: Duration = Duration::from_secs(20);
pub const CLIENT_KEY_HEADER: &str = "X-Owyx-Client-Key";

/// Resolve the API base URL: explicit arg > OWYX_API_BASE_URL env > prod default.
pub fn resolve_base(base_url: Option<&str>) -> String {
    if let Some(u) = base_url.map(str::trim).filter(|s| !s.is_empty()) {
        return u.trim_end_matches('/').to_string();
    }
    match std::env::var("OWYX_API_BASE_URL") {
        Ok(v) if !v.trim().is_empty() => v.trim().trim_end_matches('/').to_string(),
        _ => DEFAULT_BASE.to_string(),
    }
}

/// Shared secret for api.owyx.site (`LAUNCHER_CLIENT_KEY` on the site).
/// Env `OWYX_CLIENT_KEY` wins; else optional `clientKey` in launcher config dir file
/// `~/owyx/client_key` (one line). Dev against loopback may omit the key.
pub fn resolve_client_key() -> Option<String> {
    if let Ok(v) = std::env::var("OWYX_CLIENT_KEY") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    let path = config::app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("client_key");
    if let Ok(raw) = fs::read_to_string(&path) {
        let t = raw.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    None
}

fn http_client(base_url: &str) -> Result<reqwest::Client, String> {
    let base = reqwest::Url::parse(base_url).map_err(|_| "network".to_string())?;
    let expected_host = base
        .host_str()
        .ok_or_else(|| "network".to_string())?
        .to_ascii_lowercase();
    let expected_scheme = base.scheme().to_string();
    let expected_port = base.port_or_known_default();
    let redirect_policy = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= 5 {
            return attempt.error("Too many redirects");
        }
        let next = attempt.url();
        let same_origin = next.scheme() == expected_scheme
            && next
                .host_str()
                .is_some_and(|host| host.eq_ignore_ascii_case(&expected_host))
            && next.port_or_known_default() == expected_port
            && next.username().is_empty()
            && next.password().is_none();
        if same_origin {
            attempt.follow()
        } else {
            attempt.error("Owyx API redirect changed origin")
        }
    });
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(key) = resolve_client_key() {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(CLIENT_KEY_HEADER.as_bytes()),
            reqwest::header::HeaderValue::from_str(&key),
        ) {
            headers.insert(name, val);
        }
    }
    reqwest::Client::builder()
        .user_agent(concat!("Owyx/", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .timeout(TIMEOUT)
        .connect_timeout(Duration::from_secs(10))
        .redirect(redirect_policy)
        .build()
        .map_err(|_| "network".to_string())
}

// ---- token store (~/owyx/owyx_auth.json) --------------------------------

#[derive(Serialize, Deserialize, Default)]
struct TokenStore {
    #[serde(default)]
    accounts: HashMap<String, StoredAccount>,
}

#[derive(Serialize, Deserialize, Clone)]
struct StoredAccount {
    base_url: String,
    token: String,
}

fn store_path() -> Result<PathBuf, String> {
    Ok(config::app_data_dir()?.join("owyx_auth.json"))
}

fn load_store() -> TokenStore {
    let Ok(path) = store_path() else {
        return TokenStore::default();
    };
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => TokenStore::default(),
    }
}

fn save_store(store: &TokenStore) -> Result<(), String> {
    let path = store_path()?;
    let raw = serde_json::to_string_pretty(store).map_err(|_| "server".to_string())?;
    config::atomic_write_path(&path, raw.as_bytes())
}

// ---- API shapes ---------------------------------------------------------

/// What the launcher UI receives after a successful login / me call.
/// The token is intentionally NOT part of this struct.
#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OwyxSession {
    pub nickname: String,
    pub email: String,
    pub role: String,
    pub trust_level: i64,
    pub banned: bool,
    pub server_access: bool,
    pub access_reason: String,
    pub skin_url: Option<String>,
    pub skin_model: String,
    pub base_url: String,
}

fn parse_me(base: &str, me: &serde_json::Value) -> OwyxSession {
    let user = &me["user"];
    let cosmetics = &me["cosmetics"];
    OwyxSession {
        nickname: user["nickname"].as_str().unwrap_or_default().to_string(),
        email: user["email"].as_str().unwrap_or_default().to_string(),
        role: user["role"].as_str().unwrap_or("user").to_string(),
        trust_level: user["trustLevel"].as_i64().unwrap_or(0),
        banned: user["banned"].as_bool().unwrap_or(false),
        server_access: me["serverAccess"].as_bool().unwrap_or(false),
        access_reason: me["accessReason"].as_str().unwrap_or("").to_string(),
        skin_url: me["skinUrl"].as_str().map(|s| s.to_string()),
        skin_model: cosmetics["skinModel"]
            .as_str()
            .unwrap_or("classic")
            .to_string(),
        base_url: base.to_string(),
    }
}

/// Login with email + password, then fetch the launcher profile. Stores the JWT.
pub async fn login(
    base_url: Option<&str>,
    email: &str,
    password: &str,
) -> Result<OwyxSession, String> {
    let base = resolve_base(base_url);
    let client = http_client(&base)?;

    let login_resp = client
        .post(format!("{base}/api/auth/login"))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|_| "network".to_string())?;

    let status = login_resp.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            400 | 401 => "invalid_credentials".to_string(),
            403 => "account_inactive".to_string(),
            _ => "server".to_string(),
        });
    }

    let login_body: serde_json::Value =
        login_resp.json().await.map_err(|_| "server".to_string())?;
    let token = login_body["token"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "server".to_string())?
        .to_string();

    let session = fetch_me(&client, &base, &token).await?;

    let mut store = load_store();
    store.accounts.insert(
        session.nickname.clone(),
        StoredAccount {
            base_url: base.clone(),
            token,
        },
    );
    save_store(&store)?;

    Ok(session)
}

async fn fetch_me(
    client: &reqwest::Client,
    base: &str,
    token: &str,
) -> Result<OwyxSession, String> {
    let me_resp = client
        .get(format!("{base}/api/launcher/me"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "network".to_string())?;
    let status = me_resp.status();
    if !status.is_success() {
        // 401/403 = session gone or forbidden — ask the user to sign in again
        // instead of a vague "server" error (site+launcher both hold JWTs).
        return Err(match status.as_u16() {
            401 | 403 => "no_session".to_string(),
            _ => "server".to_string(),
        });
    }
    let me_body: serde_json::Value = me_resp.json().await.map_err(|_| "server".to_string())?;
    Ok(parse_me(base, &me_body))
}

/// Refresh the stored session for a nickname (re-fetch /me with the saved token).
pub async fn me(nickname: &str) -> Result<OwyxSession, String> {
    let store = load_store();
    let account = store
        .accounts
        .get(nickname)
        .cloned()
        .ok_or_else(|| "no_session".to_string())?;
    let client = http_client(&account.base_url)?;
    match fetch_me(&client, &account.base_url, &account.token).await {
        Ok(session) => Ok(session),
        Err(code) if code == "no_session" => {
            // Drop the dead token so the next attempt goes through the login UI.
            let _ = logout(nickname);
            Err(code)
        }
        Err(other) => Err(other),
    }
}

/// Drop the stored token for a nickname (logout / delete profile).
pub fn logout(nickname: &str) -> Result<(), String> {
    let mut store = load_store();
    if store.accounts.remove(nickname).is_some() {
        save_store(&store)?;
    }
    Ok(())
}

fn stored_account(nickname: &str) -> Result<StoredAccount, String> {
    load_store()
        .accounts
        .get(nickname)
        .cloned()
        .ok_or_else(|| "no_session".to_string())
}

/// Admin API call using the stored JWT. Path must stay under /api/admin/.
pub async fn admin_request(
    nickname: &str,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let path = path.trim();
    if !path.starts_with("/api/admin/") {
        return Err("Admin path must start with /api/admin/".into());
    }
    let method = method.trim().to_ascii_uppercase();
    if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "DELETE") {
        return Err("Unsupported method".into());
    }
    let account = stored_account(nickname)?;
    let client = http_client(&account.base_url)?;
    let url = format!("{}{path}", account.base_url);
    let mut req = match method.as_str() {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => client.get(&url),
    };
    req = req.bearer_auth(&account.token);
    if let Some(body) = body {
        if method != "GET" {
            req = req.json(&body);
        }
    }
    let resp = req.send().await.map_err(|_| "network".to_string())?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
    if !status.is_success() {
        let msg = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("server");
        return Err(if status.as_u16() == 401 || status.as_u16() == 403 {
            "no_session".into()
        } else {
            msg.to_string()
        });
    }
    Ok(value)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinApplyResult {
    pub applied: bool,
    pub path: Option<String>,
    pub model: String,
    pub message: String,
}

/// Download the account skin from /me onto disk (profile + optional instance).
pub async fn apply_skin(
    nickname: &str,
    instance_id: Option<String>,
) -> Result<SkinApplyResult, String> {
    let session = me(nickname).await?;
    let Some(url) = session.skin_url.filter(|s| !s.is_empty()) else {
        return Ok(SkinApplyResult {
            applied: false,
            path: None,
            model: session.skin_model,
            message: "no_skin".into(),
        });
    };
    crate::catalog::assert_asset_url(&url, &session.base_url)?;
    let client = crate::catalog::http_client(&session.base_url)?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|_| "network".to_string())?
        .error_for_status()
        .map_err(|_| "server".to_string())?;
    crate::catalog::assert_asset_url(response.url().as_str(), &session.base_url)?;
    let bytes = response.bytes().await.map_err(|_| "network".to_string())?;
    if bytes.len() < 64 || bytes.len() > 512 * 1024 {
        return Err("Skin file size is not a valid PNG skin".into());
    }
    if bytes[0] != 0x89 || bytes.get(1) != Some(&b'P') {
        return Err("Skin is not a PNG".into());
    }

    let root = config::ensure_layout()?;
    let skins = root.join("skins");
    fs::create_dir_all(&skins).map_err(|e| format!("Create skins dir: {e}"))?;
    let safe_nick: String = nickname
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(32)
        .collect();
    if safe_nick.is_empty() {
        return Err("Nickname cannot be used as a skin file name".into());
    }
    let dest = skins.join(format!("{safe_nick}.png"));
    config::atomic_write_path(&dest, &bytes)?;

    if let Some(id) = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        crate::instances::validate_instance_id(id)?;
        let inst_dir = root.join("instances").join(id).join("game").join("owyx");
        fs::create_dir_all(&inst_dir).map_err(|e| format!("Create instance skin dir: {e}"))?;
        config::atomic_write_path(&inst_dir.join("skin.png"), &bytes)?;
        config::atomic_write_path(
            &inst_dir.join("skin-model.txt"),
            session.skin_model.as_bytes(),
        )?;
    }

    Ok(SkinApplyResult {
        applied: true,
        path: Some(dest.display().to_string()),
        model: session.skin_model,
        message: "ok".into(),
    })
}
