use crate::State;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

#[derive(Serialize, Deserialize, Debug)]
pub struct Logs {
    pub datetime_string: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}
impl Logs {
    async fn build(
        profile_uuid: uuid::Uuid,
        datetime_string: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            stdout: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(
                    get_stdout_by_datetime(profile_uuid, &datetime_string)
                        .await?,
                )
            },
            stderr: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(
                    get_stderr_by_datetime(profile_uuid, &datetime_string)
                        .await?,
                )
            },
            datetime_string,
        })
    }
}

#[tracing::instrument]
pub async fn get_logs(
    profile_uuid: uuid::Uuid,
    clear_contents: Option<bool>,
) -> crate::Result<Vec<Logs>> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    let mut logs = Vec::new();
    if logs_folder.exists() {
        for entry in std::fs::read_dir(logs_folder)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(datetime_string) = path.file_name() {
                    logs.push(
                        Logs::build(
                            profile_uuid,
                            datetime_string.to_string_lossy().to_string(),
                            clear_contents,
                        )
                        .await,
                    );
                }
            }
        }
    }

    let mut logs = logs.into_iter().collect::<crate::Result<Vec<Logs>>>()?;
    logs.sort_by_key(|x| x.datetime_string.clone());
    Ok(logs)
}

#[tracing::instrument]
pub async fn get_logs_by_datetime(
    profile_uuid: uuid::Uuid,
    datetime_string: String,
) -> crate::Result<Logs> {
    Ok(Logs {
        stdout: Some(
            get_stdout_by_datetime(profile_uuid, &datetime_string).await?,
        ),
        stderr: Some(
            get_stderr_by_datetime(profile_uuid, &datetime_string).await?,
        ),
        datetime_string,
    })
}

#[tracing::instrument]
pub async fn get_stdout_by_datetime(
    profile_uuid: uuid::Uuid,
    datetime_string: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    Ok(
        read_to_string(logs_folder.join(datetime_string).join("stdout.log"))
            .await?,
    )
}

#[tracing::instrument]
pub async fn get_stderr_by_datetime(
    profile_uuid: uuid::Uuid,
    datetime_string: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    Ok(
        read_to_string(logs_folder.join(datetime_string).join("stderr.log"))
            .await?,
    )
}

#[tracing::instrument]
pub async fn delete_logs(profile_uuid: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    for entry in std::fs::read_dir(logs_folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn delete_logs_by_datetime(
    profile_uuid: uuid::Uuid,
    datetime_string: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_uuid);
    std::fs::remove_dir_all(logs_folder.join(datetime_string))?;
    Ok(())
}
