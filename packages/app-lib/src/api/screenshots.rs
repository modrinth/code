use std::ffi::OsStr;
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::canonicalize;
use crate::profile::get_full_path;
use crate::util::io::{metadata, read_dir};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Screenshot {
    pub path: String,
    pub creation_date: DateTime<Utc>
}

pub async fn get_all_profile_screenshots(
    profile_path: &str,
) -> crate::Result<Vec<Screenshot>> {
    get_all_screenshots_in_profile(&get_full_path(profile_path).await?)
        .await
}

async fn get_all_screenshots_in_profile(
    profile_dir: &Path,
) -> crate::Result<Vec<Screenshot>> {
    let screenshots_dir = profile_dir.join("screenshots");
    if metadata(&screenshots_dir).await.is_err() {
        return Ok(Vec::new());
    }

    let mut dir = read_dir(&screenshots_dir).await?;
    let mut screenshots = Vec::new();

    while let Some(entry) = dir.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let path = entry.path();
        if path
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ext.eq_ignore_ascii_case("png"))
            != Some(true)
        {
            continue;
        }

        let abs_path: std::path::PathBuf = canonicalize(&path).await?;
        let full_path = abs_path.to_string_lossy().into_owned();

        let meta = entry.metadata().await?;
        let created_time = meta.created().unwrap_or(meta.modified()?);
        let creation_date = DateTime::<Utc>::from(created_time);

        screenshots.push(Screenshot { path: full_path, creation_date });
    }

    screenshots.sort_by_key(|s| s.creation_date);
    
    Ok(screenshots)
}