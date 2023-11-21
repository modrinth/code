use std::io::{Read, SeekFrom};

use crate::{
    prelude::{Credentials, DirectoryInfo},
    util::io::{self, IOError},
    {state::ProfilePathId, State},
};
use futures::TryFutureExt;
use serde::Serialize;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

#[derive(Serialize, Debug)]
pub struct Logs {
    pub filename: String,
    pub output: Option<CensoredString>,
}

#[derive(Serialize, Debug)]
pub struct LatestLogCursor {
    pub cursor: u64,
    pub output: CensoredString,
    pub new_file: bool,
}

#[derive(Serialize, Debug)] // Not deserialize
#[serde(transparent)]
pub struct CensoredString(String);
impl CensoredString {
    pub fn censor(mut s: String, credentials_set: &Vec<Credentials>) -> Self {
        let username = whoami::username();
        s = s
            .replace(&format!("/{}/", username), "/{COMPUTER_USERNAME}/")
            .replace(&format!("\\{}\\", username), "\\{COMPUTER_USERNAME}\\");
        for credentials in credentials_set {
            s = s
                .replace(&credentials.access_token, "{MINECRAFT_ACCESS_TOKEN}")
                .replace(&credentials.username, "{MINECRAFT_USERNAME}")
                .replace(
                    &credentials.id.as_simple().to_string(),
                    "{MINECRAFT_UUID}",
                )
                .replace(
                    &credentials.id.as_hyphenated().to_string(),
                    "{MINECRAFT_UUID}",
                );
        }

        Self(s)
    }
}

impl Logs {
    async fn build(
        profile_subpath: &ProfilePathId,
        filename: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            output: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(get_output_by_filename(profile_subpath, &filename).await?)
            },
            filename,
        })
    }
}

#[tracing::instrument]
pub async fn get_logs(
    profile_path: ProfilePathId,
    clear_contents: Option<bool>,
) -> crate::Result<Vec<Logs>> {
    let profile_path =
        if let Some(p) = crate::profile::get(&profile_path, None).await? {
            p.profile_id()
        } else {
            return Err(crate::ErrorKind::UnmanagedProfileError(
                profile_path.to_string(),
            )
            .into());
        };

    let logs_folder = DirectoryInfo::profile_logs_dir(&profile_path).await?;
    let mut logs = Vec::new();
    if logs_folder.exists() {
        for entry in std::fs::read_dir(&logs_folder)
            .map_err(|e| IOError::with_path(e, &logs_folder))?
        {
            let entry: std::fs::DirEntry =
                entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();

                logs.push(
                    Logs::build(&profile_path, file_name, clear_contents).await,
                );
            }
        }
    }

    let mut logs = logs.into_iter().collect::<crate::Result<Vec<Logs>>>()?;
    logs.sort_by_key(|x| x.filename.clone());
    Ok(logs)
}

#[tracing::instrument]
pub async fn get_logs_by_filename(
    profile_path: ProfilePathId,
    filename: String,
) -> crate::Result<Logs> {
    let profile_path =
        if let Some(p) = crate::profile::get(&profile_path, None).await? {
            p.profile_id()
        } else {
            return Err(crate::ErrorKind::UnmanagedProfileError(
                profile_path.to_string(),
            )
            .into());
        };
    Ok(Logs {
        output: Some(get_output_by_filename(&profile_path, &filename).await?),
        filename,
    })
}

#[tracing::instrument]
pub async fn get_output_by_filename(
    profile_subpath: &ProfilePathId,
    file_name: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;
    let logs_folder = DirectoryInfo::profile_logs_dir(profile_subpath).await?;
    let path = logs_folder.join(file_name);

    let credentials: Vec<Credentials> =
        state.users.read().await.clone().0.into_values().collect();

    // Load .gz file into String
    if let Some(ext) = path.extension() {
        if ext == "gz" {
            let file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut contents = [0; 1024];
            let mut result = String::new();
            let mut gz =
                flate2::read::GzDecoder::new(std::io::BufReader::new(file));

            while gz
                .read(&mut contents)
                .map_err(|e| IOError::with_path(e, &path))?
                > 0
            {
                result.push_str(&String::from_utf8_lossy(&contents));
                contents = [0; 1024];
            }
            return Ok(CensoredString::censor(result, &credentials));
        } else if ext == "log" {
            let mut result = String::new();
            let mut contents = [0; 1024];
            let mut file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            // iteratively read the file to a String
            while file
                .read(&mut contents)
                .map_err(|e| IOError::with_path(e, &path))?
                > 0
            {
                result.push_str(&String::from_utf8_lossy(&contents));
                contents = [0; 1024];
            }
            let result = CensoredString::censor(result, &credentials);
            return Ok(result);
        }
    }
    Err(crate::ErrorKind::OtherError(format!(
        "File extension not supported: {}",
        path.display()
    ))
    .into())
}

#[tracing::instrument]
pub async fn delete_logs(profile_path: ProfilePathId) -> crate::Result<()> {
    let profile_path =
        if let Some(p) = crate::profile::get(&profile_path, None).await? {
            p.profile_id()
        } else {
            return Err(crate::ErrorKind::UnmanagedProfileError(
                profile_path.to_string(),
            )
            .into());
        };

    let logs_folder = DirectoryInfo::profile_logs_dir(&profile_path).await?;
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
pub async fn delete_logs_by_filename(
    profile_path: ProfilePathId,
    filename: &str,
) -> crate::Result<()> {
    let profile_path =
        if let Some(p) = crate::profile::get(&profile_path, None).await? {
            p.profile_id()
        } else {
            return Err(crate::ErrorKind::UnmanagedProfileError(
                profile_path.to_string(),
            )
            .into());
        };

    let logs_folder = DirectoryInfo::profile_logs_dir(&profile_path).await?;
    let path = logs_folder.join(filename);
    io::remove_dir_all(&path).await?;
    Ok(())
}

#[tracing::instrument]
pub async fn get_latest_log_cursor(
    profile_path: ProfilePathId,
    cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    get_generic_live_log_cursor(profile_path, "latest.log", cursor).await
}

#[tracing::instrument]
pub async fn get_std_log_cursor(
    profile_path: ProfilePathId,
    cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    get_generic_live_log_cursor(profile_path, "latest_stdout.log", cursor).await
}

#[tracing::instrument]
pub async fn get_generic_live_log_cursor(
    profile_path: ProfilePathId,
    log_file_name: &str,
    mut cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    let profile_path =
        if let Some(p) = crate::profile::get(&profile_path, None).await? {
            p.profile_id()
        } else {
            return Err(crate::ErrorKind::UnmanagedProfileError(
                profile_path.to_string(),
            )
            .into());
        };

    let state = State::get().await?;
    let logs_folder = DirectoryInfo::profile_logs_dir(&profile_path).await?;
    let path = logs_folder.join(log_file_name);
    if !path.exists() {
        // Allow silent failure if latest.log doesn't exist (as the instance may have been launched, but not yet created the file)
        return Ok(LatestLogCursor {
            cursor: 0,
            new_file: false,
            output: CensoredString("".to_string()),
        });
    }

    let mut file = File::open(&path)
        .await
        .map_err(|e| IOError::with_path(e, &path))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| IOError::with_path(e, &path))?;

    let mut new_file = false;
    if cursor > metadata.len() {
        // Cursor is greater than file length, reset cursor to 0
        // Likely cause is that the file was rotated while the log was being read
        cursor = 0;
        new_file = true;
    }

    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(cursor))
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Seek to cursor
    let bytes_read = file
        .read_to_end(&mut buffer)
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Read to end of file
    let output = String::from_utf8_lossy(&buffer).to_string(); // Convert to String
    let cursor = cursor + bytes_read as u64; // Update cursor

    let credentials: Vec<Credentials> =
        state.users.read().await.clone().0.into_values().collect();
    let output = CensoredString::censor(output, &credentials);
    Ok(LatestLogCursor {
        cursor,
        new_file,
        output,
    })
}
