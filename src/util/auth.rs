use crate::database;
use crate::database::models::project_item::QueryProject;
use crate::database::models::user_item;
use crate::database::models::version_item::QueryVersion;
use crate::database::{models, Project, Version};
use crate::models::users::{Badges, Role, User, UserId, UserPayoutData};
use crate::routes::ApiError;
use crate::Utc;
use actix_web::http::header::HeaderMap;
use actix_web::http::header::COOKIE;
use actix_web::web;
use reqwest::header::{HeaderValue, AUTHORIZATION};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use sqlx::PgPool;
use thiserror::Error;

use super::pat::get_user_from_pat;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("An unknown database error occurred")]
    Sqlx(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    Database(#[from] models::DatabaseError),
    #[error("Error while parsing JSON: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Error while communicating over the internet: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Error while decoding PAT: {0}")]
    Decoding(#[from] crate::models::ids::DecodingError),
    #[error("Invalid Authentication Credentials")]
    InvalidCredentials,
    #[error("Authentication method was not valid")]
    InvalidAuthMethod,
    #[error("GitHub Token from incorrect Client ID")]
    InvalidClientId,
}

// A user as stored in the Minos database
#[derive(Serialize, Deserialize, Debug)]
pub struct MinosUser {
    pub id: String,       // This is the unique generated Ory name
    pub username: String, // unique username
    pub email: String,
    pub name: Option<String>, // real name
    pub github_id: Option<u64>,
    pub discord_id: Option<u64>,
    pub google_id: Option<u128>,
    pub gitlab_id: Option<u64>,
    pub microsoft_id: Option<u64>,
    pub apple_id: Option<u64>,
}

// A payload marking a new user in Minos, with data to be inserted into Labrinth
#[serde_as]
#[derive(Deserialize, Debug)]
pub struct MinosNewUser {
    pub id: String,       // This is the unique generated Ory name
    pub username: String, // unique username
    pub email: String,

    pub name: Option<String>,            // real name
    pub default_picture: Option<String>, // uri of default avatar
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub github_id: Option<i64>, // we allow Github to be submitted to connect to an existing account
}

// Attempt to append a Minos user to an existing user, if one exists
// (combining the the legacy user with the Minos user)
pub async fn link_or_insert_new_user(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    minos_new_user: MinosNewUser,
) -> Result<(), AuthenticationError> {
    // If the user with this Github ID already exists, we can just merge the two accounts
    if let Some(github_id) = minos_new_user.github_id {
        if let Some(existing_user) =
            user_item::User::get_from_github_id(github_id as u64, &mut *transaction).await?
        {
            existing_user
                .merge_minos_user(&minos_new_user.id, &mut *transaction)
                .await?;
            return Ok(());
        }
    }
    // No user exists, so we need to create a new user
    insert_new_user(transaction, minos_new_user).await?;

    Ok(())
}

// Insert a new user into the database from a MinosUser
pub async fn insert_new_user(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    minos_new_user: MinosNewUser,
) -> Result<(), AuthenticationError> {
    let user_id = crate::database::models::generate_user_id(transaction).await?;

    database::models::User {
        id: user_id,
        kratos_id: Some(minos_new_user.id),
        username: minos_new_user.username,
        name: minos_new_user.name,
        email: Some(minos_new_user.email),
        avatar_url: minos_new_user.default_picture,
        bio: None,
        github_id: minos_new_user.github_id,
        created: Utc::now(),
        role: Role::Developer.to_string(),
        badges: Badges::default(),
        balance: Decimal::ZERO,
        payout_wallet: None,
        payout_wallet_type: None,
        payout_address: None,
    }
    .insert(transaction)
    .await?;

    Ok(())
}

// Gets MinosUser from Kratos ID
// This uses an administrative bearer token to access the Minos API
// Should NOT be directly accessible to users
pub async fn get_minos_user(kratos_id: &str) -> Result<MinosUser, AuthenticationError> {
    let ory_auth_bearer = dotenvy::var("ORY_AUTH_BEARER").unwrap();
    let req = reqwest::Client::new()
        .get(format!(
            "{}/admin/user/{kratos_id}",
            dotenvy::var("MINOS_URL").unwrap()
        ))
        .header(reqwest::header::USER_AGENT, "Labrinth")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {ory_auth_bearer}"),
        );
    let res = req.send().await?.error_for_status()?;
    let res = res.json().await?;
    Ok(res)
}

// pass the cookies to Minos to get the user.
pub async fn get_minos_user_from_cookies(cookies: &str) -> Result<MinosUser, AuthenticationError> {
    let req = reqwest::Client::new()
        .get(dotenvy::var("MINOS_URL").unwrap() + "/user")
        .header(reqwest::header::USER_AGENT, "Modrinth")
        .header(reqwest::header::COOKIE, cookies);
    let res = req.send().await?;

    let res = match res.status() {
        reqwest::StatusCode::OK => res,
        reqwest::StatusCode::UNAUTHORIZED => return Err(AuthenticationError::InvalidCredentials),
        _ => res.error_for_status()?,
    };
    Ok(res.json().await?)
}

pub async fn get_user_from_headers<'a, E>(
    headers: &HeaderMap,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let token: Option<&reqwest::header::HeaderValue> = headers.get(AUTHORIZATION);
    let cookies_unparsed: Option<&reqwest::header::HeaderValue> = headers.get(COOKIE);

    // Fetch DB user record and minos user from headers
    let (db_user, minos_user) = match (token, cookies_unparsed) {
        // If both, favour the bearer token first- redirect to cookie on failure
        (Some(token), Some(cookies)) => {
            match get_db_and_minos_user_from_bearer_token(token, executor).await {
                Ok((db, minos)) => (db, minos),
                Err(_) => get_db_and_minos_user_from_cookies(cookies, executor).await?,
            }
        }
        (Some(token), _) => get_db_and_minos_user_from_bearer_token(token, executor).await?,
        (_, Some(cookies)) => get_db_and_minos_user_from_cookies(cookies, executor).await?,
        _ => return Err(AuthenticationError::InvalidAuthMethod), // No credentials passed
    };

    let user = User {
        id: UserId::from(db_user.id),
        kratos_id: db_user.kratos_id,
        github_id: minos_user.github_id,
        discord_id: minos_user.discord_id,
        google_id: minos_user.google_id,
        microsoft_id: minos_user.microsoft_id,
        apple_id: minos_user.apple_id,
        gitlab_id: minos_user.gitlab_id,
        username: db_user.username,
        name: db_user.name,
        email: db_user.email,
        avatar_url: db_user.avatar_url,
        bio: db_user.bio,
        created: db_user.created,
        role: Role::from_string(&db_user.role),
        badges: db_user.badges,
        payout_data: Some(UserPayoutData {
            balance: db_user.balance,
            payout_wallet: db_user.payout_wallet,
            payout_wallet_type: db_user.payout_wallet_type,
            payout_address: db_user.payout_address,
        }),
    };
    Ok(user)
}

pub async fn get_user_from_headers_transaction(
    headers: &HeaderMap,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<User, AuthenticationError> {
    let token: Option<&reqwest::header::HeaderValue> = headers.get(AUTHORIZATION);
    let cookies_unparsed: Option<&reqwest::header::HeaderValue> = headers.get(COOKIE);

    // Fetch DB user record and minos user from headers
    let (db_user, minos_user) = match (token, cookies_unparsed) {
        // If both, favour the bearer token first- redirect to cookie on failure
        (Some(token), Some(cookies)) => {
            match get_db_and_minos_user_from_bearer_token(token, &mut *transaction).await {
                Ok((db, minos)) => (db, minos),
                Err(_) => get_db_and_minos_user_from_cookies(cookies, &mut *transaction).await?,
            }
        }
        (Some(token), _) => {
            get_db_and_minos_user_from_bearer_token(token, &mut *transaction).await?
        }
        (_, Some(cookies)) => {
            get_db_and_minos_user_from_cookies(cookies, &mut *transaction).await?
        }
        _ => return Err(AuthenticationError::InvalidAuthMethod), // No credentials passed
    };

    let user = User {
        id: UserId::from(db_user.id),
        kratos_id: db_user.kratos_id,
        github_id: minos_user.github_id,
        discord_id: minos_user.discord_id,
        google_id: minos_user.google_id,
        microsoft_id: minos_user.microsoft_id,
        apple_id: minos_user.apple_id,
        gitlab_id: minos_user.gitlab_id,
        username: db_user.username,
        name: db_user.name,
        email: db_user.email,
        avatar_url: db_user.avatar_url,
        bio: db_user.bio,
        created: db_user.created,
        role: Role::from_string(&db_user.role),
        badges: db_user.badges,
        payout_data: Some(UserPayoutData {
            balance: db_user.balance,
            payout_wallet: db_user.payout_wallet,
            payout_wallet_type: db_user.payout_wallet_type,
            payout_address: db_user.payout_address,
        }),
    };
    Ok(user)
}

pub async fn get_db_and_minos_user_from_bearer_token<'a, E>(
    token: &HeaderValue,
    executor: E,
) -> Result<(user_item::User, MinosUser), AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let db_user = get_user_record_from_bearer_token(
        token
            .to_str()
            .map_err(|_| AuthenticationError::InvalidCredentials)?,
        executor,
    )
    .await?
    .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
    let minos_user = get_minos_user(
        &db_user
            .kratos_id
            .clone()
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?,
    )
    .await?;
    Ok((db_user, minos_user))
}

pub async fn get_db_and_minos_user_from_cookies<'a, E>(
    cookies: &HeaderValue,
    executor: E,
) -> Result<(user_item::User, MinosUser), AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let minos_user = get_minos_user_from_cookies(
        cookies
            .to_str()
            .map_err(|_| AuthenticationError::InvalidCredentials)?,
    )
    .await?;
    let db_user = models::User::get_from_minos_kratos_id(minos_user.id.clone(), executor)
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;
    Ok((db_user, minos_user))
}

pub async fn get_user_record_from_bearer_token<'a, 'b, E>(
    token: &str,
    executor: E,
) -> Result<Option<user_item::User>, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    if token.starts_with("Bearer ") {
        let token: &str = token.trim_start_matches("Bearer ");

        // Tokens beginning with Ory are considered to be Kratos tokens (in reality, extracted cookies) and can be forwarded to Minos
        let possible_user = match token.split_once('_') {
            Some(("modrinth", _)) => get_user_from_pat(token, executor).await?,
            Some(("ory", _)) => get_user_from_minos_session_token(token, executor).await?,
            Some(("github", _)) | Some(("gho", _)) | Some(("ghp", _)) => {
                get_user_from_github_token(token, executor).await?
            }
            _ => return Err(AuthenticationError::InvalidAuthMethod),
        };
        Ok(possible_user)
    } else {
        Err(AuthenticationError::InvalidAuthMethod)
    }
}

pub async fn get_user_from_minos_session_token<'a, 'b, E>(
    token: &str,
    executor: E,
) -> Result<Option<user_item::User>, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let ory_auth_bearer = dotenvy::var("ORY_AUTH_BEARER").unwrap();
    let req = reqwest::Client::new()
        .get(dotenvy::var("MINOS_URL").unwrap() + "/admin/user/token?token=" + token)
        .header(reqwest::header::USER_AGENT, "Labrinth")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {ory_auth_bearer}"),
        );
    let res = req.send().await?.error_for_status()?;
    let minos_user: MinosUser = res.json().await?;
    let db_user = models::User::get_from_minos_kratos_id(minos_user.id.clone(), executor).await?;
    Ok(db_user)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubUser {
    pub id: u64,
}
// Get a database user from a GitHub PAT
pub async fn get_user_from_github_token<'a, E>(
    access_token: &str,
    executor: E,
) -> Result<Option<user_item::User>, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let response = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header(reqwest::header::USER_AGENT, "Modrinth")
        .header(AUTHORIZATION, format!("token {access_token}"))
        .send()
        .await?;

    if access_token.starts_with("gho_") {
        let client_id = response
            .headers()
            .get("x-oauth-client-id")
            .and_then(|x| x.to_str().ok());

        if client_id != Some(&*dotenvy::var("GITHUB_CLIENT_ID").unwrap()) {
            return Err(AuthenticationError::InvalidClientId);
        }
    }

    let github_user: GitHubUser = response.json().await?;

    Ok(user_item::User::get_from_github_id(github_user.id, executor).await?)
}

pub async fn check_is_moderator_from_headers<'a, 'b, E>(
    headers: &HeaderMap,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let user = get_user_from_headers(headers, executor).await?;

    if user.role.is_mod() {
        Ok(user)
    } else {
        Err(AuthenticationError::InvalidCredentials)
    }
}

pub async fn is_authorized(
    project_data: &Project,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<bool, ApiError> {
    let mut authorized = !project_data.status.is_hidden();

    if let Some(user) = &user_option {
        if !authorized {
            if user.role.is_mod() {
                authorized = true;
            } else {
                let user_id: models::ids::UserId = user.id.into();

                let project_exists = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
                    project_data.team_id as database::models::ids::TeamId,
                    user_id as database::models::ids::UserId,
                )
                .fetch_one(&***pool)
                .await?
                .exists;

                authorized = project_exists.unwrap_or(false);
            }
        }
    }

    Ok(authorized)
}

pub async fn filter_authorized_projects(
    projects: Vec<QueryProject>,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<Vec<crate::models::projects::Project>, ApiError> {
    let mut return_projects = Vec::new();
    let mut check_projects = Vec::new();

    for project in projects {
        if !project.inner.status.is_hidden()
            || user_option
                .as_ref()
                .map(|x| x.role.is_mod())
                .unwrap_or(false)
        {
            return_projects.push(project.into());
        } else if user_option.is_some() {
            check_projects.push(project);
        }
    }

    if !check_projects.is_empty() {
        if let Some(user) = user_option {
            let user_id: models::ids::UserId = user.id.into();

            use futures::TryStreamExt;

            sqlx::query!(
                "
                SELECT m.id id, m.team_id team_id FROM team_members tm
                INNER JOIN mods m ON m.team_id = tm.team_id
                WHERE tm.team_id = ANY($1) AND tm.user_id = $2
                ",
                &check_projects
                    .iter()
                    .map(|x| x.inner.team_id.0)
                    .collect::<Vec<_>>(),
                user_id as database::models::ids::UserId,
            )
            .fetch_many(&***pool)
            .try_for_each(|e| {
                if let Some(row) = e.right() {
                    check_projects.retain(|x| {
                        let bool = x.inner.id.0 == row.id && x.inner.team_id.0 == row.team_id;

                        if bool {
                            return_projects.push(x.clone().into());
                        }

                        !bool
                    });
                }

                futures::future::ready(Ok(()))
            })
            .await?;
        }
    }

    Ok(return_projects)
}

pub async fn is_authorized_version(
    version_data: &Version,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<bool, ApiError> {
    let mut authorized = !version_data.status.is_hidden();

    if let Some(user) = &user_option {
        if !authorized {
            if user.role.is_mod() {
                authorized = true;
            } else {
                let user_id: models::ids::UserId = user.id.into();

                let version_exists = sqlx::query!(
                    "SELECT EXISTS(SELECT 1 FROM mods m INNER JOIN team_members tm ON tm.team_id = m.team_id AND user_id = $2 WHERE m.id = $1)",
                    version_data.project_id as database::models::ids::ProjectId,
                    user_id as database::models::ids::UserId,
                )
                    .fetch_one(&***pool)
                    .await?
                    .exists;

                authorized = version_exists.unwrap_or(false);
            }
        }
    }

    Ok(authorized)
}

pub async fn filter_authorized_versions(
    versions: Vec<QueryVersion>,
    user_option: &Option<User>,
    pool: &web::Data<PgPool>,
) -> Result<Vec<crate::models::projects::Version>, ApiError> {
    let mut return_versions = Vec::new();
    let mut check_versions = Vec::new();

    for version in versions {
        if !version.inner.status.is_hidden()
            || user_option
                .as_ref()
                .map(|x| x.role.is_mod())
                .unwrap_or(false)
        {
            return_versions.push(version.into());
        } else if user_option.is_some() {
            check_versions.push(version);
        }
    }

    if !check_versions.is_empty() {
        if let Some(user) = user_option {
            let user_id: models::ids::UserId = user.id.into();

            use futures::TryStreamExt;

            sqlx::query!(
                "
                SELECT m.id FROM mods m
                INNER JOIN team_members tm ON tm.team_id = m.team_id AND user_id = $2
                WHERE m.id = ANY($1)
                ",
                &check_versions
                    .iter()
                    .map(|x| x.inner.project_id.0)
                    .collect::<Vec<_>>(),
                user_id as database::models::ids::UserId,
            )
            .fetch_many(&***pool)
            .try_for_each(|e| {
                if let Some(row) = e.right() {
                    check_versions.retain(|x| {
                        let bool = x.inner.project_id.0 == row.id;

                        if bool {
                            return_versions.push(x.clone().into());
                        }

                        !bool
                    });
                }

                futures::future::ready(Ok(()))
            })
            .await?;
        }
    }

    Ok(return_versions)
}
