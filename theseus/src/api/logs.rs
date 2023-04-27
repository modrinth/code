//! Authentication flow interface
use crate::{launcher::auth as inner, State};
use futures::prelude::*;
use serde::{Serialize, Deserialize};
use tokio::{sync::oneshot, fs::{File, read_to_string}};

#[derive(Serialize, Deserialize)]
pub struct Logs {
    pub stdout: String,
    pub stderr: String,
}

pub async fn get_logs(profile_uuid: uuid::Uuid) -> crate::Result<Vec<Logs>> {
    
}

#[tracing::instrument]
pub async fn get_logs_by_datetime(profile_uuid : uuid::Uuid, datetime_string : String) -> crate::Result<Logs> {
    Ok(Logs {
        stdout: get_stdout_by_datetime(profile_uuid, datetime_string.clone()).await?,
        stderr: get_stderr_by_datetime(profile_uuid, datetime_string).await?,
    })
}

#[tracing::instrument]
pub async fn get_stdout_by_datetime(profile_uuid : uuid::Uuid, datetime_string : String) -> crate::Result<String> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    Ok(read_to_string(logs_folder.join(datetime_string).join("stdout.log")).await?)
}

#[tracing::instrument]
pub async fn get_stderr_by_datetime(profile_uuid : uuid::Uuid, datetime_string : String) -> crate::Result<String> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    Ok(read_to_string(logs_folder.join(datetime_string).join("stderr.log")).await?)
}

/// Get a specific user by user ID
/// Prefer to use 'refresh' instead of this function
#[tracing::instrument]
pub async fn get_user(user: uuid::Uuid) -> crate::Result<Credentials> {
    let state = State::get().await?;
    let users = state.users.read().await;
    let user = users.get(user)?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to get nonexistent user with ID {user}"
        ))
        .as_error()
    })?;
    Ok(user)
}
