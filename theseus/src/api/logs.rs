use crate::{
    util::io::{self, IOError},
    {state::ProfilePathId, State},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Logs {
    pub datetime_string: String,
    pub output: Option<String>,
}
impl Logs {
    async fn build(
        profile_subpath: &ProfilePathId,
        datetime_string: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            output: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(
                    get_output_by_datetime(profile_subpath, &datetime_string)
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
    let profile_path = if let Some(p) =
        crate::profile::get_by_uuid(profile_uuid, None).await?
    {
        p.profile_id()
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(
            profile_uuid.to_string(),
        )
        .into());
    };

    let logs_folder = state.directories.profile_logs_dir(&profile_path).await?;
    let mut logs = Vec::new();
    if logs_folder.exists() {
        for entry in std::fs::read_dir(&logs_folder)
            .map_err(|e| IOError::with_path(e, &logs_folder))?
        {
            let entry =
                entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(datetime_string) = path.file_name() {
                    logs.push(
                        Logs::build(
                            &profile_path,
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
    let profile_path = if let Some(p) =
        crate::profile::get_by_uuid(profile_uuid, None).await?
    {
        p.profile_id()
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(
            profile_uuid.to_string(),
        )
        .into());
    };
    Ok(Logs {
        output: Some(
            get_output_by_datetime(&profile_path, &datetime_string).await?,
        ),
        datetime_string,
    })
}

#[tracing::instrument]
pub async fn get_output_by_datetime(
    profile_subpath: &ProfilePathId,
    datetime_string: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let logs_folder =
        state.directories.profile_logs_dir(profile_subpath).await?;
    let path = logs_folder.join(datetime_string).join("stdout.log");
    Ok(io::read_to_string(&path).await?)
}

#[tracing::instrument]
pub async fn delete_logs(profile_uuid: uuid::Uuid) -> crate::Result<()> {
    let profile_path = if let Some(p) =
        crate::profile::get_by_uuid(profile_uuid, None).await?
    {
        p.profile_id()
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(
            profile_uuid.to_string(),
        )
        .into());
    };

    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(&profile_path).await?;
    for entry in std::fs::read_dir(&logs_folder)
        .map_err(|e| IOError::with_path(e, &logs_folder))?
    {
        let entry = entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
        let path = entry.path();
        if path.is_dir() {
            io::remove_dir_all(&path).await?;
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn delete_logs_by_datetime(
    profile_uuid: uuid::Uuid,
    datetime_string: &str,
) -> crate::Result<()> {
    let profile_path = if let Some(p) =
        crate::profile::get_by_uuid(profile_uuid, None).await?
    {
        p.profile_id()
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(
            profile_uuid.to_string(),
        )
        .into());
    };

    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(&profile_path).await?;
    let path = logs_folder.join(datetime_string);
    io::remove_dir_all(&path).await?;
    Ok(())
}
