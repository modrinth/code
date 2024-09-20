use crate::config::MODRINTH_API_URL;
use crate::state::{CacheBehaviour, CachedEntry};
use crate::util::fetch::{fetch_advanced, FetchSemaphore};
use chrono::{DateTime, Duration, TimeZone, Utc};
use dashmap::DashMap;
use futures::TryStreamExt;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthCredentials {
    pub session: String,
    pub expires: DateTime<Utc>,
    pub user_id: String,
    pub active: bool,
}

impl ModrinthCredentials {
    pub async fn get_and_refresh(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<Option<Self>> {
        let creds = Self::get_active(exec).await?;

        if let Some(mut creds) = creds {
            if creds.expires < Utc::now() {
                #[derive(Deserialize)]
                struct Session {
                    session: String,
                }

                let resp = fetch_advanced(
                    Method::POST,
                    &format!("{MODRINTH_API_URL}session/refresh"),
                    None,
                    None,
                    Some(("Authorization", &*creds.session)),
                    None,
                    semaphore,
                    exec,
                )
                .await
                .ok()
                .and_then(|resp| serde_json::from_slice::<Session>(&resp).ok());

                if let Some(value) = resp {
                    creds.session = value.session;
                    creds.expires = Utc::now() + Duration::weeks(2);
                    creds.upsert(exec).await?;

                    Ok(Some(creds))
                } else {
                    Self::remove(&creds.user_id, exec).await?;

                    Ok(None)
                }
            } else {
                Ok(Some(creds))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_active(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        let res = sqlx::query!(
            "
            SELECT
                id, active, session_id, expires
            FROM modrinth_users
            WHERE active = TRUE
            "
        )
        .fetch_optional(exec)
        .await?;

        Ok(res.map(|x| Self {
            session: x.session_id,
            expires: Utc
                .timestamp_opt(x.expires, 0)
                .single()
                .unwrap_or_else(Utc::now),
            user_id: x.id,
            active: x.active == 1,
        }))
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<DashMap<String, Self>> {
        let res = sqlx::query!(
            "
            SELECT
                id, active, session_id, expires
            FROM modrinth_users
            "
        )
        .fetch(exec)
        .try_fold(DashMap::new(), |acc, x| {
            acc.insert(
                x.id.clone(),
                Self {
                    session: x.session_id,
                    expires: Utc
                        .timestamp_opt(x.expires, 0)
                        .single()
                        .unwrap_or_else(Utc::now),
                    user_id: x.id,
                    active: x.active == 1,
                },
            );

            async move { Ok(acc) }
        })
        .await?;

        Ok(res)
    }

    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<()> {
        let expires = self.expires.timestamp();

        if self.active {
            sqlx::query!(
                "
                UPDATE modrinth_users
                SET active = FALSE
                "
            )
            .execute(exec)
            .await?;
        }

        sqlx::query!(
            "
            INSERT INTO modrinth_users (id, active, session_id, expires)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET
                active = $2,
                session_id = $3,
                expires = $4
            ",
            self.user_id,
            self.active,
            self.session,
            expires,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn remove(
        user_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        sqlx::query!(
            "
            DELETE FROM modrinth_users WHERE id = $1
            ",
            user_id,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub(crate) async fn refresh_all() -> crate::Result<()> {
        let state = crate::State::get().await?;
        let all = Self::get_all(&state.pool).await?;

        let user_ids = all.into_iter().map(|x| x.0).collect::<Vec<_>>();

        CachedEntry::get_user_many(
            &user_ids.iter().map(|x| &**x).collect::<Vec<_>>(),
            Some(CacheBehaviour::Bypass),
            &state.pool,
            &state.fetch_semaphore,
        )
        .await?;

        Ok(())
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ModrinthCredentialsResult {
    TwoFactorRequired { flow: String },
    Credentials(ModrinthCredentials),
}

async fn get_result_from_res(
    code_key: &str,
    response: HashMap<String, Value>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<ModrinthCredentialsResult> {
    if let Some(flow) = response.get("flow").and_then(|x| x.as_str()) {
        Ok(ModrinthCredentialsResult::TwoFactorRequired {
            flow: flow.to_string(),
        })
    } else if let Some(code) = response.get(code_key).and_then(|x| x.as_str()) {
        let info = fetch_info(code, semaphore, exec).await?;

        Ok(ModrinthCredentialsResult::Credentials(
            ModrinthCredentials {
                session: code.to_string(),
                expires: Utc::now() + Duration::weeks(2),
                user_id: info.id,
                active: true,
            },
        ))
    } else if let Some(error) =
        response.get("description").and_then(|x| x.as_str())
    {
        Err(crate::ErrorKind::OtherError(format!(
            "Failed to login with error {error}"
        ))
        .as_error())
    } else {
        Err(crate::ErrorKind::OtherError(String::from(
            "Flow/code/error not found in response!",
        ))
        .as_error())
    }
}

async fn get_creds_from_res(
    response: HashMap<String, Value>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<ModrinthCredentials> {
    if let Some(code) = response.get("session").and_then(|x| x.as_str()) {
        let info = fetch_info(code, semaphore, exec).await?;

        Ok(ModrinthCredentials {
            session: code.to_string(),
            expires: Utc::now() + Duration::weeks(2),
            user_id: info.id,
            active: true,
        })
    } else if let Some(error) =
        response.get("description").and_then(|x| x.as_str())
    {
        Err(crate::ErrorKind::OtherError(format!(
            "Failed to login with error {error}"
        ))
        .as_error())
    } else {
        Err(crate::ErrorKind::OtherError(String::from(
            "Flow/code/error not found in response!",
        ))
        .as_error())
    }
}

pub fn get_login_url(provider: &str) -> String {
    format!(
        "{MODRINTH_API_URL}auth/init?url={}&provider={provider}",
        urlencoding::encode("https://launcher-files.modrinth.com/detect.txt")
    )
}

pub async fn finish_login_flow(
    response: HashMap<String, Value>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<ModrinthCredentialsResult> {
    get_result_from_res("code", response, semaphore, exec).await
}

pub async fn login_password(
    username: &str,
    password: &str,
    challenge: &str,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<ModrinthCredentialsResult> {
    let resp = fetch_advanced(
        Method::POST,
        &format!("{MODRINTH_API_URL}auth/login"),
        None,
        Some(serde_json::json!({
            "username": username,
            "password": password,
            "challenge": challenge,
        })),
        None,
        None,
        semaphore,
        exec,
    )
    .await?;
    let value = serde_json::from_slice::<HashMap<String, Value>>(&resp)?;

    get_result_from_res("session", value, semaphore, exec).await
}

pub async fn login_2fa(
    code: &str,
    flow: &str,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<ModrinthCredentials> {
    let resp = fetch_advanced(
        Method::POST,
        &format!("{MODRINTH_API_URL}auth/login/2fa"),
        None,
        Some(serde_json::json!({
            "code": code,
            "flow": flow,
        })),
        None,
        None,
        semaphore,
        exec,
    )
    .await?;

    let response = serde_json::from_slice::<HashMap<String, Value>>(&resp)?;

    get_creds_from_res(response, semaphore, exec).await
}

pub async fn create_account(
    username: &str,
    email: &str,
    password: &str,
    challenge: &str,
    sign_up_newsletter: bool,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<ModrinthCredentials> {
    let resp = fetch_advanced(
        Method::POST,
        &format!("{MODRINTH_API_URL}auth/create"),
        None,
        Some(serde_json::json!({
            "username": username,
            "email": email,
            "password": password,
            "challenge": challenge,
            "sign_up_newsletter": sign_up_newsletter,
        })),
        None,
        None,
        semaphore,
        exec,
    )
    .await?;
    let response = serde_json::from_slice::<HashMap<String, Value>>(&resp)?;

    get_creds_from_res(response, semaphore, exec).await
}

async fn fetch_info(
    token: &str,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<crate::state::cache::User> {
    let result = fetch_advanced(
        Method::GET,
        &format!("{MODRINTH_API_URL}user"),
        None,
        None,
        Some(("Authorization", token)),
        None,
        semaphore,
        exec,
    )
    .await?;
    let value = serde_json::from_slice(&result)?;

    Ok(value)
}
