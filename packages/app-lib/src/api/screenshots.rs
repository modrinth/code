use crate::profile::get_full_path;
use crate::util::io::{metadata, read_dir};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::fs::{canonicalize, read, remove_file};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Screenshot {
    pub path: String,
    pub creation_date: DateTime<Utc>,
}

pub async fn get_all_profile_screenshots(
    profile_path: &str,
) -> crate::Result<Vec<Screenshot>> {
    let full = get_full_path(profile_path).await?;
    get_all_screenshots_in_profile(&full).await
}

pub async fn delete_profile_screenshot(
    profile_path: &str,
    screenshot: &Screenshot,
) -> crate::Result<bool> {
    let full = get_full_path(profile_path).await?;
    delete_screenshot_in_profile(&full, screenshot).await
}

async fn delete_screenshot_in_profile(
    profile_dir: &Path,
    screenshot: &Screenshot,
) -> crate::Result<bool> {
    if let Some(path) =
        get_valid_screenshot_path(profile_dir, screenshot).await?
    {
        remove_file(path).await?;
        Ok(true)
    } else {
        Ok(false)
    }
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

        let abs_path: PathBuf = canonicalize(&path).await?;
        let full_path = abs_path.to_string_lossy().into_owned();

        let meta = entry.metadata().await?;
        let created_time = meta.created().unwrap_or(meta.modified()?);
        let creation_date = DateTime::<Utc>::from(created_time);

        screenshots.push(Screenshot {
            path: full_path,
            creation_date,
        });
    }

    screenshots.sort_by_key(|s| s.creation_date);
    Ok(screenshots)
}

pub async fn get_screenshot_data(
    profile_dir: &Path,
    screenshot: &Screenshot,
) -> crate::Result<Option<String>> {
    if let Some(valid_path) =
        get_valid_screenshot_path(profile_dir, screenshot).await?
    {
        let bytes = read(&valid_path).await?;
        let encoded = STANDARD.encode(&bytes);
        Ok(Some(encoded))
    } else {
        Ok(None)
    }
}

pub async fn get_valid_screenshot_path(
    profile_dir: &Path,
    screenshot: &Screenshot,
) -> crate::Result<Option<PathBuf>> {
    let screenshots_dir = profile_dir.join("screenshots");
    if metadata(&screenshots_dir).await.is_err() {
        return Ok(None);
    }

    let canonical_dir = match canonicalize(&screenshots_dir).await {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let requested = PathBuf::from(&screenshot.path);
    let canonical_req = match canonicalize(&requested).await {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    if canonical_req.starts_with(&canonical_dir) {
        Ok(Some(canonical_req))
    } else {
        Ok(None)
    }
}
