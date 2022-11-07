use crate::database;
use crate::database::models;
use crate::database::models::project_item::QueryProject;
use crate::models::users::{Role, User, UserId, UserPayoutData};
use crate::routes::ApiError;
use actix_web::http::header::HeaderMap;
use actix_web::web;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("An unknown database error occurred")]
    Sqlx(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    Database(#[from] models::DatabaseError),
    #[error("Error while parsing JSON: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Error while communicating to GitHub OAuth2: {0}")]
    Github(#[from] reqwest::Error),
    #[error("Invalid Authentication Credentials")]
    InvalidCredentials,
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

    let res =
        models::User::get_from_github_id(github_user.id, executor).await?;

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
            role: Role::from_string(&result.role),
            badges: result.badges,
            payout_data: Some(UserPayoutData {
                balance: result.balance,
                payout_wallet: result.payout_wallet,
                payout_wallet_type: result.payout_wallet_type,
                payout_address: result.payout_address,
            }),
        }),
        None => Err(AuthenticationError::InvalidCredentials),
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
        .ok_or(AuthenticationError::InvalidCredentials)?
        .to_str()
        .map_err(|_| AuthenticationError::InvalidCredentials)?;

    get_user_from_token(token, executor).await
}

pub async fn check_is_moderator_from_headers<'a, 'b, E>(
    headers: &HeaderMap,
    executor: E,
) -> Result<User, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let user = get_user_from_headers(headers, executor).await?;

    if user.role.is_mod() {
        Ok(user)
    } else {
        Err(AuthenticationError::InvalidCredentials)
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
        _ => Err(AuthenticationError::InvalidCredentials),
    }
}

pub async fn is_authorized(
    project_data: &QueryProject,
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
                    project_data.inner.team_id as database::models::ids::TeamId,
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
