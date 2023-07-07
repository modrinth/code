use crate::database::models::{generate_state_id, StateId};
use crate::models::ids::base62_impl::{parse_base62, to_base62};
use std::collections::HashMap;
use std::sync::Arc;

use crate::parse_strings_from_var;

use actix_web::web::{scope, Data, Query, ServiceConfig};
use actix_web::{get, HttpRequest, HttpResponse};
use chrono::Utc;
use reqwest::header::AUTHORIZATION;
use rust_decimal::Decimal;

use crate::auth::session::issue_session;
use crate::auth::AuthenticationError;
use crate::file_hosting::FileHost;
use crate::models::users::{Badges, Role};
use crate::util::ext::{get_image_content_type, get_image_ext};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("auth").service(auth_callback).service(init));
}

#[derive(Serialize, Deserialize, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    #[default]
    GitHub,
    Discord,
    Microsoft,
    GitLab,
    Google,
    Steam,
}

#[derive(Debug)]
pub struct TempUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,

    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub name: Option<String>,
}

impl AuthProvider {
    pub fn get_redirect_url(&self, state: StateId) -> Result<String, AuthenticationError> {
        let state = to_base62(state.0 as u64);
        let self_addr = dotenvy::var("SELF_ADDR")?;
        let raw_redirect_uri = format!("{}/v2/auth/callback", self_addr);
        let redirect_uri = urlencoding::encode(&raw_redirect_uri);

        Ok(match self {
            AuthProvider::GitHub => {
                let client_id = dotenvy::var("GITHUB_CLIENT_ID")?;

                format!(
                    "https://github.com/login/oauth/authorize?client_id={}&state={}&scope=read%3Auser%20user%3Aemail&redirect_uri={}",
                    client_id,
                    state,
                    redirect_uri,
                )
            }
            AuthProvider::Discord => {
                let client_id = dotenvy::var("DISCORD_CLIENT_ID")?;

                format!("https://discord.com/api/oauth2/authorize?client_id={}&state={}&response_type=code&scope=identify%20email&redirect_uri={}", client_id, state, redirect_uri)
            }
            AuthProvider::Microsoft => {
                let client_id = dotenvy::var("MICROSOFT_CLIENT_ID")?;

                format!("https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&scope=user.read&state={}&prompt=select_account&redirect_uri={}", client_id, state, redirect_uri)
            }
            AuthProvider::GitLab => {
                let client_id = dotenvy::var("GITLAB_CLIENT_ID")?;

                format!(
                    "https://gitlab.com/oauth/authorize?client_id={}&state={}&scope=read_user+profile+email&response_type=code&redirect_uri={}",
                    client_id,
                    state,
                    redirect_uri,
                )
            }
            AuthProvider::Google => {
                let client_id = dotenvy::var("GOOGLE_CLIENT_ID")?;

                format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&state={}&scope={}&response_type=code&redirect_uri={}",
                    client_id,
                    state,
                    urlencoding::encode("https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile"),
                    redirect_uri,
                )
            }
            AuthProvider::Steam => {
                format!(
                    "https://steamcommunity.com/openid/login?openid.ns={}&openid.mode={}&openid.return_to={}{}{}&openid.realm={}&openid.identity={}&openid.claimed_id={}",
                    urlencoding::encode("http://specs.openid.net/auth/2.0"),
                    "checkid_setup",
                    redirect_uri, urlencoding::encode("?state="), state,
                    self_addr,
                    "http://specs.openid.net/auth/2.0/identifier_select",
                    "http://specs.openid.net/auth/2.0/identifier_select",
                )
            }
        })
    }

    pub async fn get_token(
        &self,
        query: HashMap<String, String>,
    ) -> Result<String, AuthenticationError> {
        let redirect_uri = format!("{}/v2/auth/callback", dotenvy::var("SELF_ADDR")?);

        #[derive(Deserialize)]
        struct AccessToken {
            pub access_token: String,
        }

        let res = match self {
            AuthProvider::GitHub => {
                let code = query
                    .get("code")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                let client_id = dotenvy::var("GITHUB_CLIENT_ID")?;
                let client_secret = dotenvy::var("GITHUB_CLIENT_SECRET")?;

                let url = format!(
                    "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}&redirect_uri={}",
                    client_id, client_secret, code, redirect_uri
                );

                let token: AccessToken = reqwest::Client::new()
                    .post(&url)
                    .header(reqwest::header::ACCEPT, "application/json")
                    .send()
                    .await?
                    .json()
                    .await?;

                token.access_token
            }
            AuthProvider::Discord => {
                let code = query
                    .get("code")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                let client_id = dotenvy::var("DISCORD_CLIENT_ID")?;
                let client_secret = dotenvy::var("DISCORD_CLIENT_SECRET")?;

                let mut map = HashMap::new();
                map.insert("client_id", &*client_id);
                map.insert("client_secret", &*client_secret);
                map.insert("code", code);
                map.insert("grant_type", "authorization_code");
                map.insert("redirect_uri", &redirect_uri);

                let token: AccessToken = reqwest::Client::new()
                    .post("https://discord.com/api/v10/oauth2/token")
                    .header(reqwest::header::ACCEPT, "application/json")
                    .form(&map)
                    .send()
                    .await?
                    .json()
                    .await?;

                token.access_token
            }
            AuthProvider::Microsoft => {
                let code = query
                    .get("code")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                let client_id = dotenvy::var("MICROSOFT_CLIENT_ID")?;
                let client_secret = dotenvy::var("MICROSOFT_CLIENT_SECRET")?;

                let mut map = HashMap::new();
                map.insert("client_id", &*client_id);
                map.insert("client_secret", &*client_secret);
                map.insert("code", code);
                map.insert("grant_type", "authorization_code");
                map.insert("redirect_uri", &redirect_uri);

                let token: AccessToken = reqwest::Client::new()
                    .post("https://login.live.com/oauth20_token.srf")
                    .header(reqwest::header::ACCEPT, "application/json")
                    .form(&map)
                    .send()
                    .await?
                    .json()
                    .await?;

                token.access_token
            }
            AuthProvider::GitLab => {
                let code = query
                    .get("code")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                let client_id = dotenvy::var("GITLAB_CLIENT_ID")?;
                let client_secret = dotenvy::var("GITLAB_CLIENT_SECRET")?;

                let mut map = HashMap::new();
                map.insert("client_id", &*client_id);
                map.insert("client_secret", &*client_secret);
                map.insert("code", code);
                map.insert("grant_type", "authorization_code");
                map.insert("redirect_uri", &redirect_uri);

                let token: AccessToken = reqwest::Client::new()
                    .post("https://gitlab.com/oauth/token")
                    .header(reqwest::header::ACCEPT, "application/json")
                    .form(&map)
                    .send()
                    .await?
                    .json()
                    .await?;

                token.access_token
            }
            AuthProvider::Google => {
                let code = query
                    .get("code")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                let client_id = dotenvy::var("GOOGLE_CLIENT_ID")?;
                let client_secret = dotenvy::var("GOOGLE_CLIENT_SECRET")?;

                let mut map = HashMap::new();
                map.insert("client_id", &*client_id);
                map.insert("client_secret", &*client_secret);
                map.insert("code", code);
                map.insert("grant_type", "authorization_code");
                map.insert("redirect_uri", &redirect_uri);

                let token: AccessToken = reqwest::Client::new()
                    .post("https://oauth2.googleapis.com/token")
                    .header(reqwest::header::ACCEPT, "application/json")
                    .form(&map)
                    .send()
                    .await?
                    .json()
                    .await?;

                token.access_token
            }
            AuthProvider::Steam => {
                let mut form = HashMap::new();

                let signed = query
                    .get("openid.signed")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                form.insert(
                    "openid.assoc_handle".to_string(),
                    &**query
                        .get("openid.assoc_handle")
                        .ok_or_else(|| AuthenticationError::InvalidCredentials)?,
                );
                form.insert("openid.signed".to_string(), &**signed);
                form.insert(
                    "openid.sig".to_string(),
                    &**query
                        .get("openid.sig")
                        .ok_or_else(|| AuthenticationError::InvalidCredentials)?,
                );
                form.insert("openid.ns".to_string(), "http://specs.openid.net/auth/2.0");
                form.insert("openid.mode".to_string(), "check_authentication");

                for val in signed.split(',') {
                    if let Some(arr_val) = query.get(&format!("openid.{}", val)) {
                        form.insert(format!("openid.{}", val), &**arr_val);
                    }
                }

                let res = reqwest::Client::new()
                    .post("https://steamcommunity.com/openid/login")
                    .header("Accept-language", "en")
                    .form(&form)
                    .send()
                    .await?
                    .text()
                    .await?;

                if res.contains("is_valid:true") {
                    let identity = query
                        .get("openid.identity")
                        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

                    identity
                        .rsplit('/')
                        .next()
                        .ok_or_else(|| AuthenticationError::InvalidCredentials)?
                        .to_string()
                } else {
                    return Err(AuthenticationError::InvalidCredentials);
                }
            }
        };

        Ok(res)
    }

    pub async fn get_user(&self, token: &str) -> Result<TempUser, AuthenticationError> {
        let res = match self {
            AuthProvider::GitHub => {
                let response = reqwest::Client::new()
                    .get("https://api.github.com/user")
                    .header(reqwest::header::USER_AGENT, "Modrinth")
                    .header(AUTHORIZATION, format!("token {token}"))
                    .send()
                    .await?;

                if token.starts_with("gho_") {
                    let client_id = response
                        .headers()
                        .get("x-oauth-client-id")
                        .and_then(|x| x.to_str().ok());

                    if client_id != Some(&*dotenvy::var("GITHUB_CLIENT_ID").unwrap()) {
                        return Err(AuthenticationError::InvalidClientId);
                    }
                }

                #[derive(Serialize, Deserialize, Debug)]
                pub struct GitHubUser {
                    pub login: String,
                    pub id: u64,
                    pub avatar_url: String,
                    pub name: Option<String>,
                    pub email: Option<String>,
                    pub bio: Option<String>,
                }

                let github_user: GitHubUser = response.json().await?;

                TempUser {
                    id: github_user.id.to_string(),
                    username: github_user.login,
                    email: github_user.email,
                    avatar_url: Some(github_user.avatar_url),
                    bio: github_user.bio,
                    name: github_user.name,
                }
            }
            AuthProvider::Discord => {
                #[derive(Serialize, Deserialize, Debug)]
                pub struct DiscordUser {
                    pub username: String,
                    pub id: String,
                    pub avatar: Option<String>,
                    pub global_name: Option<String>,
                    pub email: Option<String>,
                }

                let discord_user: DiscordUser = reqwest::Client::new()
                    .get("https://discord.com/api/v10/users/@me")
                    .header(reqwest::header::USER_AGENT, "Modrinth")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .send()
                    .await?
                    .json()
                    .await?;

                let id = discord_user.id.clone();
                TempUser {
                    id: discord_user.id,
                    username: discord_user.username,
                    email: discord_user.email,
                    avatar_url: discord_user
                        .avatar
                        .map(|x| format!("https://cdn.discordapp.com/avatars/{}/{}.webp", id, x)),
                    bio: None,
                    name: discord_user.global_name,
                }
            }
            AuthProvider::Microsoft => {
                #[derive(Deserialize, Debug)]
                #[serde(rename_all = "camelCase")]
                pub struct MicrosoftUser {
                    pub id: String,
                    pub display_name: Option<String>,
                    pub mail: Option<String>,
                    pub user_principal_name: String,
                }

                let microsoft_user: MicrosoftUser = reqwest::Client::new()
                    .get("https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName")
                    .header(reqwest::header::USER_AGENT, "Modrinth")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .send()
                    .await?.json().await?;

                TempUser {
                    id: microsoft_user.id,
                    username: microsoft_user
                        .user_principal_name
                        .split('@')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                    email: microsoft_user.mail,
                    avatar_url: None,
                    bio: None,
                    name: microsoft_user.display_name,
                }
            }
            AuthProvider::GitLab => {
                #[derive(Serialize, Deserialize, Debug)]
                pub struct GitLabUser {
                    pub id: i32,
                    pub username: String,
                    pub email: Option<String>,
                    pub avatar_url: Option<String>,
                    pub name: Option<String>,
                    pub bio: Option<String>,
                }

                let gitlab_user: GitLabUser = reqwest::Client::new()
                    .get("https://gitlab.com/api/v4/user")
                    .header(reqwest::header::USER_AGENT, "Modrinth")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .send()
                    .await?
                    .json()
                    .await?;

                TempUser {
                    id: gitlab_user.id.to_string(),
                    username: gitlab_user.username,
                    email: gitlab_user.email,
                    avatar_url: gitlab_user.avatar_url,
                    bio: gitlab_user.bio,
                    name: gitlab_user.name,
                }
            }
            AuthProvider::Google => {
                #[derive(Deserialize, Debug)]
                pub struct GoogleUser {
                    pub id: String,
                    pub email: String,
                    pub name: Option<String>,
                    pub bio: Option<String>,
                    pub picture: Option<String>,
                }

                let google_user: GoogleUser = reqwest::Client::new()
                    .get("https://www.googleapis.com/userinfo/v2/me")
                    .header(reqwest::header::USER_AGENT, "Modrinth")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .send()
                    .await?
                    .json()
                    .await?;

                TempUser {
                    id: google_user.id,
                    username: google_user
                        .email
                        .split('@')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                    email: Some(google_user.email),
                    avatar_url: google_user.picture,
                    bio: None,
                    name: google_user.name,
                }
            }
            AuthProvider::Steam => {
                let api_key = dotenvy::var("STEAM_API_KEY")?;

                #[derive(Deserialize)]
                struct SteamResponse {
                    response: Players,
                }

                #[derive(Deserialize)]
                struct Players {
                    players: Vec<Player>,
                }

                #[derive(Deserialize)]
                struct Player {
                    steamid: String,
                    personaname: String,
                    profileurl: String,
                    avatar: Option<String>,
                }

                let response: String = reqwest::get(
                    &format!(
                        "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
                        api_key,
                        token
                    )
                )
                    .await?
                    .text()
                    .await?;

                let mut response: SteamResponse = serde_json::from_str(&response)?;

                if let Some(player) = response.response.players.pop() {
                    let username = player
                        .profileurl
                        .trim_matches('/')
                        .rsplit('/')
                        .next()
                        .unwrap_or(&player.steamid)
                        .to_string();
                    TempUser {
                        id: player.steamid,
                        username,
                        email: None,
                        avatar_url: player.avatar,
                        bio: None,
                        name: Some(player.personaname),
                    }
                } else {
                    return Err(AuthenticationError::InvalidCredentials);
                }
            }
        };

        Ok(res)
    }

    pub async fn get_user_id<'a, 'b, E>(
        &self,
        id: &str,
        executor: E,
    ) -> Result<Option<crate::database::models::UserId>, AuthenticationError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(match self {
            AuthProvider::GitHub => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE github_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::UserId(x.id))
            }
            AuthProvider::Discord => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE discord_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::UserId(x.id))
            }
            AuthProvider::Microsoft => {
                let value = sqlx::query!("SELECT id FROM users WHERE microsoft_id = $1", id)
                    .fetch_optional(executor)
                    .await?;

                value.map(|x| crate::database::models::UserId(x.id))
            }
            AuthProvider::GitLab => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE gitlab_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::UserId(x.id))
            }
            AuthProvider::Google => {
                let value = sqlx::query!("SELECT id FROM users WHERE google_id = $1", id)
                    .fetch_optional(executor)
                    .await?;

                value.map(|x| crate::database::models::UserId(x.id))
            }
            AuthProvider::Steam => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE steam_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::UserId(x.id))
            }
        })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthProvider::GitHub => "github",
            AuthProvider::Discord => "discord",
            AuthProvider::Microsoft => "microsoft",
            AuthProvider::GitLab => "gitlab",
            AuthProvider::Google => "google",
            AuthProvider::Steam => "steam",
        }
    }

    pub fn from_str(string: &str) -> AuthProvider {
        match string {
            "github" => AuthProvider::GitHub,
            "discord" => AuthProvider::Discord,
            "microsoft" => AuthProvider::Microsoft,
            "gitlab" => AuthProvider::GitLab,
            "google" => AuthProvider::Google,
            "steam" => AuthProvider::Steam,
            _ => AuthProvider::GitHub,
        }
    }
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_str())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthorizationInit {
    pub url: String,
    #[serde(default)]
    pub provider: AuthProvider,
}
#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub code: String,
    pub state: String,
}

// Init link takes us to GitHub API and calls back to callback endpoint with a code and state
// http://localhost:8000/auth/init?url=https://modrinth.com
#[get("init")]
pub async fn init(
    Query(info): Query<AuthorizationInit>, // callback url
    client: Data<PgPool>,
) -> Result<HttpResponse, AuthenticationError> {
    let url = url::Url::parse(&info.url).map_err(|_| AuthenticationError::Url)?;

    let allowed_callback_urls = parse_strings_from_var("ALLOWED_CALLBACK_URLS").unwrap_or_default();
    let domain = url.host_str().ok_or(AuthenticationError::Url)?;
    if !allowed_callback_urls.iter().any(|x| domain.ends_with(x)) && domain != "modrinth.com" {
        return Err(AuthenticationError::Url);
    }

    let mut transaction = client.begin().await?;

    let state = generate_state_id(&mut transaction).await?;

    sqlx::query!(
        "
        INSERT INTO states (id, url, provider)
        VALUES ($1, $2, $3)
        ",
        state.0,
        info.url,
        info.provider.to_string()
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    let url = info.provider.get_redirect_url(state)?;
    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", &*url))
        .json(serde_json::json!({ "url": url })))
}

#[get("callback")]
pub async fn auth_callback(
    req: HttpRequest,
    Query(query): Query<HashMap<String, String>>,
    client: Data<PgPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    redis: Data<deadpool_redis::Pool>,
) -> Result<HttpResponse, AuthenticationError> {
    let mut transaction = client.begin().await?;

    let state = query
        .get("state")
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
    let state_id: u64 = parse_base62(state)?;

    let result_option = sqlx::query!(
        "
        SELECT url, expires, provider FROM states
        WHERE id = $1
        ",
        state_id as i64
    )
    .fetch_optional(&mut *transaction)
    .await?;

    // Extract cookie header from request
    if let Some(result) = result_option {
        // Extract cookie header to get authenticated user from Minos
        let duration: chrono::Duration = result.expires - Utc::now();
        if duration.num_seconds() < 0 {
            return Err(AuthenticationError::InvalidCredentials);
        }
        sqlx::query!(
            "
            DELETE FROM states
            WHERE id = $1
            ",
            state_id as i64
        )
        .execute(&mut *transaction)
        .await?;

        let provider = AuthProvider::from_str(&result.provider);

        let token = provider.get_token(query).await?;
        let oauth_user = provider.get_user(&token).await?;
        let user_id = if let Some(user_id) = provider
            .get_user_id(&oauth_user.id, &mut *transaction)
            .await?
        {
            user_id
        } else {
            let user_id = crate::database::models::generate_user_id(&mut transaction).await?;

            let mut username_increment: i32 = 0;
            let mut username = None;

            while username.is_none() {
                let test_username = format!(
                    "{}{}",
                    oauth_user.username,
                    if username_increment > 0 {
                        username_increment.to_string()
                    } else {
                        "".to_string()
                    }
                );

                let new_id =
                    crate::database::models::User::get(&test_username, &**client, &redis).await?;

                if new_id.is_none() {
                    username = Some(test_username);
                } else {
                    username_increment += 1;
                }
            }

            let avatar_url = if let Some(avatar_url) = oauth_user.avatar_url {
                let cdn_url = dotenvy::var("CDN_URL")?;

                let res = reqwest::get(&avatar_url).await?;
                let headers = res.headers().clone();

                let img_data = if let Some(content_type) = headers
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|ct| ct.to_str().ok())
                {
                    get_image_ext(content_type).map(|ext| (ext, content_type))
                } else if let Some(ext) = avatar_url.rsplit('.').next() {
                    get_image_content_type(ext).map(|content_type| (ext, content_type))
                } else {
                    None
                };

                if let Some((ext, content_type)) = img_data {
                    let bytes = res.bytes().await?;
                    let hash = sha1::Sha1::from(&bytes).hexdigest();

                    let upload_data = file_host
                        .upload_file(
                            content_type,
                            &format!(
                                "user/{}/{}.{}",
                                crate::models::users::UserId::from(user_id),
                                hash,
                                ext
                            ),
                            bytes,
                        )
                        .await?;

                    Some(format!("{}/{}", cdn_url, upload_data.file_name))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(username) = username {
                crate::database::models::User {
                    id: user_id,
                    github_id: if provider == AuthProvider::GitHub {
                        Some(
                            oauth_user
                                .id
                                .clone()
                                .parse()
                                .map_err(|_| AuthenticationError::InvalidCredentials)?,
                        )
                    } else {
                        None
                    },
                    discord_id: if provider == AuthProvider::Discord {
                        Some(
                            oauth_user
                                .id
                                .parse()
                                .map_err(|_| AuthenticationError::InvalidCredentials)?,
                        )
                    } else {
                        None
                    },
                    gitlab_id: if provider == AuthProvider::GitLab {
                        Some(
                            oauth_user
                                .id
                                .parse()
                                .map_err(|_| AuthenticationError::InvalidCredentials)?,
                        )
                    } else {
                        None
                    },
                    google_id: if provider == AuthProvider::Google {
                        Some(oauth_user.id.clone())
                    } else {
                        None
                    },
                    steam_id: if provider == AuthProvider::Steam {
                        Some(
                            oauth_user
                                .id
                                .parse()
                                .map_err(|_| AuthenticationError::InvalidCredentials)?,
                        )
                    } else {
                        None
                    },
                    microsoft_id: if provider == AuthProvider::Microsoft {
                        Some(oauth_user.id)
                    } else {
                        None
                    },
                    username,
                    name: oauth_user.name,
                    email: oauth_user.email,
                    avatar_url,
                    bio: oauth_user.bio,
                    created: Utc::now(),
                    role: Role::Developer.to_string(),
                    badges: Badges::default(),
                    balance: Decimal::ZERO,
                    payout_wallet: None,
                    payout_wallet_type: None,
                    payout_address: None,
                }
                .insert(&mut transaction)
                .await?;

                user_id
            } else {
                return Err(AuthenticationError::InvalidCredentials);
            }
        };

        let session = issue_session(req, user_id, &mut transaction, &redis).await?;
        transaction.commit().await?;

        let redirect_url = if result.url.contains('?') {
            format!("{}&code={}", result.url, session.session)
        } else {
            format!("{}?code={}", result.url, session.session)
        };

        Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", &*redirect_url))
            .json(serde_json::json!({ "url": redirect_url })))
    } else {
        Err(AuthenticationError::InvalidCredentials)
    }
}
