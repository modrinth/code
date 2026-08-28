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
use crate::util::io::{self, IOError};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

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

pub(super) async fn reconcile_source_screenshots(
    state: &State,
    source: &InstanceScreenshotSource,
    scanned: Vec<ScannedScreenshot>,
) -> crate::Result<Vec<InstanceScreenshot>> {
    let existing =
        screenshot_rows::list_screenshots(&source.id, &state.pool).await?;
    let existing_by_name = existing
        .iter()
        .map(|row| (row.file_name.as_str(), row))
        .collect::<HashMap<_, _>>();
    let metadata_matches = existing.len() == scanned.len()
        && scanned.iter().all(|scanned| {
            existing_by_name
                .get(scanned.file_name.as_str())
                .is_some_and(|row| {
                    row.file_size == scanned.file_size
                        && row.modified_at == scanned.modified_at
                        && row.created_at
                            == scanned.created_at.timestamp_millis()
                })
        });
    let mut unmatched_by_hash =
        HashMap::<(String, i64), Vec<ScreenshotRow>>::new();
    let mut resolved = Vec::with_capacity(scanned.len());

    if metadata_matches {
        let mut existing_by_name = existing
            .into_iter()
            .map(|row| (row.file_name.clone(), row))
            .collect::<HashMap<_, _>>();
        for scanned in scanned {
            let row = existing_by_name.remove(&scanned.file_name).ok_or_else(
                || {
                    crate::ErrorKind::InputError(
                        "Screenshot index changed during reconciliation"
                            .to_string(),
                    )
                },
            )?;
            resolved.push(ResolvedScreenshot {
                scanned,
                row,
                is_new: false,
                changed: false,
            });
        }
    } else {
        let mut unmatched_by_name = existing
            .into_iter()
            .map(|row| (row.file_name.clone(), row))
            .collect::<HashMap<_, _>>();
        let mut hashed = Vec::with_capacity(scanned.len());
        for mut scanned in scanned {
            let (file_size, content_hash) =
                sha1_file_async(&scanned.path).await?;
            scanned.file_size = i64::try_from(file_size).map_err(|_| {
                crate::ErrorKind::InputError(
                    "Screenshot is too large to index".to_string(),
                )
            })?;
            hashed.push((scanned, content_hash));
        }

        let mut renamed_or_new = Vec::new();
        for (scanned, content_hash) in hashed {
            if let Some(row) = unmatched_by_name.remove(&scanned.file_name) {
                resolved.push(resolve_scanned_screenshot(
                    source,
                    scanned,
                    content_hash,
                    Some(row),
                ));
            } else {
                renamed_or_new.push((scanned, content_hash));
            }
        }

        for row in unmatched_by_name.into_values() {
            unmatched_by_hash
                .entry((row.content_hash.clone(), row.file_size))
                .or_default()
                .push(row);
        }

        for (scanned, content_hash) in renamed_or_new {
            let hash_key = (content_hash.clone(), scanned.file_size);
            let matched =
                unmatched_by_hash.get_mut(&hash_key).and_then(|rows| {
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
