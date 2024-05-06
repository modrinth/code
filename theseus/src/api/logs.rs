use std::io::{Read, SeekFrom};
use std::time::SystemTime;

use futures::TryFutureExt;
use once_cell::unsync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{
    prelude::{Credentials, DirectoryInfo},
    util::io::{self, IOError},
    {state::ProfilePathId, State},
};

#[derive(Serialize, Debug)]
pub struct Logs {
    pub log_type: LogType,
    pub filename: String,
    pub age: u64,
    pub output: Option<CensoredString>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum LogType {
    InfoLog,
    CrashReport,
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
        log_type: LogType,
        age: SystemTime,
        profile_subpath: &ProfilePathId,
        filename: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            log_type,
            age: age
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            output: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(
                    get_output_by_filename(
                        profile_subpath,
                        log_type,
                        &filename,
                    )
                    .await?,
                )
            },
            filename,
        })
    }
}

#[tracing::instrument]
pub async fn get_logs_from_type(
    profile_path: &ProfilePathId,
    log_type: LogType,
    clear_contents: Option<bool>,
    logs: &mut Vec<crate::Result<Logs>>,
) -> crate::Result<()> {
    let now = Lazy::<SystemTime, _>::new(|| SystemTime::now());

    let logs_folder = match log_type {
        LogType::InfoLog => {
            DirectoryInfo::profile_logs_dir(profile_path).await?
        }
        LogType::CrashReport => {
            DirectoryInfo::crash_reports_dir(profile_path).await?
        }
    };
    if logs_folder.exists() {
        for entry in std::fs::read_dir(&logs_folder)
            .map_err(|e| IOError::with_path(e, &logs_folder))?
        {
            let entry: std::fs::DirEntry =
                entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
            let age = entry.metadata()?.created().unwrap_or_else(|_| *now);
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                tracing::info!("Pushing log file {file_name}");
                logs.push(
                    Logs::build(
                        log_type,
                        age,
                        &profile_path,
                        file_name,
                        clear_contents,
                    )
                    .await,
                );
            }
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn get_logs(
    profile_path_id: ProfilePathId,
    clear_contents: Option<bool>,
) -> crate::Result<Vec<Logs>> {
    let profile_path = profile_path_id.profile_path().await?;

    let mut logs = Vec::new();
    get_logs_from_type(
        &profile_path,
        LogType::InfoLog,
        clear_contents,
        &mut logs,
    )
    .await?;
    get_logs_from_type(
        &profile_path,
        LogType::CrashReport,
        clear_contents,
        &mut logs,
    )
    .await?;

    let mut logs = logs.into_iter().collect::<crate::Result<Vec<Logs>>>()?;
    tracing::info!("Log locations: {:#?}", logs.iter().map(|x| x.filename.clone()).collect::<Vec<String>>());
    logs.sort_by(|a, b| a.age.cmp(&b.age).then(a.filename.cmp(&b.filename)));
    Ok(logs)
}

#[tracing::instrument]
pub async fn get_logs_by_filename(
    profile_path_id: ProfilePathId,
    log_type: LogType,
    filename: String,
) -> crate::Result<Logs> {
    let profile_path = profile_path_id.profile_path().await?;

    let path = match log_type {
        LogType::InfoLog => DirectoryInfo::profile_logs_dir(&profile_path).await,
        LogType::CrashReport => DirectoryInfo::crash_reports_dir(&profile_path).await,
    }?
    .join(&filename);

    let metadata = std::fs::metadata(&path)?;
    let age = metadata.created().unwrap_or_else(|_| SystemTime::now());

    Logs::build(log_type, age, &profile_path, filename, Some(true)).await
}

#[tracing::instrument]
pub async fn get_output_by_filename(
    profile_subpath: &ProfilePathId,
    log_type: LogType,
    file_name: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;

    let logs_folder = match log_type {
        LogType::InfoLog => {
            DirectoryInfo::profile_logs_dir(profile_subpath).await?
        }
        LogType::CrashReport => {
            DirectoryInfo::crash_reports_dir(profile_subpath).await?
        }
    };

    let path = logs_folder.join(file_name);

    let credentials: Vec<Credentials> = state
        .users
        .read()
        .await
        .users
        .clone()
        .into_values()
        .collect();

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
        } else if ext == "log" || ext == "txt" {
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
pub async fn delete_logs(profile_path_id: ProfilePathId) -> crate::Result<()> {
    let profile_path = profile_path_id.profile_path().await?;

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
    profile_path_id: ProfilePathId,
    log_type: LogType,
    filename: &str,
) -> crate::Result<()> {
    let profile_path = profile_path_id.profile_path().await?;

    let logs_folder = match log_type {
        LogType::InfoLog => DirectoryInfo::profile_logs_dir(&profile_path).await,
        LogType::CrashReport => DirectoryInfo::crash_reports_dir(&profile_path).await,
    }?;

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
pub async fn get_generic_live_log_cursor(
    profile_path_id: ProfilePathId,
    log_file_name: &str,
    mut cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    let profile_path = profile_path_id.profile_path().await?;

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

    let credentials: Vec<Credentials> = state
        .users
        .read()
        .await
        .users
        .clone()
        .into_values()
        .collect();
    let output = CensoredString::censor(output, &credentials);
    Ok(LatestLogCursor {
        cursor,
        new_file,
        output,
    })
}
