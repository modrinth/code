use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use tokio::fs::File;
use tokio_util::compat::FuturesAsyncWriteCompatExt;

use crate::util::io::{self, IOError};

const SCREENSHOTS_DIRECTORY: &str = "screenshots";

#[derive(Clone, Debug, Serialize)]
pub struct InstanceScreenshot {
    pub file_name: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub path: PathBuf,
}

pub async fn list_screenshots(
    instance_id: &str,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let screenshots_dir = screenshots_dir(instance_id).await?;
    let mut entries = match io::read_dir(&screenshots_dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };
    let mut screenshots = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| IOError::with_path(error, &screenshots_dir))?
    {
        let file_type = entry
            .file_type()
            .await
            .map_err(|error| IOError::with_path(error, entry.path()))?;
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();
        if !has_png_extension(&path) {
            continue;
        }

        let Some(file_name) = entry.file_name().to_str().map(str::to_owned)
        else {
            continue;
        };
        let metadata = entry
            .metadata()
            .await
            .map_err(|error| IOError::with_path(error, &path))?;
        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(DateTime::<Utc>::from)
            .map_err(|error| IOError::with_path(error, &path))?;

        screenshots.push(InstanceScreenshot {
            file_name,
            created_at,
            path,
        });
    }

    screenshots.sort_by_key(|screenshot| std::cmp::Reverse(screenshot.created_at));
    Ok(screenshots)
}

pub async fn delete_screenshot(
    instance_id: &str,
    file_name: &str,
) -> crate::Result<()> {
    let path = get_screenshot_path(instance_id, file_name).await?;
    io::remove_file(path).await?;
    Ok(())
}

pub async fn export_screenshots(
    instance_id: &str,
    file_names: &[String],
    export_path: PathBuf,
) -> crate::Result<()> {
    if file_names.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "At least one screenshot must be selected".to_string(),
        )
        .into());
    }

    let mut screenshots = Vec::with_capacity(file_names.len());
    for file_name in file_names {
        screenshots.push((
            file_name,
            get_screenshot_path(instance_id, file_name).await?,
        ));
    }

    let mut file = File::create(&export_path)
        .await
        .map_err(|error| IOError::with_path(error, &export_path))?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    for (file_name, path) in screenshots {
        let mut stream = writer
            .write_entry_stream(
                ZipEntryBuilder::new(
                    file_name.as_str().into(),
                    Compression::Stored,
                )
                .build(),
            )
            .await?
            .compat_write();
        let mut source = File::open(&path)
            .await
            .map_err(|error| IOError::with_path(error, &path))?;
        tokio::io::copy(&mut source, &mut stream)
            .await
            .map_err(IOError::from)?;
        stream.into_inner().close().await?;
    }

    writer.close().await?;
    Ok(())
}

pub async fn get_screenshot_path(
    instance_id: &str,
    file_name: &str,
) -> crate::Result<PathBuf> {
    validate_file_name(file_name)?;

    let screenshots_dir = screenshots_dir(instance_id).await?;
    let canonical_dir = tokio::fs::canonicalize(&screenshots_dir)
        .await
        .map_err(|error| IOError::with_path(error, &screenshots_dir))?;
    let requested_path = screenshots_dir.join(file_name);
    let metadata = tokio::fs::symlink_metadata(&requested_path)
        .await
        .map_err(|error| IOError::with_path(error, &requested_path))?;

    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(crate::ErrorKind::InputError(
            "Screenshot must be a regular file".to_string(),
        )
        .into());
    }

    let canonical_path = tokio::fs::canonicalize(&requested_path)
        .await
        .map_err(|error| IOError::with_path(error, &requested_path))?;
    if !canonical_path.starts_with(&canonical_dir) {
        return Err(crate::ErrorKind::InputError(
            "Screenshot path is outside the instance screenshots directory"
                .to_string(),
        )
        .into());
    }

    Ok(canonical_path)
}

async fn screenshots_dir(instance_id: &str) -> crate::Result<PathBuf> {
    let path = super::get_full_path(instance_id)
        .await?
        .join(SCREENSHOTS_DIRECTORY);

    match tokio::fs::symlink_metadata(&path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(crate::ErrorKind::InputError(
                "Instance screenshots directory cannot be a symbolic link"
                    .to_string(),
            )
            .into())
        }
        Ok(_) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(path),
        Err(error) => Err(IOError::with_path(error, &path).into()),
    }
}

fn validate_file_name(file_name: &str) -> crate::Result<()> {
    let path = Path::new(file_name);
    let mut components = path.components();
    let is_single_file = matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none();

    if !is_single_file || !has_png_extension(path) {
        return Err(crate::ErrorKind::InputError(
            "Invalid screenshot file name".to_string(),
        )
        .into());
    }

    Ok(())
}

fn has_png_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
}
