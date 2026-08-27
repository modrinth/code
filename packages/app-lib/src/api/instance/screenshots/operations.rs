use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use tokio::sync::OwnedMutexGuard;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::reconciliation::{
    list_source_screenshots, reconcile_source_screenshots,
    scan_source_screenshots,
};
use crate::State;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::{
    instance_rows::{self, InstanceScreenshotSource},
    screenshot_rows,
};
use crate::util::fetch::sha1_file_async;
use crate::util::io::{self, IOError};

const SCREENSHOTS_DIRECTORY: &str = "screenshots";
const SCREENSHOT_SCAN_CONCURRENCY: usize = 8;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ScreenshotKey {
    pub instance_id: String,
    pub file_name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct InstanceScreenshot {
    pub id: String,
    pub instance_id: String,
    pub instance_name: String,
    pub file_name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: i64,
    pub group_id: Option<String>,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotEditSaveMode {
    CreateCopy,
    ReplaceEdit,
}

pub async fn list_screenshots(
    instance_id: &str,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let state = State::get().await?;
    let source =
        instance_rows::get_instance_screenshot_source(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;

    list_source_screenshots(&state, source).await
}

pub async fn list_synced_screenshots() -> crate::Result<Vec<InstanceScreenshot>>
{
    let state = State::get().await?;
    let sources =
        instance_rows::list_synced_screenshot_sources(&state.pool).await?;
    list_source_screenshot_sets(&state, sources).await
}

pub async fn list_all_screenshots() -> crate::Result<Vec<InstanceScreenshot>> {
    let state = State::get().await?;
    let sources = instance_rows::list_screenshot_sources(&state.pool).await?;
    list_source_screenshot_sets(&state, sources).await
}

async fn list_source_screenshot_sets(
    state: &State,
    sources: Vec<InstanceScreenshotSource>,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let mut screenshots =
        stream::iter(sources.into_iter().map(|source| async move {
            list_source_screenshots(state, source).await
        }))
        .buffer_unordered(SCREENSHOT_SCAN_CONCURRENCY)
        .try_collect::<Vec<Vec<InstanceScreenshot>>>()
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    sort_screenshots(&mut screenshots);
    Ok(screenshots)
}

async fn lock_instance_screenshots<'a>(
    state: &State,
    instance_ids: impl IntoIterator<Item = &'a str>,
) -> Vec<OwnedMutexGuard<()>> {
    let mut locks = Vec::new();
    for instance_id in instance_ids {
        locks.push(state.lock_instance_screenshots(instance_id).await);
    }
    locks
}

pub async fn delete_screenshots(keys: &[ScreenshotKey]) -> crate::Result<()> {
    ensure_unique_keys(keys)?;
    let state = State::get().await?;
    let mut instance_ids = keys
        .iter()
        .map(|key| key.instance_id.as_str())
        .collect::<Vec<_>>();
    instance_ids.sort_unstable();
    instance_ids.dedup();
    let _locks = lock_instance_screenshots(&state, instance_ids).await;

    for key in keys {
        io::remove_file(get_screenshot_path(key).await?).await?;
        screenshot_rows::delete_screenshot_by_key(
            &key.instance_id,
            &key.file_name,
            &state.pool,
        )
        .await?;
    }

    Ok(())
}

pub async fn export_screenshots(
    keys: &[ScreenshotKey],
    export_path: PathBuf,
) -> crate::Result<()> {
    ensure_unique_keys(keys)?;
    if keys.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "At least one screenshot must be selected".to_string(),
        )
        .into());
    }

    let state = State::get().await?;
    let mut sources = HashMap::new();
    let mut archive_folders = HashMap::<String, String>::new();
    let mut used_archive_folders = HashSet::new();
    let mut screenshots = Vec::with_capacity(keys.len());

    for key in keys {
        let source = match sources.get(&key.instance_id) {
            Some(source) => source,
            None => {
                let source = instance_rows::get_instance_screenshot_source(
                    &key.instance_id,
                    &state.pool,
                )
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError("Unknown instance".to_string())
                })?;
                sources.insert(key.instance_id.clone(), source);
                &sources[&key.instance_id]
            }
        };
        let archive_folder = match archive_folders.get(&key.instance_id) {
            Some(folder) => folder.clone(),
            None => {
                let folder = unique_archive_folder(
                    &sanitize_archive_component(&source.name),
                    &mut used_archive_folders,
                );
                archive_folders.insert(key.instance_id.clone(), folder.clone());
                folder
            }
        };
        let archive_name = format!("{archive_folder}/{}", key.file_name);
        screenshots.push((archive_name, get_screenshot_path(key).await?));
    }

    tokio::task::spawn_blocking(move || -> crate::Result<()> {
        let file = std::fs::File::create(&export_path)
            .map_err(|error| IOError::with_path(error, &export_path))?;
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored);

        for (archive_name, path) in screenshots {
            writer
                .start_file(archive_name, options)
                .map_err(std::io::Error::from)?;
            let mut source = std::fs::File::open(&path)
                .map_err(|error| IOError::with_path(error, &path))?;
            std::io::copy(&mut source, &mut writer).map_err(IOError::from)?;
        }

        writer.finish().map_err(std::io::Error::from)?;
        Ok(())
    })
    .await??;

    Ok(())
}

pub async fn move_screenshots(
    keys: &[ScreenshotKey],
    target_instance_id: &str,
) -> crate::Result<Vec<ScreenshotKey>> {
    ensure_unique_keys(keys)?;
    if keys.is_empty() {
        return Ok(Vec::new());
    }

    let state = State::get().await?;
    let mut instance_ids = keys
        .iter()
        .map(|key| key.instance_id.as_str())
        .chain(std::iter::once(target_instance_id))
        .collect::<Vec<_>>();
    instance_ids.sort_unstable();
    instance_ids.dedup();
    let _locks = lock_instance_screenshots(&state, instance_ids).await;
    let target_source = instance_rows::get_instance_screenshot_source(
        target_instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown target instance".to_string())
    })?;
    let target_dir = source_screenshots_dir(&state, &target_source).await?;
    io::create_dir_all(&target_dir).await?;
    ensure_directory_is_not_symlink(&target_dir).await?;

    let mut moved_keys = Vec::with_capacity(keys.len());
    for key in keys {
        if key.instance_id == target_instance_id {
            continue;
        }

        let source_path = get_screenshot_path(key).await?;
        let target_path =
            available_target_path(&target_dir, &key.file_name).await?;
        io::rename_or_move(&source_path, &target_path)
            .await
            .map_err(|error| {
                crate::ErrorKind::InputError(format!(
                    "Could not move screenshot: {error}"
                ))
            })?;
        let file_name = target_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or_else(|| {
                crate::ErrorKind::InputError(
                    "Could not determine moved screenshot file name"
                        .to_string(),
                )
            })?
            .to_string();
        screenshot_rows::move_screenshot(
            &key.instance_id,
            &key.file_name,
            target_instance_id,
            &file_name,
            &state.pool,
        )
        .await?;
        moved_keys.push(ScreenshotKey {
            instance_id: target_instance_id.to_string(),
            file_name,
        });
    }

    Ok(moved_keys)
}

pub async fn get_screenshot_path(
    key: &ScreenshotKey,
) -> crate::Result<PathBuf> {
    validate_file_name(&key.file_name)?;

    let state = State::get().await?;
    let source = instance_rows::get_instance_screenshot_source(
        &key.instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let screenshots_dir = source_screenshots_dir(&state, &source).await?;
    let canonical_dir = tokio::fs::canonicalize(&screenshots_dir)
        .await
        .map_err(|error| IOError::with_path(error, &screenshots_dir))?;
    let requested_path = screenshots_dir.join(&key.file_name);
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

pub async fn save_edited_screenshot(
    key: ScreenshotKey,
    png_bytes: Vec<u8>,
    mode: ScreenshotEditSaveMode,
) -> crate::Result<InstanceScreenshot> {
    validate_file_name(&key.file_name)?;

    let state = State::get().await?;
    let source = instance_rows::get_instance_screenshot_source(
        &key.instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let _lock = state.lock_instance_screenshots(&source.id).await;

    let scanned = scan_source_screenshots(&state, &source).await?;
    let current =
        reconcile_source_screenshots(&state, &source, scanned).await?;
    let source_screenshot = current
        .iter()
        .find(|screenshot| screenshot.file_name == key.file_name)
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown screenshot".to_string())
        })?;
    let source_row = screenshot_rows::get_screenshot_by_key(
        &key.instance_id,
        &key.file_name,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown screenshot".to_string())
    })?;

    let source_dimensions =
        png_dimensions(io::read(&source_screenshot.path).await?).await?;
    let edited_dimensions = png_dimensions(png_bytes.clone()).await?;
    if edited_dimensions.0 > source_dimensions.0
        || edited_dimensions.1 > source_dimensions.1
    {
        return Err(crate::ErrorKind::InputError(format!(
            "Edited screenshot dimensions cannot exceed {}x{}",
            source_dimensions.0, source_dimensions.1,
        ))
        .into());
    }

    let screenshots_dir = source_screenshots_dir(&state, &source).await?;
    let (target_path, copy_group) = match mode {
        ScreenshotEditSaveMode::CreateCopy => (
            available_target_path(&screenshots_dir, &source_row.file_name)
                .await?,
            true,
        ),
        ScreenshotEditSaveMode::ReplaceEdit => {
            (source_screenshot.path.clone(), false)
        }
    };
    let target_file_name = target_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Could not determine edited screenshot file name".to_string(),
            )
        })?
        .to_string();

    io::write(&target_path, png_bytes).await?;
    if !copy_group {
        let (file_size, content_hash) = sha1_file_async(&target_path).await?;
        let file_size = i64::try_from(file_size).map_err(|_| {
            crate::ErrorKind::InputError(
                "Screenshot is too large to index".to_string(),
            )
        })?;
        let mut source_row = source_row.clone();
        source_row.content_hash = content_hash;
        source_row.file_size = file_size;
        let mut tx = state.pool.begin().await?;
        screenshot_rows::update_screenshot(&source_row, &mut tx).await?;
        tx.commit().await?;
    }
    let scanned = scan_source_screenshots(&state, &source).await?;
    let reconciled =
        reconcile_source_screenshots(&state, &source, scanned).await?;
    let mut saved = reconciled
        .into_iter()
        .find(|screenshot| screenshot.file_name == target_file_name)
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Could not index edited screenshot".to_string(),
            )
        })?;

    if copy_group {
        let result: crate::Result<()> = async {
            let mut tx = state.pool.begin().await?;
            screenshot_rows::copy_group_membership(
                &source_row.id,
                &saved.id,
                &mut tx,
            )
            .await?;
            tx.commit().await?;
            Ok(())
        }
        .await;
        if let Err(error) = result {
            let _ = io::remove_file(&target_path).await;
            let _ = screenshot_rows::delete_screenshot_by_key(
                &source.id,
                &target_file_name,
                &state.pool,
            )
            .await;
            return Err(error);
        }
    }

    let mut saved_row = screenshot_rows::get_screenshot_by_key(
        &saved.instance_id,
        &saved.file_name,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Could not load edited screenshot".to_string(),
        )
    })?;
    if !copy_group {
        let mut tx = state.pool.begin().await?;
        if saved_row.created_at != source_row.created_at {
            saved_row.created_at = source_row.created_at;
            screenshot_rows::update_screenshot(&saved_row, &mut tx).await?;
            saved.created_at =
                DateTime::from_timestamp_millis(source_row.created_at)
                    .ok_or_else(|| {
                        crate::ErrorKind::InputError(
                            "Screenshot creation time is out of range"
                                .to_string(),
                        )
                    })?;
        }
        tx.commit().await?;
    }
    saved.modified_at = saved_row.modified_at;
    saved.group_id = saved_row.group_id;

    let _ = emit_instance(
        &saved.instance_id,
        InstancePayloadType::ScreenshotsUpdated,
    )
    .await;

    Ok(saved)
}

pub(super) async fn source_screenshots_dir(
    state: &State,
    source: &InstanceScreenshotSource,
) -> crate::Result<PathBuf> {
    let instance_dir = state.directories.instances_dir().join(&source.path);
    let canonical_instance_dir =
        tokio::fs::canonicalize(&instance_dir)
            .await
            .map_err(|error| IOError::with_path(error, &instance_dir))?;
    let screenshots_dir = canonical_instance_dir.join(SCREENSHOTS_DIRECTORY);

    ensure_directory_is_not_symlink(&screenshots_dir).await?;
    Ok(screenshots_dir)
}

async fn ensure_directory_is_not_symlink(path: &Path) -> crate::Result<()> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(crate::ErrorKind::InputError(
                "Instance screenshots directory cannot be a symbolic link"
                    .to_string(),
            )
            .into())
        }
        Ok(metadata) if !metadata.is_dir() => {
            Err(crate::ErrorKind::InputError(
                "Instance screenshots path must be a directory".to_string(),
            )
            .into())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(IOError::with_path(error, path).into()),
    }
}

async fn available_target_path(
    target_dir: &Path,
    file_name: &str,
) -> crate::Result<PathBuf> {
    validate_file_name(file_name)?;
    let original = target_dir.join(file_name);
    if !tokio::fs::try_exists(&original)
        .await
        .map_err(|error| IOError::with_path(error, &original))?
    {
        return Ok(original);
    }

    let path = Path::new(file_name);
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap();
    let extension = path.extension().and_then(|value| value.to_str()).unwrap();

    for suffix in 1_u32.. {
        let candidate =
            target_dir.join(format!("{stem} ({suffix}).{extension}"));
        if !tokio::fs::try_exists(&candidate)
            .await
            .map_err(|error| IOError::with_path(error, &candidate))?
        {
            return Ok(candidate);
        }
    }

    unreachable!()
}

async fn png_dimensions(bytes: Vec<u8>) -> crate::Result<(u32, u32)> {
    tokio::task::spawn_blocking(move || {
        image::ImageReader::with_format(
            Cursor::new(bytes),
            image::ImageFormat::Png,
        )
        .decode()
        .map(|image| (image.width(), image.height()))
        .map_err(|error| {
            crate::ErrorKind::InputError(format!(
                "Could not decode screenshot as PNG: {error}"
            ))
            .into()
        })
    })
    .await
    .map_err(|error| {
        crate::ErrorKind::InputError(format!(
            "Could not validate screenshot: {error}"
        ))
    })?
}

fn ensure_unique_keys(keys: &[ScreenshotKey]) -> crate::Result<()> {
    let mut unique = HashSet::new();
    if keys.iter().all(|key| unique.insert(key)) {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "Screenshot selection contains duplicates".to_string(),
    )
    .into())
}

fn validate_file_name(file_name: &str) -> crate::Result<()> {
    let path = Path::new(file_name);
    let mut components = path.components();
    let is_single_file =
        matches!(components.next(), Some(Component::Normal(_)))
            && components.next().is_none();

    if !is_single_file || !has_png_extension(path) {
        return Err(crate::ErrorKind::InputError(
            "Invalid screenshot file name".to_string(),
        )
        .into());
    }

    Ok(())
}

pub(super) fn has_png_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
}

pub(super) fn sort_screenshots(screenshots: &mut [InstanceScreenshot]) {
    screenshots.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| left.instance_name.cmp(&right.instance_name))
            .then_with(|| left.file_name.cmp(&right.file_name))
    });
}

fn sanitize_archive_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => character,
        })
        .collect::<String>();

    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "Instance".to_string()
    } else {
        sanitized
    }
}

fn unique_archive_folder(
    base: &str,
    used_folders: &mut HashSet<String>,
) -> String {
    if used_folders.insert(base.to_string()) {
        return base.to_string();
    }

    for suffix in 2_u32.. {
        let candidate = format!("{base} ({suffix})");
        if used_folders.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!()
}
