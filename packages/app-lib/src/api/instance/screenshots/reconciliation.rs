use super::operations::{
    InstanceScreenshot, has_png_extension, sort_screenshots,
    source_screenshots_dir,
};
use crate::State;
use crate::state::instances::adapters::sqlite::{
    instance_rows::{self, InstanceScreenshotSource},
    screenshot_rows::{self, ScreenshotRow},
};
use crate::util::fetch::sha1_file_async;
use crate::util::io::IOError;
use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SCREENSHOT_HASH_CONCURRENCY: usize = 8;

pub(super) struct ScannedScreenshot {
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

fn resolve_scanned_screenshot(
    source: &InstanceScreenshotSource,
    scanned: ScannedScreenshot,
    content_hash: String,
    matched: Option<ScreenshotRow>,
) -> ResolvedScreenshot {
    let (mut row, is_new) = match matched {
        Some(row) => (row, false),
        None => (
            ScreenshotRow {
                id: Uuid::new_v4().to_string(),
                instance_id: source.id.clone(),
                file_name: scanned.file_name.clone(),
                content_hash: content_hash.clone(),
                file_size: scanned.file_size,
                modified_at: scanned.modified_at,
                created_at: scanned.created_at.timestamp_millis(),
                group_id: None,
            },
            true,
        ),
    };
    let changed = !is_new
        && (row.file_name != scanned.file_name
            || row.content_hash != content_hash
            || row.file_size != scanned.file_size
            || row.modified_at != scanned.modified_at
            || row.created_at != scanned.created_at.timestamp_millis());
    row.instance_id.clone_from(&source.id);
    row.file_name.clone_from(&scanned.file_name);
    row.content_hash = content_hash;
    row.file_size = scanned.file_size;
    row.modified_at = scanned.modified_at;
    row.created_at = scanned.created_at.timestamp_millis();

    ResolvedScreenshot {
        scanned,
        row,
        is_new,
        changed,
    }
}

pub(super) async fn list_source_screenshots(
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

pub(super) async fn scan_source_screenshots(
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

    let screenshots_dir = source_screenshots_dir(state, source).await?;
    tokio::task::spawn_blocking(move || scan_screenshots_dir(&screenshots_dir))
        .await?
}

fn scan_screenshots_dir(
    screenshots_dir: &Path,
) -> crate::Result<Vec<ScannedScreenshot>> {
    let entries = match std::fs::read_dir(screenshots_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };
    let mut screenshots = Vec::new();

    for entry in entries {
        let entry = entry
            .map_err(|error| IOError::with_path(error, screenshots_dir))?;
        let file_type = entry
            .file_type()
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

pub(super) async fn reconcile_source_screenshots(
    state: &State,
    source: &InstanceScreenshotSource,
    scanned: Vec<ScannedScreenshot>,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let existing =
        screenshot_rows::list_screenshots(&source.id, &state.pool).await?;
    let mut unmatched_by_name = existing
        .into_iter()
        .map(|row| (row.file_name.clone(), row))
        .collect::<HashMap<_, _>>();
    let mut resolved = Vec::with_capacity(scanned.len());
    let mut needs_hash = Vec::new();

    for scanned in scanned {
        match unmatched_by_name.remove(&scanned.file_name) {
            Some(row)
                if row.file_size == scanned.file_size
                    && row.modified_at == scanned.modified_at
                    && row.created_at
                        == scanned.created_at.timestamp_millis() =>
            {
                resolved.push(ResolvedScreenshot {
                    scanned,
                    row,
                    is_new: false,
                    changed: false,
                });
            }
            matched => needs_hash.push((scanned, matched)),
        }
    }

    let hashed = stream::iter(needs_hash.into_iter().map(
        |(mut scanned, matched)| async move {
            let (file_size, content_hash) =
                sha1_file_async(&scanned.path).await?;
            scanned.file_size = i64::try_from(file_size).map_err(|_| {
                crate::ErrorKind::InputError(
                    "Screenshot is too large to index".to_string(),
                )
            })?;
            Ok::<_, crate::Error>((scanned, content_hash, matched))
        },
    ))
    .buffer_unordered(SCREENSHOT_HASH_CONCURRENCY)
    .try_collect::<Vec<_>>()
    .await?;

    let mut unmatched_by_hash =
        HashMap::<(String, i64), Vec<ScreenshotRow>>::new();
    for row in unmatched_by_name.into_values() {
        unmatched_by_hash
            .entry((row.content_hash.clone(), row.file_size))
            .or_default()
            .push(row);
    }

    for (scanned, content_hash, matched_by_name) in hashed {
        if let Some(row) = matched_by_name {
            resolved.push(resolve_scanned_screenshot(
                source,
                scanned,
                content_hash,
                Some(row),
            ));
            continue;
        }

        let hash_key = (content_hash.clone(), scanned.file_size);
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
        resolved.push(resolve_scanned_screenshot(
            source,
            scanned,
            content_hash,
            matched,
        ));
    }

    let mut tx = state.pool.begin().await?;
    for row in unmatched_by_hash.into_values().flatten() {
        screenshot_rows::delete_screenshot(&row.id, &mut tx).await?;
    }
    for screenshot in &resolved {
        if !screenshot.is_new && screenshot.changed {
            let mut row = screenshot.row.clone();
            row.file_name = Uuid::new_v4().to_string();
            screenshot_rows::update_screenshot(&row, &mut tx).await?;
        }
    }
    for screenshot in &resolved {
        if screenshot.is_new {
            screenshot_rows::insert_screenshot(&screenshot.row, &mut tx)
                .await?;
        } else if screenshot.changed {
            screenshot_rows::update_screenshot(&screenshot.row, &mut tx)
                .await?;
        }
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
            modified_at: screenshot.row.modified_at,
            group_id: screenshot.row.group_id,
            path: screenshot.scanned.path,
        })
        .collect::<Vec<_>>();
    sort_screenshots(&mut screenshots);
    Ok(screenshots)
}
