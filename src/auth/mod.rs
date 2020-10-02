use crate::database::models;
use crate::models::users::{Role, User, UserId};
use actix_web::http::HeaderMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("An unknown database error occurred")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
    #[error("Error while parsing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while communicating to GitHub OAuth2: {0}")]
    GithubError(#[from] reqwest::Error),
    #[error("Invalid Authentication Credentials")]
    InvalidCredentialsError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub name: String,
    pub email: Option<String>,
    pub bio: String,
}

pub async fn get_github_user_from_token(
    access_token: &str,
) -> Result<GitHubUser, AuthenticationError> {
    Ok(reqwest::Client::new()
        .get("https://api.github.com/user")
        .header(reqwest::header::USER_AGENT, "Modrinth")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", access_token),
        )
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_user_from_token<'a, 'b, E>(
    access_token: &str,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let github_user = get_github_user_from_token(access_token).await?;

    let res = models::User::get_from_github_id(github_user.id, executor).await?;

    match res {
        Some(result) => Ok(User {
            id: UserId::from(result.id),
            github_id: result.github_id.map(|i| i as u64),
            username: result.username,
            name: result.name,
            email: result.email,
            avatar_url: result.avatar_url,
            bio: result.bio,
            created: result.created,
            role: Role::from_string(&*result.role),
        }),
        None => Err(AuthenticationError::InvalidCredentialsError),
    }
}
pub async fn get_user_from_headers<'a, 'b, E>(
    headers: &HeaderMap,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let token = headers
        .get("Authorization")
        .ok_or(AuthenticationError::InvalidCredentialsError)?
        .to_str()
        .map_err(|_| AuthenticationError::InvalidCredentialsError)?;

    Ok(get_user_from_token(token, executor).await?)
}

pub async fn check_is_moderator_from_headers<'a, 'b, E>(
    headers: &HeaderMap,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let user = get_user_from_headers(headers, executor).await?;

    match user.role {
        Role::Moderator | Role::Admin => Ok(user),
        _ => Err(AuthenticationError::InvalidCredentialsError),
    }
}

pub async fn check_is_admin_from_headers<'a, 'b, E>(
    headers: &HeaderMap,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let user = get_user_from_headers(headers, executor).await?;

    match user.role {
        Role::Admin => Ok(user),
        _ => Err(AuthenticationError::InvalidCredentialsError),
    }
}
