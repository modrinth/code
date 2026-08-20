use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use tokio::fs::File;
use tokio_util::compat::FuturesAsyncWriteCompatExt;

use crate::State;
use crate::state::instances::adapters::sqlite::{
    instance_rows::{self, InstanceScreenshotSource},
    screenshot_rows::{self, ScreenshotRow},
};
use crate::util::fetch::sha1_file_async;
use crate::util::io::{self, IOError};
use uuid::Uuid;

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
    pub group_id: Option<String>,
    #[serde(skip)]
    pub path: PathBuf,
}

struct ScannedScreenshot {
    file_name: String,
    created_at: DateTime<Utc>,
    modified_at: i64,
    file_size: i64,
    path: PathBuf,
}

struct ResolvedScreenshot {
    scanned: ScannedScreenshot,
    row: ScreenshotRow,
    is_new: bool,
    changed: bool,
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

pub async fn delete_screenshots(keys: &[ScreenshotKey]) -> crate::Result<()> {
    ensure_unique_keys(keys)?;
    let state = State::get().await?;
    let mut instance_ids = keys
        .iter()
        .map(|key| key.instance_id.as_str())
        .collect::<Vec<_>>();
    instance_ids.sort_unstable();
    instance_ids.dedup();
    let mut locks = Vec::with_capacity(instance_ids.len());
    for instance_id in instance_ids {
        locks.push(state.lock_instance_screenshots(instance_id).await);
    }

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
                sources.get(&key.instance_id).unwrap()
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

    let mut file = File::create(&export_path)
        .await
        .map_err(|error| IOError::with_path(error, &export_path))?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    for (archive_name, path) in screenshots {
        let mut stream = writer
            .write_entry_stream(
                ZipEntryBuilder::new(archive_name.into(), Compression::Stored)
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
    let mut locks = Vec::with_capacity(instance_ids.len());
    for instance_id in instance_ids {
        locks.push(state.lock_instance_screenshots(instance_id).await);
    }
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

async fn list_source_screenshots(
    state: &State,
    source: InstanceScreenshotSource,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let _lock = state.lock_instance_screenshots(&source.id).await;
    let scanned = scan_source_screenshots(state, &source).await?;
    reconcile_source_screenshots(state, &source, scanned).await
}

pub(crate) async fn reconcile_screenshots(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let Some(source) =
        instance_rows::get_instance_screenshot_source(instance_id, &state.pool)
            .await?
    else {
        return Ok(());
    };

    list_source_screenshots(&state, source).await?;
    Ok(())
}

async fn scan_source_screenshots(
    state: &State,
    source: &InstanceScreenshotSource,
) -> crate::Result<Vec<ScannedScreenshot>> {
    let instance_dir = state.directories.instances_dir().join(&source.path);
    if !tokio::fs::try_exists(&instance_dir)
        .await
        .map_err(|error| IOError::with_path(error, &instance_dir))?
    {
        return Ok(Vec::new());
    }

    let screenshots_dir = source_screenshots_dir(state, &source).await?;
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
        let modified_at = metadata
            .modified()
            .map(DateTime::<Utc>::from)
            .map(|value| value.timestamp_millis())
            .map_err(|error| IOError::with_path(error, &path))?;
        let file_size = i64::try_from(metadata.len()).map_err(|_| {
            crate::ErrorKind::InputError(
                "Screenshot is too large to index".to_string(),
            )
        })?;

        screenshots.push(ScannedScreenshot {
            file_name,
            created_at,
            modified_at,
            file_size,
            path,
        });
    }

    Ok(screenshots)
}

async fn reconcile_source_screenshots(
    state: &State,
    source: &InstanceScreenshotSource,
    scanned: Vec<ScannedScreenshot>,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let existing =
        screenshot_rows::list_screenshots(&source.id, &state.pool).await?;
    let mut existing_by_name = existing
        .into_iter()
        .map(|row| (row.file_name.clone(), row))
        .collect::<HashMap<_, _>>();
    let mut resolved = Vec::with_capacity(scanned.len());
    let mut unmatched_scanned = Vec::new();

    for scanned in scanned {
        let Some(mut row) = existing_by_name.remove(&scanned.file_name) else {
            unmatched_scanned.push(scanned);
            continue;
        };
        let metadata_changed = row.file_size != scanned.file_size
            || row.modified_at != scanned.modified_at
            || row.created_at != scanned.created_at.timestamp_millis();
        if metadata_changed {
            let (_, hash) = sha1_file_async(&scanned.path).await?;
            row.content_hash = hash;
            row.file_size = scanned.file_size;
            row.modified_at = scanned.modified_at;
            row.created_at = scanned.created_at.timestamp_millis();
        }
        resolved.push(ResolvedScreenshot {
            scanned,
            row,
            is_new: false,
            changed: metadata_changed,
        });
    }

    let mut unmatched_by_hash =
        HashMap::<(String, i64), Vec<ScreenshotRow>>::new();
    for row in existing_by_name.into_values() {
        unmatched_by_hash
            .entry((row.content_hash.clone(), row.file_size))
            .or_default()
            .push(row);
    }

    for scanned in unmatched_scanned {
        let (file_size, content_hash) = sha1_file_async(&scanned.path).await?;
        let file_size = i64::try_from(file_size).map_err(|_| {
            crate::ErrorKind::InputError(
                "Screenshot is too large to index".to_string(),
            )
        })?;
        let hash_key = (content_hash.clone(), file_size);
        let matched = unmatched_by_hash.get_mut(&hash_key).and_then(|rows| {
            if rows.is_empty() {
                return None;
            }
            let created_at = scanned.created_at.timestamp_millis();
            let index = rows
                .iter()
                .position(|row| {
                    row.modified_at == scanned.modified_at
                        && row.created_at == created_at
                })
                .unwrap_or(rows.len() - 1);
            Some(rows.swap_remove(index))
        });
        let (mut row, is_new) = match matched {
            Some(row) => (row, false),
            None => (
                ScreenshotRow {
                    id: Uuid::new_v4().to_string(),
                    instance_id: source.id.clone(),
                    file_name: scanned.file_name.clone(),
                    content_hash: content_hash.clone(),
                    file_size,
                    modified_at: scanned.modified_at,
                    created_at: scanned.created_at.timestamp_millis(),
                    group_id: None,
                },
                true,
            ),
        };
        row.instance_id.clone_from(&source.id);
        row.file_name.clone_from(&scanned.file_name);
        row.content_hash = content_hash;
        row.file_size = file_size;
        row.modified_at = scanned.modified_at;
        row.created_at = scanned.created_at.timestamp_millis();
        resolved.push(ResolvedScreenshot {
            scanned,
            row,
            is_new,
            changed: !is_new,
        });
    }

    let mut tx = state.pool.begin().await?;
    for screenshot in &resolved {
        if screenshot.is_new {
            screenshot_rows::insert_screenshot(&screenshot.row, &mut tx)
                .await?;
        } else if screenshot.changed {
            screenshot_rows::update_screenshot(&screenshot.row, &mut tx)
                .await?;
        }
    }
    for row in unmatched_by_hash.into_values().flatten() {
        screenshot_rows::delete_screenshot(&row.id, &mut tx).await?;
    }
    tx.commit().await?;

    let mut screenshots = resolved
        .into_iter()
        .map(|screenshot| InstanceScreenshot {
            id: screenshot.row.id,
            instance_id: source.id.clone(),
            instance_name: source.name.clone(),
            file_name: screenshot.scanned.file_name,
            created_at: screenshot.scanned.created_at,
            group_id: screenshot.row.group_id,
            path: screenshot.scanned.path,
        })
        .collect::<Vec<_>>();
    sort_screenshots(&mut screenshots);
    Ok(screenshots)
}

async fn source_screenshots_dir(
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

fn has_png_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
}

fn sort_screenshots(screenshots: &mut [InstanceScreenshot]) {
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
