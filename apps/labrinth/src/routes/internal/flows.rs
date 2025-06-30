use crate::auth::email::send_email;
use crate::auth::validate::get_user_record_from_bearer_token;
use crate::auth::{AuthProvider, AuthenticationError, get_user_from_headers};
use crate::database::models::DBUser;
use crate::database::models::flow_item::DBFlow;
use crate::database::redis::RedisPool;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::pats::Scopes;
use crate::models::users::{Badges, Role};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::internal::session::issue_session;
use crate::util::captcha::check_hcaptcha;
use crate::util::env::parse_strings_from_var;
use crate::util::ext::get_image_ext;
use crate::util::img::upload_image_optimized;
use crate::util::validate::validation_errors_to_string;
use actix_web::web::{Data, Query, ServiceConfig, scope};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use ariadne::ids::base62_impl::{parse_base62, to_base62};
use ariadne::ids::random_base62_rng;
use base64::Engine;
use chrono::{Duration, Utc};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use validator::Validate;
use zxcvbn::Score;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("auth")
            .service(init)
            .service(auth_callback)
            .service(delete_auth_provider)
            .service(create_account_with_password)
            .service(login_password)
            .service(login_2fa)
            .service(begin_2fa_flow)
            .service(finish_2fa_flow)
            .service(remove_2fa)
            .service(reset_password_begin)
            .service(change_password)
            .service(resend_verify_email)
            .service(set_email)
            .service(verify_email)
            .service(subscribe_newsletter)
            .service(get_newsletter_subscription_status),
    );
}

#[derive(Debug)]
pub struct TempUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,

    pub avatar_url: Option<String>,
    pub bio: Option<String>,

    pub country: Option<String>,
}

impl TempUser {
    async fn create_account(
        self,
        provider: AuthProvider,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        client: &PgPool,
        file_host: &Arc<dyn FileHost + Send + Sync>,
        redis: &RedisPool,
    ) -> Result<crate::database::models::DBUserId, AuthenticationError> {
        if let Some(email) = &self.email {
            if crate::database::models::DBUser::get_by_email(email, client)
                .await?
                .is_some()
            {
                return Err(AuthenticationError::DuplicateUser);
            }
        }

        let user_id =
            crate::database::models::generate_user_id(transaction).await?;

        let mut username_increment: i32 = 0;
        let mut username = None;

        while username.is_none() {
            let test_username = format!(
                "{}{}",
                self.username,
                if username_increment > 0 {
                    username_increment.to_string()
                } else {
                    "".to_string()
                }
            );

            let new_id = crate::database::models::DBUser::get(
                &test_username,
                client,
                redis,
            )
            .await?;

            if new_id.is_none() {
                username = Some(test_username);
            } else {
                username_increment += 1;
            }
        }

        let (avatar_url, raw_avatar_url) = if let Some(avatar_url) =
            self.avatar_url
        {
            let res = reqwest::get(&avatar_url).await?;
            let headers = res.headers().clone();

            let img_data = if let Some(content_type) = headers
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
            {
                get_image_ext(content_type)
            } else {
                avatar_url.rsplit('.').next()
            };

            if let Some(ext) = img_data {
                let bytes = res.bytes().await?;

                let upload_result = upload_image_optimized(
                    &format!("user/{}", ariadne::ids::UserId::from(user_id)),
                    FileHostPublicity::Public,
                    bytes,
                    ext,
                    Some(96),
                    Some(1.0),
                    &**file_host,
                )
                .await;

                if let Ok(upload_result) = upload_result {
                    (Some(upload_result.url), Some(upload_result.raw_url))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        if let Some(username) = username {
            crate::database::models::DBUser {
                id: user_id,
                github_id: if provider == AuthProvider::GitHub {
                    Some(
                        self.id.clone().parse().map_err(|_| {
                            AuthenticationError::InvalidCredentials
                        })?,
                    )
                } else {
                    None
                },
                discord_id: if provider == AuthProvider::Discord {
                    Some(
                        self.id.parse().map_err(|_| {
                            AuthenticationError::InvalidCredentials
                        })?,
                    )
                } else {
                    None
                },
                gitlab_id: if provider == AuthProvider::GitLab {
                    Some(
                        self.id.parse().map_err(|_| {
                            AuthenticationError::InvalidCredentials
                        })?,
                    )
                } else {
                    None
                },
                google_id: if provider == AuthProvider::Google {
                    Some(self.id.clone())
                } else {
                    None
                },
                steam_id: if provider == AuthProvider::Steam {
                    Some(
                        self.id.parse().map_err(|_| {
                            AuthenticationError::InvalidCredentials
                        })?,
                    )
                } else {
                    None
                },
                microsoft_id: if provider == AuthProvider::Microsoft {
                    Some(self.id.clone())
                } else {
                    None
                },
                password: None,
                paypal_id: if provider == AuthProvider::PayPal {
                    Some(self.id)
                } else {
                    None
                },
                paypal_country: self.country,
                paypal_email: if provider == AuthProvider::PayPal {
                    self.email.clone()
                } else {
                    None
                },
                venmo_handle: None,
                stripe_customer_id: None,
                totp_secret: None,
                username,
                email: self.email,
                email_verified: true,
                avatar_url,
                raw_avatar_url,
                bio: self.bio,
                created: Utc::now(),
                role: Role::Developer.to_string(),
                badges: Badges::default(),
                allow_friend_requests: true,
            }
            .insert(transaction)
            .await?;

            Ok(user_id)
        } else {
            Err(AuthenticationError::InvalidCredentials)
        }
    }
}

impl AuthProvider {
    pub fn get_redirect_url(
        &self,
        state: String,
    ) -> Result<String, AuthenticationError> {
        let self_addr = dotenvy::var("SELF_ADDR")?;
        let raw_redirect_uri = format!("{self_addr}/v2/auth/callback");
        let redirect_uri = urlencoding::encode(&raw_redirect_uri);

        Ok(match self {
            AuthProvider::GitHub => {
                let client_id = dotenvy::var("GITHUB_CLIENT_ID")?;

                format!(
                    "https://github.com/login/oauth/authorize?client_id={client_id}&prompt=select_account&state={state}&scope=read%3Auser%20user%3Aemail&redirect_uri={redirect_uri}",
                )
            }
            AuthProvider::Discord => {
                let client_id = dotenvy::var("DISCORD_CLIENT_ID")?;

                format!(
                    "https://discord.com/api/oauth2/authorize?client_id={client_id}&state={state}&response_type=code&scope=identify%20email&redirect_uri={redirect_uri}"
                )
            }
            AuthProvider::Microsoft => {
                let client_id = dotenvy::var("MICROSOFT_CLIENT_ID")?;

                format!(
                    "https://login.live.com/oauth20_authorize.srf?client_id={client_id}&response_type=code&scope=user.read&state={state}&prompt=select_account&redirect_uri={redirect_uri}"
                )
            }
            AuthProvider::GitLab => {
                let client_id = dotenvy::var("GITLAB_CLIENT_ID")?;

                format!(
                    "https://gitlab.com/oauth/authorize?client_id={client_id}&state={state}&scope=read_user+profile+email&response_type=code&redirect_uri={redirect_uri}",
                )
            }
            AuthProvider::Google => {
                let client_id = dotenvy::var("GOOGLE_CLIENT_ID")?;

                format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&state={}&scope={}&response_type=code&redirect_uri={}",
                    client_id,
                    state,
                    urlencoding::encode(
                        "https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile"
                    ),
                    redirect_uri,
                )
            }
            AuthProvider::Steam => {
                format!(
                    "https://steamcommunity.com/openid/login?openid.ns={}&openid.mode={}&openid.return_to={}{}{}&openid.realm={}&openid.identity={}&openid.claimed_id={}",
                    urlencoding::encode("http://specs.openid.net/auth/2.0"),
                    "checkid_setup",
                    redirect_uri,
                    urlencoding::encode("?state="),
                    state,
                    self_addr,
                    "http://specs.openid.net/auth/2.0/identifier_select",
                    "http://specs.openid.net/auth/2.0/identifier_select",
                )
            }
            AuthProvider::PayPal => {
                let api_url = dotenvy::var("PAYPAL_API_URL")?;
                let client_id = dotenvy::var("PAYPAL_CLIENT_ID")?;

                let auth_url = if api_url.contains("sandbox") {
                    "sandbox.paypal.com"
                } else {
                    "paypal.com"
                };

                format!(
                    "https://{auth_url}/connect?flowEntry=static&client_id={client_id}&scope={}&response_type=code&redirect_uri={redirect_uri}&state={state}",
                    urlencoding::encode(
                        "openid email address https://uri.paypal.com/services/paypalattributes"
                    ),
                )
            }
        })
    }

    pub async fn get_token(
        &self,
        query: HashMap<String, String>,
    ) -> Result<String, AuthenticationError> {
        let redirect_uri =
            format!("{}/v2/auth/callback", dotenvy::var("SELF_ADDR")?);

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
                    "https://github.com/login/oauth/access_token?client_id={client_id}&client_secret={client_secret}&code={code}&redirect_uri={redirect_uri}"
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
                    &**query.get("openid.assoc_handle").ok_or_else(|| {
                        AuthenticationError::InvalidCredentials
                    })?,
                );
                form.insert("openid.signed".to_string(), &**signed);
                form.insert(
                    "openid.sig".to_string(),
                    &**query.get("openid.sig").ok_or_else(|| {
                        AuthenticationError::InvalidCredentials
                    })?,
                );
                form.insert(
                    "openid.ns".to_string(),
                    "http://specs.openid.net/auth/2.0",
                );
                form.insert("openid.mode".to_string(), "check_authentication");

                for val in signed.split(',') {
                    if let Some(arr_val) = query.get(&format!("openid.{val}")) {
                        form.insert(format!("openid.{val}"), &**arr_val);
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
                    let identity =
                        query.get("openid.identity").ok_or_else(|| {
                            AuthenticationError::InvalidCredentials
                        })?;

                    identity
                        .rsplit('/')
                        .next()
                        .ok_or_else(|| AuthenticationError::InvalidCredentials)?
                        .to_string()
                } else {
                    return Err(AuthenticationError::InvalidCredentials);
                }
            }
            AuthProvider::PayPal => {
                let code = query
                    .get("code")
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
                let api_url = dotenvy::var("PAYPAL_API_URL")?;
                let client_id = dotenvy::var("PAYPAL_CLIENT_ID")?;
                let client_secret = dotenvy::var("PAYPAL_CLIENT_SECRET")?;

                let mut map = HashMap::new();
                map.insert("code", code.as_str());
                map.insert("grant_type", "authorization_code");

                let token: AccessToken = reqwest::Client::new()
                    .post(format!("{api_url}oauth2/token"))
                    .header(reqwest::header::ACCEPT, "application/json")
                    .header(
                        AUTHORIZATION,
                        format!(
                            "Basic {}",
                            base64::engine::general_purpose::STANDARD
                                .encode(format!("{client_id}:{client_secret}"))
                        ),
                    )
                    .form(&map)
                    .send()
                    .await?
                    .json()
                    .await?;

                token.access_token
            }
        };

        Ok(res)
    }

    pub async fn get_user(
        &self,
        token: &str,
    ) -> Result<TempUser, AuthenticationError> {
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

                    if client_id
                        != Some(&*dotenvy::var("GITHUB_CLIENT_ID").unwrap())
                    {
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
                    country: None,
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
                    avatar_url: discord_user.avatar.map(|x| {
                        format!(
                            "https://cdn.discordapp.com/avatars/{id}/{x}.webp"
                        )
                    }),
                    bio: None,
                    country: None,
                }
            }
            AuthProvider::Microsoft => {
                #[derive(Deserialize, Debug)]
                #[serde(rename_all = "camelCase")]
                pub struct MicrosoftUser {
                    pub id: String,
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
                    country: None,
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
                    country: None,
                }
            }
            AuthProvider::Google => {
                #[derive(Deserialize, Debug)]
                pub struct GoogleUser {
                    pub id: String,
                    pub email: String,
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
                    country: None,
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
                    profileurl: String,
                    avatar: Option<String>,
                }

                let response: String = reqwest::get(
                    &format!(
                        "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={api_key}&steamids={token}"
                    )
                )
                    .await?
                    .text()
                    .await?;

                let mut response: SteamResponse =
                    serde_json::from_str(&response)?;

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
                        country: None,
                    }
                } else {
                    return Err(AuthenticationError::InvalidCredentials);
                }
            }
            AuthProvider::PayPal => {
                #[derive(Deserialize, Debug)]
                pub struct PayPalUser {
                    pub payer_id: String,
                    pub email: String,
                    pub picture: Option<String>,
                    pub address: PayPalAddress,
                }

                #[derive(Deserialize, Debug)]
                pub struct PayPalAddress {
                    pub country: String,
                }

                let api_url = dotenvy::var("PAYPAL_API_URL")?;

                let paypal_user: PayPalUser = reqwest::Client::new()
                    .get(format!(
                        "{api_url}identity/openidconnect/userinfo?schema=openid"
                    ))
                    .header(reqwest::header::USER_AGENT, "Modrinth")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .send()
                    .await?
                    .json()
                    .await?;

                TempUser {
                    id: paypal_user.payer_id,
                    username: paypal_user
                        .email
                        .split('@')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                    email: Some(paypal_user.email),
                    avatar_url: paypal_user.picture,
                    bio: None,
                    country: Some(paypal_user.address.country),
                }
            }
        };

        Ok(res)
    }

    pub async fn get_user_id<'a, 'b, E>(
        &self,
        id: &str,
        executor: E,
    ) -> Result<Option<crate::database::models::DBUserId>, AuthenticationError>
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

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
            AuthProvider::Discord => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE discord_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
            AuthProvider::Microsoft => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE microsoft_id = $1",
                    id
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
            AuthProvider::GitLab => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE gitlab_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
            AuthProvider::Google => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE google_id = $1",
                    id
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
            AuthProvider::Steam => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE steam_id = $1",
                    id.parse::<i64>()
                        .map_err(|_| AuthenticationError::InvalidCredentials)?
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
            AuthProvider::PayPal => {
                let value = sqlx::query!(
                    "SELECT id FROM users WHERE paypal_id = $1",
                    id
                )
                .fetch_optional(executor)
                .await?;

                value.map(|x| crate::database::models::DBUserId(x.id))
            }
        })
    }

    pub async fn update_user_id(
        &self,
        user_id: crate::database::models::DBUserId,
        id: Option<&str>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AuthenticationError> {
        match self {
            AuthProvider::GitHub => {
                sqlx::query!(
                    "
                    UPDATE users
                    SET github_id = $2
                    WHERE (id = $1)
                    ",
                    user_id as crate::database::models::DBUserId,
                    id.and_then(|x| x.parse::<i64>().ok())
                )
                .execute(&mut **transaction)
                .await?;
            }
            AuthProvider::Discord => {
                sqlx::query!(
                    "
                    UPDATE users
                    SET discord_id = $2
                    WHERE (id = $1)
                    ",
                    user_id as crate::database::models::DBUserId,
                    id.and_then(|x| x.parse::<i64>().ok())
                )
                .execute(&mut **transaction)
                .await?;
            }
            AuthProvider::Microsoft => {
                sqlx::query!(
                    "
                    UPDATE users
                    SET microsoft_id = $2
                    WHERE (id = $1)
                    ",
                    user_id as crate::database::models::DBUserId,
                    id,
                )
                .execute(&mut **transaction)
                .await?;
            }
            AuthProvider::GitLab => {
                sqlx::query!(
                    "
                    UPDATE users
                    SET gitlab_id = $2
                    WHERE (id = $1)
                    ",
                    user_id as crate::database::models::DBUserId,
                    id.and_then(|x| x.parse::<i64>().ok())
                )
                .execute(&mut **transaction)
                .await?;
            }
            AuthProvider::Google => {
                sqlx::query!(
                    "
                    UPDATE users
                    SET google_id = $2
                    WHERE (id = $1)
                    ",
                    user_id as crate::database::models::DBUserId,
                    id,
                )
                .execute(&mut **transaction)
                .await?;
            }
            AuthProvider::Steam => {
                sqlx::query!(
                    "
                    UPDATE users
                    SET steam_id = $2
                    WHERE (id = $1)
                    ",
                    user_id as crate::database::models::DBUserId,
                    id.and_then(|x| x.parse::<i64>().ok())
                )
                .execute(&mut **transaction)
                .await?;
            }
            AuthProvider::PayPal => {
                if id.is_none() {
                    sqlx::query!(
                        "
                        UPDATE users
                        SET paypal_country = NULL, paypal_email = NULL, paypal_id = NULL
                        WHERE (id = $1)
                        ",
                        user_id as crate::database::models::DBUserId,
                    )
                    .execute(&mut **transaction)
                    .await?;
                } else {
                    sqlx::query!(
                        "
                        UPDATE users
                        SET paypal_id = $2
                        WHERE (id = $1)
                        ",
                        user_id as crate::database::models::DBUserId,
                        id,
                    )
                    .execute(&mut **transaction)
                    .await?;
                }
            }
        }

        Ok(())
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthProvider::GitHub => "GitHub",
            AuthProvider::Discord => "Discord",
            AuthProvider::Microsoft => "Microsoft",
            AuthProvider::GitLab => "GitLab",
            AuthProvider::Google => "Google",
            AuthProvider::Steam => "Steam",
            AuthProvider::PayPal => "PayPal",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthorizationInit {
    pub url: String,
    #[serde(default)]
    pub provider: AuthProvider,
    pub token: Option<String>,
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
    req: HttpRequest,
    Query(info): Query<AuthorizationInit>, // callback url
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, AuthenticationError> {
    let url =
        url::Url::parse(&info.url).map_err(|_| AuthenticationError::Url)?;

    let allowed_callback_urls =
        parse_strings_from_var("ALLOWED_CALLBACK_URLS").unwrap_or_default();
    let domain = url.host_str().ok_or(AuthenticationError::Url)?;
    if !allowed_callback_urls.iter().any(|x| domain.ends_with(x))
        && domain != "modrinth.com"
    {
        return Err(AuthenticationError::Url);
    }

    let user_id = if let Some(token) = info.token {
        let (_, user) = get_user_record_from_bearer_token(
            &req,
            Some(&token),
            &**client,
            &redis,
            &session_queue,
        )
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

        Some(user.id)
    } else {
        None
    };

    let state = DBFlow::OAuth {
        user_id,
        url: info.url,
        provider: info.provider,
    }
    .insert(Duration::minutes(30), &redis)
    .await?;

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
    redis: Data<RedisPool>,
) -> Result<HttpResponse, crate::auth::templates::ErrorPage> {
    let state_string = query
        .get("state")
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?
        .clone();

    let state = state_string.clone();
    let res: Result<HttpResponse, AuthenticationError> = async move {
        let flow = DBFlow::get(&state, &redis).await?;

        // Extract cookie header from request
        if let Some(DBFlow::OAuth {
                        user_id,
                        provider,
                        url,
                    }) = flow
        {
            DBFlow::remove(&state, &redis).await?;

            let token = provider.get_token(query).await?;
            let oauth_user = provider.get_user(&token).await?;

            let user_id_opt = provider.get_user_id(&oauth_user.id, &**client).await?;

            let mut transaction = client.begin().await?;
            if let Some(id) = user_id {
                if user_id_opt.is_some() {
                    return Err(AuthenticationError::DuplicateUser);
                }

                provider
                    .update_user_id(id, Some(&oauth_user.id), &mut transaction)
                    .await?;

                let user = crate::database::models::DBUser::get_id(id, &**client, &redis).await?;

                if provider == AuthProvider::PayPal  {
                    sqlx::query!(
                        "
                        UPDATE users
                        SET paypal_country = $1, paypal_email = $2, paypal_id = $3
                        WHERE (id = $4)
                        ",
                        oauth_user.country,
                        oauth_user.email,
                        oauth_user.id,
                        id as crate::database::models::ids::DBUserId,
                    )
                        .execute(&mut *transaction)
                        .await?;
                } else if let Some(email) = user.and_then(|x| x.email) {
                    send_email(
                        email,
                        "Authentication method added",
                        &format!("When logging into Modrinth, you can now log in using the {} authentication provider.", provider.as_str()),
                        "If you did not make this change, please contact us immediately through our support channels on Discord or via email (support@modrinth.com).",
                        None,
                    )?;
                }

                transaction.commit().await?;
                crate::database::models::DBUser::clear_caches(&[(id, None)], &redis).await?;

                Ok(HttpResponse::TemporaryRedirect()
                    .append_header(("Location", &*url))
                    .json(serde_json::json!({ "url": url })))
            } else {
                let user_id = if let Some(user_id) = user_id_opt {
                    let user = crate::database::models::DBUser::get_id(user_id, &**client, &redis)
                        .await?
                        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

                    if user.totp_secret.is_some() {
                        let flow = DBFlow::Login2FA { user_id: user.id }
                            .insert(Duration::minutes(30), &redis)
                            .await?;

                        let redirect_url = format!(
                            "{}{}error=2fa_required&flow={}",
                            url,
                            if url.contains('?') { "&" } else { "?" },
                            flow
                        );

                        return Ok(HttpResponse::TemporaryRedirect()
                            .append_header(("Location", &*redirect_url))
                            .json(serde_json::json!({ "url": redirect_url })));
                    }

                    user_id
                } else {
                    oauth_user.create_account(provider, &mut transaction, &client, &file_host, &redis).await?
                };

                let session = issue_session(req, user_id, &mut transaction, &redis).await?;
                transaction.commit().await?;

                let redirect_url = format!(
                    "{}{}code={}{}",
                    url,
                    if url.contains('?') { '&' } else { '?' },
                    session.session,
                    if user_id_opt.is_none() {
                        "&new_account=true"
                    } else {
                        ""
                    }
                );

                Ok(HttpResponse::TemporaryRedirect()
                    .append_header(("Location", &*redirect_url))
                    .json(serde_json::json!({ "url": redirect_url })))
            }
        } else {
            Err::<HttpResponse, AuthenticationError>(AuthenticationError::InvalidCredentials)
        }
    }.await;

    Ok(res?)
}

#[derive(Deserialize)]
pub struct DeleteAuthProvider {
    pub provider: AuthProvider,
}

#[delete("provider")]
pub async fn delete_auth_provider(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    delete_provider: web::Json<DeleteAuthProvider>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_AUTH_WRITE,
    )
    .await?
    .1;

    if user.auth_providers.is_none_or(|x| x.len() <= 1)
        && !user.has_password.unwrap_or(false)
    {
        return Err(ApiError::InvalidInput(
            "You must have another authentication method added to this account!".to_string(),
        ));
    }

    let mut transaction = pool.begin().await?;

    delete_provider
        .provider
        .update_user_id(user.id.into(), None, &mut transaction)
        .await?;

    if delete_provider.provider != AuthProvider::PayPal {
        if let Some(email) = user.email {
            send_email(
                email,
                "Authentication method removed",
                &format!(
                    "When logging into Modrinth, you can no longer log in using the {} authentication provider.",
                    delete_provider.provider.as_str()
                ),
                "If you did not make this change, please contact us immediately through our support channels on Discord or via email (support@modrinth.com).",
                None,
            )?;
        }
    }

    transaction.commit().await?;
    crate::database::models::DBUser::clear_caches(
        &[(user.id.into(), None)],
        &redis,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn sign_up_sendy(email: &str) -> Result<(), AuthenticationError> {
    let url = dotenvy::var("SENDY_URL")?;
    let id = dotenvy::var("SENDY_LIST_ID")?;
    let api_key = dotenvy::var("SENDY_API_KEY")?;
    let site_url = dotenvy::var("SITE_URL")?;

    if url.is_empty() || url == "none" {
        tracing::info!("Sendy URL not set, skipping signup");
        return Ok(());
    }

    let mut form = HashMap::new();

    form.insert("api_key", &*api_key);
    form.insert("email", email);
    form.insert("list", &*id);
    form.insert("referrer", &*site_url);

    let client = reqwest::Client::new();
    client
        .post(format!("{url}/subscribe"))
        .form(&form)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(())
}

pub async fn check_sendy_subscription(
    email: &str,
) -> Result<bool, AuthenticationError> {
    let url = dotenvy::var("SENDY_URL")?;
    let id = dotenvy::var("SENDY_LIST_ID")?;
    let api_key = dotenvy::var("SENDY_API_KEY")?;

    if url.is_empty() || url == "none" {
        tracing::info!(
            "Sendy URL not set, returning false for subscription check"
        );
        return Ok(false);
    }

    let mut form = HashMap::new();
    form.insert("api_key", &*api_key);
    form.insert("email", email);
    form.insert("list_id", &*id);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{url}/api/subscribers/subscription-status.php"))
        .form(&form)
        .send()
        .await?
        .text()
        .await?;

    Ok(response.trim() == "Subscribed")
}

#[derive(Deserialize, Validate)]
pub struct NewAccount {
    #[validate(length(min = 1, max = 39), regex(path = *crate::util::validate::RE_URL_SAFE))]
    pub username: String,
    #[validate(length(min = 8, max = 256))]
    pub password: String,
    #[validate(email)]
    pub email: String,
    pub challenge: String,
    pub sign_up_newsletter: Option<bool>,
}

#[post("create")]
pub async fn create_account_with_password(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    new_account: web::Json<NewAccount>,
) -> Result<HttpResponse, ApiError> {
    new_account.0.validate().map_err(|err| {
        ApiError::InvalidInput(validation_errors_to_string(err, None))
    })?;

    if !check_hcaptcha(&req, &new_account.challenge).await? {
        return Err(ApiError::Turnstile);
    }

    if crate::database::models::DBUser::get(
        &new_account.username,
        &**pool,
        &redis,
    )
    .await?
    .is_some()
    {
        return Err(ApiError::InvalidInput("Username is taken!".to_string()));
    }

    let mut transaction = pool.begin().await?;
    let user_id =
        crate::database::models::generate_user_id(&mut transaction).await?;

    let new_account = new_account.0;

    let score = zxcvbn::zxcvbn(
        &new_account.password,
        &[&new_account.username, &new_account.email],
    );

    if score.score() < Score::Three {
        return Err(ApiError::InvalidInput(
            if let Some(feedback) = score.feedback().and_then(|x| x.warning()) {
                format!("Password too weak: {feedback}")
            } else {
                "Specified password is too weak! Please improve its strength."
                    .to_string()
            },
        ));
    }

    let hasher = Argon2::default();
    let salt = SaltString::generate(&mut ChaCha20Rng::from_entropy());
    let password_hash = hasher
        .hash_password(new_account.password.as_bytes(), &salt)?
        .to_string();

    if crate::database::models::DBUser::get_by_email(
        &new_account.email,
        &**pool,
    )
    .await?
    .is_some()
    {
        return Err(ApiError::InvalidInput(
            "Email is already registered on Modrinth!".to_string(),
        ));
    }

    crate::database::models::DBUser {
        id: user_id,
        github_id: None,
        discord_id: None,
        gitlab_id: None,
        google_id: None,
        steam_id: None,
        microsoft_id: None,
        password: Some(password_hash),
        paypal_id: None,
        paypal_country: None,
        paypal_email: None,
        venmo_handle: None,
        stripe_customer_id: None,
        totp_secret: None,
        username: new_account.username.clone(),
        email: Some(new_account.email.clone()),
        email_verified: false,
        avatar_url: None,
        raw_avatar_url: None,
        bio: None,
        created: Utc::now(),
        role: Role::Developer.to_string(),
        badges: Badges::default(),
        allow_friend_requests: true,
    }
    .insert(&mut transaction)
    .await?;

    let session = issue_session(req, user_id, &mut transaction, &redis).await?;
    let res = crate::models::sessions::Session::from(session, true, None);

    let flow = DBFlow::ConfirmEmail {
        user_id,
        confirm_email: new_account.email.clone(),
    }
    .insert(Duration::hours(24), &redis)
    .await?;

    send_email_verify(
        new_account.email.clone(),
        flow,
        &format!("Welcome to Modrinth, {}!", new_account.username),
    )?;

    if new_account.sign_up_newsletter.unwrap_or(false) {
        sign_up_sendy(&new_account.email).await?;
    }

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(res))
}

#[derive(Deserialize, Validate)]
pub struct Login {
    #[serde(rename = "username")]
    pub username_or_email: String,
    pub password: String,
    pub challenge: String,
}

#[post("login")]
pub async fn login_password(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    login: web::Json<Login>,
) -> Result<HttpResponse, ApiError> {
    if !check_hcaptcha(&req, &login.challenge).await? {
        return Err(ApiError::Turnstile);
    }

    let user = if let Some(user) = crate::database::models::DBUser::get(
        &login.username_or_email,
        &**pool,
        &redis,
    )
    .await?
    {
        user
    } else {
        let user = crate::database::models::DBUser::get_by_email(
            &login.username_or_email,
            &**pool,
        )
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

        crate::database::models::DBUser::get_id(user, &**pool, &redis)
            .await?
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?
    };

    let hasher = Argon2::default();
    hasher
        .verify_password(
            login.password.as_bytes(),
            &PasswordHash::new(
                &user
                    .password
                    .ok_or_else(|| AuthenticationError::InvalidCredentials)?,
            )?,
        )
        .map_err(|_| AuthenticationError::InvalidCredentials)?;

    if user.totp_secret.is_some() {
        let flow = DBFlow::Login2FA { user_id: user.id }
            .insert(Duration::minutes(30), &redis)
            .await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "error": "2fa_required",
            "description": "2FA is required to complete this operation.",
            "flow": flow,
        })))
    } else {
        let mut transaction = pool.begin().await?;
        let session =
            issue_session(req, user.id, &mut transaction, &redis).await?;
        let res = crate::models::sessions::Session::from(session, true, None);
        transaction.commit().await?;

        Ok(HttpResponse::Ok().json(res))
    }
}

#[derive(Deserialize, Validate)]
pub struct Login2FA {
    pub code: String,
    pub flow: String,
}

async fn validate_2fa_code(
    input: String,
    secret: String,
    allow_backup: bool,
    user_id: crate::database::models::DBUserId,
    redis: &RedisPool,
    pool: &PgPool,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<bool, AuthenticationError> {
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Encoded(secret)
            .to_bytes()
            .map_err(|_| AuthenticationError::InvalidCredentials)?,
    )
    .map_err(|_| AuthenticationError::InvalidCredentials)?;

    const TOTP_NAMESPACE: &str = "used_totp";
    let mut conn = redis.connect().await?;

    // Check if TOTP has already been used
    if conn
        .get(TOTP_NAMESPACE, &format!("{}-{}", input, user_id.0))
        .await?
        .is_some()
    {
        return Err(AuthenticationError::InvalidCredentials);
    }

    if totp
        .check_current(input.as_str())
        .map_err(|_| AuthenticationError::InvalidCredentials)?
    {
        conn.set(
            TOTP_NAMESPACE,
            &format!("{}-{}", input, user_id.0),
            "",
            Some(60),
        )
        .await?;

        Ok(true)
    } else if allow_backup {
        let backup_codes =
            crate::database::models::DBUser::get_backup_codes(user_id, pool)
                .await?;

        if !backup_codes.contains(&input) {
            Ok(false)
        } else {
            let code = parse_base62(&input).unwrap_or_default();

            sqlx::query!(
                "
                    DELETE FROM user_backup_codes
                    WHERE user_id = $1 AND code = $2
                    ",
                user_id as crate::database::models::ids::DBUserId,
                code as i64,
            )
            .execute(&mut **transaction)
            .await?;

            crate::database::models::DBUser::clear_caches(
                &[(user_id, None)],
                redis,
            )
            .await?;

            Ok(true)
        }
    } else {
        Err(AuthenticationError::InvalidCredentials)
    }
}

#[post("login/2fa")]
pub async fn login_2fa(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    login: web::Json<Login2FA>,
) -> Result<HttpResponse, ApiError> {
    let flow = DBFlow::get(&login.flow, &redis)
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    if let DBFlow::Login2FA { user_id } = flow {
        let user =
            crate::database::models::DBUser::get_id(user_id, &**pool, &redis)
                .await?
                .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

        let mut transaction = pool.begin().await?;
        if !validate_2fa_code(
            login.code.clone(),
            user.totp_secret
                .ok_or_else(|| AuthenticationError::InvalidCredentials)?,
            true,
            user.id,
            &redis,
            &pool,
            &mut transaction,
        )
        .await?
        {
            return Err(ApiError::Authentication(
                AuthenticationError::InvalidCredentials,
            ));
        }
        DBFlow::remove(&login.flow, &redis).await?;

        let session =
            issue_session(req, user_id, &mut transaction, &redis).await?;
        let res = crate::models::sessions::Session::from(session, true, None);
        transaction.commit().await?;

        Ok(HttpResponse::Ok().json(res))
    } else {
        Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ))
    }
}

#[post("2fa/get_secret")]
pub async fn begin_2fa_flow(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_AUTH_WRITE,
    )
    .await?
    .1;

    if !user.has_totp.unwrap_or(false) {
        let string = totp_rs::Secret::generate_secret();
        let encoded = string.to_encoded();

        let flow = DBFlow::Initialize2FA {
            user_id: user.id.into(),
            secret: encoded.to_string(),
        }
        .insert(Duration::minutes(30), &redis)
        .await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "secret": encoded.to_string(),
            "flow": flow,
        })))
    } else {
        Err(ApiError::InvalidInput(
            "User already has 2FA enabled on their account!".to_string(),
        ))
    }
}

#[post("2fa")]
pub async fn finish_2fa_flow(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    login: web::Json<Login2FA>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let flow = DBFlow::get(&login.flow, &redis)
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    if let DBFlow::Initialize2FA { user_id, secret } = flow {
        let user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Scopes::USER_AUTH_WRITE,
        )
        .await?
        .1;

        if user.id != user_id.into() {
            return Err(ApiError::Authentication(
                AuthenticationError::InvalidCredentials,
            ));
        }

        let mut transaction = pool.begin().await?;

        if !validate_2fa_code(
            login.code.clone(),
            secret.clone(),
            false,
            user.id.into(),
            &redis,
            &pool,
            &mut transaction,
        )
        .await?
        {
            return Err(ApiError::Authentication(
                AuthenticationError::InvalidCredentials,
            ));
        }

        DBFlow::remove(&login.flow, &redis).await?;

        sqlx::query!(
            "
            UPDATE users
            SET totp_secret = $1
            WHERE (id = $2)
            ",
            secret,
            user_id as crate::database::models::ids::DBUserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM user_backup_codes
            WHERE user_id = $1
            ",
            user_id as crate::database::models::ids::DBUserId,
        )
        .execute(&mut *transaction)
        .await?;

        let mut codes = Vec::new();

        for _ in 0..6 {
            let mut rng = ChaCha20Rng::from_entropy();
            let val = random_base62_rng(&mut rng, 11);

            sqlx::query!(
                "
                INSERT INTO user_backup_codes (
                    user_id, code
                )
                VALUES (
                    $1, $2
                )
                ",
                user_id as crate::database::models::ids::DBUserId,
                val as i64,
            )
            .execute(&mut *transaction)
            .await?;

            codes.push(to_base62(val));
        }

        if let Some(email) = user.email {
            send_email(
                email,
                "Two-factor authentication enabled",
                "When logging into Modrinth, you can now enter a code generated by your authenticator app in addition to entering your usual email address and password.",
                "If you did not make this change, please contact us immediately through our support channels on Discord or via email (support@modrinth.com).",
                None,
            )?;
        }

        transaction.commit().await?;
        crate::database::models::DBUser::clear_caches(
            &[(user.id.into(), None)],
            &redis,
        )
        .await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "backup_codes": codes,
        })))
    } else {
        Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ))
    }
}

#[derive(Deserialize)]
pub struct Remove2FA {
    pub code: String,
}

#[delete("2fa")]
pub async fn remove_2fa(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    login: web::Json<Remove2FA>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (scopes, user) = get_user_record_from_bearer_token(
        &req,
        None,
        &**pool,
        &redis,
        &session_queue,
    )
    .await?
    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    if !scopes.contains(Scopes::USER_AUTH_WRITE) {
        return Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ));
    }

    let mut transaction = pool.begin().await?;

    if !validate_2fa_code(
        login.code.clone(),
        user.totp_secret.ok_or_else(|| {
            ApiError::InvalidInput(
                "User does not have 2FA enabled on the account!".to_string(),
            )
        })?,
        true,
        user.id,
        &redis,
        &pool,
        &mut transaction,
    )
    .await?
    {
        return Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ));
    }

    sqlx::query!(
        "
        UPDATE users
        SET totp_secret = NULL
        WHERE (id = $1)
        ",
        user.id as crate::database::models::ids::DBUserId,
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query!(
        "
        DELETE FROM user_backup_codes
        WHERE user_id = $1
        ",
        user.id as crate::database::models::ids::DBUserId,
    )
    .execute(&mut *transaction)
    .await?;

    if let Some(email) = user.email {
        send_email(
            email,
            "Two-factor authentication removed",
            "When logging into Modrinth, you no longer need two-factor authentication to gain access.",
            "If you did not make this change, please contact us immediately through our support channels on Discord or via email (support@modrinth.com).",
            None,
        )?;
    }

    transaction.commit().await?;
    crate::database::models::DBUser::clear_caches(&[(user.id, None)], &redis)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct ResetPassword {
    #[serde(rename = "username")]
    pub username_or_email: String,
    pub challenge: String,
}

#[post("password/reset")]
pub async fn reset_password_begin(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    reset_password: web::Json<ResetPassword>,
) -> Result<HttpResponse, ApiError> {
    if !check_hcaptcha(&req, &reset_password.challenge).await? {
        return Err(ApiError::Turnstile);
    }

    let user =
        match crate::database::models::DBUser::get_by_case_insensitive_email(
            &reset_password.username_or_email,
            &**pool,
        )
        .await?[..]
        {
            [] => {
                // Try finding by username or ID
                crate::database::models::DBUser::get(
                    &reset_password.username_or_email,
                    &**pool,
                    &redis,
                )
                .await?
            }
            [user_id] => {
                // If there is only one user with the given email, ignoring case,
                // we can assume it's the user we want to reset the password for
                crate::database::models::DBUser::get_id(
                    user_id, &**pool, &redis,
                )
                .await?
            }
            _ => {
                // When several users use variations of the same email with
                // different cases, we cannot reliably tell which user should
                // receive the password reset email, so fall back to case sensitive
                // search to avoid spamming multiple users
                if let Some(user_id) =
                    crate::database::models::DBUser::get_by_email(
                        &reset_password.username_or_email,
                        &**pool,
                    )
                    .await?
                {
                    crate::database::models::DBUser::get_id(
                        user_id, &**pool, &redis,
                    )
                    .await?
                } else {
                    None
                }
            }
        };

    if let Some(DBUser {
        id: user_id,
        email: Some(email),
        ..
    }) = user
    {
        let flow = DBFlow::ForgotPassword { user_id }
            .insert(Duration::hours(24), &redis)
            .await?;

        send_email(
            email,
            "Reset your password",
            "Please visit the following link below to reset your password. If the button does not work, you can copy the link and paste it into your browser.",
            "If you did not request for your password to be reset, you can safely ignore this email.",
            Some((
                "Reset password",
                &format!(
                    "{}/{}?flow={}",
                    dotenvy::var("SITE_URL")?,
                    dotenvy::var("SITE_RESET_PASSWORD_PATH")?,
                    flow
                ),
            )),
        )?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, Validate)]
pub struct ChangePassword {
    pub flow: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
}

#[patch("password")]
pub async fn change_password(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    change_password: web::Json<ChangePassword>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = if let Some(flow) = &change_password.flow {
        let flow = DBFlow::get(flow, &redis).await?;

        if let Some(DBFlow::ForgotPassword { user_id }) = flow {
            let user = crate::database::models::DBUser::get_id(
                user_id, &**pool, &redis,
            )
            .await?
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

            Some(user)
        } else {
            return Err(ApiError::CustomAuthentication(
                "The password change flow code is invalid or has expired. Did you copy it promptly and correctly?".to_string(),
            ));
        }
    } else {
        None
    };

    let user = if let Some(user) = user {
        user
    } else {
        let (scopes, user) = get_user_record_from_bearer_token(
            &req,
            None,
            &**pool,
            &redis,
            &session_queue,
        )
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

        if !scopes.contains(Scopes::USER_AUTH_WRITE) {
            return Err(ApiError::Authentication(
                AuthenticationError::InvalidCredentials,
            ));
        }

        if let Some(pass) = user.password.as_ref() {
            let old_password = change_password.old_password.as_ref().ok_or_else(|| {
                ApiError::CustomAuthentication(
                    "You must specify the old password to change your password!".to_string(),
                )
            })?;

            let hasher = Argon2::default();
            hasher.verify_password(
                old_password.as_bytes(),
                &PasswordHash::new(pass)?,
            )?;
        }

        user
    };

    let mut transaction = pool.begin().await?;

    let update_password = if let Some(new_password) =
        &change_password.new_password
    {
        let score = zxcvbn::zxcvbn(
            new_password,
            &[&user.username, &user.email.clone().unwrap_or_default()],
        );

        if score.score() < Score::Three {
            return Err(ApiError::InvalidInput(
                if let Some(feedback) =
                    score.feedback().and_then(|x| x.warning())
                {
                    format!("Password too weak: {feedback}")
                } else {
                    "Specified password is too weak! Please improve its strength.".to_string()
                },
            ));
        }

        let hasher = Argon2::default();
        let salt = SaltString::generate(&mut ChaCha20Rng::from_entropy());
        let password_hash = hasher
            .hash_password(new_password.as_bytes(), &salt)?
            .to_string();

        Some(password_hash)
    } else {
        if !(user.github_id.is_some()
            || user.gitlab_id.is_some()
            || user.microsoft_id.is_some()
            || user.google_id.is_some()
            || user.steam_id.is_some()
            || user.discord_id.is_some())
        {
            return Err(ApiError::InvalidInput(
                "You must have another authentication method added to remove password authentication!".to_string(),
            ));
        }

        None
    };

    sqlx::query!(
        "
        UPDATE users
        SET password = $1
        WHERE (id = $2)
        ",
        update_password,
        user.id as crate::database::models::ids::DBUserId,
    )
    .execute(&mut *transaction)
    .await?;

    if let Some(flow) = &change_password.flow {
        DBFlow::remove(flow, &redis).await?;
    }

    if let Some(email) = user.email {
        let changed = if update_password.is_some() {
            "changed"
        } else {
            "removed"
        };

        send_email(
            email,
            &format!("Password {changed}"),
            &format!("Your password has been {changed} on your account."),
            "If you did not make this change, please contact us immediately through our support channels on Discord or via email (support@modrinth.com).",
            None,
        )?;
    }

    transaction.commit().await?;
    crate::database::models::DBUser::clear_caches(&[(user.id, None)], &redis)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, Validate)]
pub struct SetEmail {
    #[validate(email)]
    pub email: String,
}

#[patch("email")]
pub async fn set_email(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    email: web::Json<SetEmail>,
    session_queue: Data<AuthQueue>,
    stripe_client: Data<stripe::Client>,
) -> Result<HttpResponse, ApiError> {
    email.0.validate().map_err(|err| {
        ApiError::InvalidInput(validation_errors_to_string(err, None))
    })?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_AUTH_WRITE,
    )
    .await?
    .1;

    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        UPDATE users
        SET email = $1, email_verified = FALSE
        WHERE (id = $2)
        ",
        email.email,
        user.id.0 as i64,
    )
    .execute(&mut *transaction)
    .await?;

    if let Some(user_email) = user.email {
        send_email(
            user_email,
            "Email changed",
            &format!(
                "Your email has been updated to {} on your account.",
                email.email
            ),
            "If you did not make this change, please contact us immediately through our support channels on Discord or via email (support@modrinth.com).",
            None,
        )?;
    }

    if let Some(customer_id) = user
        .stripe_customer_id
        .as_ref()
        .and_then(|x| stripe::CustomerId::from_str(x).ok())
    {
        stripe::Customer::update(
            &stripe_client,
            &customer_id,
            stripe::UpdateCustomer {
                email: Some(&email.email),
                ..Default::default()
            },
        )
        .await?;
    }

    let flow = DBFlow::ConfirmEmail {
        user_id: user.id.into(),
        confirm_email: email.email.clone(),
    }
    .insert(Duration::hours(24), &redis)
    .await?;

    send_email_verify(
        email.email.clone(),
        flow,
        "We need to verify your email address.",
    )?;

    transaction.commit().await?;
    crate::database::models::DBUser::clear_caches(
        &[(user.id.into(), None)],
        &redis,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[post("email/resend_verify")]
pub async fn resend_verify_email(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_AUTH_WRITE,
    )
    .await?
    .1;

    if let Some(email) = user.email {
        if user.email_verified.unwrap_or(false) {
            return Err(ApiError::InvalidInput(
                "User email is already verified!".to_string(),
            ));
        }

        let flow = DBFlow::ConfirmEmail {
            user_id: user.id.into(),
            confirm_email: email.clone(),
        }
        .insert(Duration::hours(24), &redis)
        .await?;

        send_email_verify(
            email,
            flow,
            "We need to verify your email address.",
        )?;

        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::InvalidInput(
            "User does not have an email.".to_string(),
        ))
    }
}

#[derive(Deserialize)]
pub struct VerifyEmail {
    pub flow: String,
}

#[post("email/verify")]
pub async fn verify_email(
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    email: web::Json<VerifyEmail>,
) -> Result<HttpResponse, ApiError> {
    let flow = DBFlow::get(&email.flow, &redis).await?;

    if let Some(DBFlow::ConfirmEmail {
        user_id,
        confirm_email,
    }) = flow
    {
        let user =
            crate::database::models::DBUser::get_id(user_id, &**pool, &redis)
                .await?
                .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

        if user.email != Some(confirm_email) {
            return Err(ApiError::InvalidInput(
                "E-mail does not match verify email. Try re-requesting the verification link."
                    .to_string(),
            ));
        }

        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            UPDATE users
            SET email_verified = TRUE
            WHERE (id = $1)
            ",
            user.id as crate::database::models::ids::DBUserId,
        )
        .execute(&mut *transaction)
        .await?;

        DBFlow::remove(&email.flow, &redis).await?;
        transaction.commit().await?;
        crate::database::models::DBUser::clear_caches(
            &[(user.id, None)],
            &redis,
        )
        .await?;

        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::InvalidInput(
            "Flow does not exist. Try re-requesting the verification link."
                .to_string(),
        ))
    }
}

#[post("email/subscribe")]
pub async fn subscribe_newsletter(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_AUTH_WRITE,
    )
    .await?
    .1;

    if let Some(email) = user.email {
        sign_up_sendy(&email).await?;

        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::InvalidInput(
            "User does not have an email.".to_string(),
        ))
    }
}

#[get("email/subscribe")]
pub async fn get_newsletter_subscription_status(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_READ,
    )
    .await?
    .1;

    if let Some(email) = user.email {
        let is_subscribed = check_sendy_subscription(&email).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "subscribed": is_subscribed
        })))
    } else {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "subscribed": false
        })))
    }
}

fn send_email_verify(
    email: String,
    flow: String,
    opener: &str,
) -> Result<(), crate::auth::email::MailError> {
    send_email(
        email,
        "Verify your email",
        opener,
        "Please visit the following link below to verify your email. If the button does not work, you can copy the link and paste it into your browser. This link expires in 24 hours.",
        Some((
            "Verify email",
            &format!(
                "{}/{}?flow={}",
                dotenvy::var("SITE_URL")?,
                dotenvy::var("SITE_VERIFY_EMAIL_PATH")?,
                flow
            ),
        )),
    )
}
