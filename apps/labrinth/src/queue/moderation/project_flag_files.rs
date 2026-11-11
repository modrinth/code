use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Read};
use std::sync::Arc;

use eyre::{Context, Result, ensure, eyre};
use regex::Regex;
use sha2::{Digest, Sha256};

use zip::ZipArchive;

use crate::database::ReadOnlyPgPool;
use crate::database::models::ids::{DBFileId, DBProjectId, DBVersionId};
use crate::env::ENV;
use crate::file_hosting::{FileHost, FileHostPublicity};

use crate::database::models::project_flag_item::PatternName;

use super::project_flag::ScannedFile;

const PREFIX_BYTES: usize = 4096;

type ScannedLeaf = (String, String, Vec<PatternName>);

#[derive(Default)]
struct Budget {
    bytes: u64,
    entries: usize,
}

impl Budget {
    fn consume(&mut self, bytes: u64) -> Result<()> {
        (bytes <= ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES - self.bytes).then_some(()).ok_or_else(|| eyre!("project file scan exceeds total byte budget")).wrap_err_with(|| eyre!("requested {bytes} bytes, remaining {} bytes, maximum {} bytes", ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES - self.bytes, ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES))?;
        self.bytes += bytes;
        Ok(())
    }
}

pub(super) struct ProjectScanFile {
    pub file_id: DBFileId,
    pub version_id: DBVersionId,
    pub filename: String,
    pub url: String,
    pub size: i64,
}

pub(super) async fn load_project_files(
    project_ids: &[DBProjectId],
    ro_pool: &ReadOnlyPgPool,
) -> Result<HashMap<DBProjectId, Vec<ProjectScanFile>>> {
    let mut files = project_ids
        .iter()
        .copied()
        .map(|id| (id, Vec::new()))
        .collect::<HashMap<_, _>>();
    if project_ids.is_empty() {
        return Ok(files);
    }
    let ids = project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
    let rows = sqlx::query!(
        r#"
        SELECT v.mod_id AS project_id, f.id, f.version_id, f.filename, f.url, f.size
        FROM files f
        INNER JOIN versions v ON v.id = f.version_id
        WHERE v.mod_id = ANY($1)
        ORDER BY v.mod_id, v.date_published DESC, v.id DESC, f.id
        "#,
        &ids
    )
    .fetch_all(&**ro_pool)
    .await
    .wrap_err("fetching project files for content scanning")?;

    for row in rows {
        files.entry(DBProjectId(row.project_id)).or_default().push(
            ProjectScanFile {
                file_id: DBFileId(row.id),
                version_id: DBVersionId(row.version_id),
                filename: row.filename,
                url: row.url,
                size: i64::from(row.size),
            },
        );
    }
    Ok(files)
}

pub(super) async fn scan_project_files(
    files: &[ProjectScanFile],
    patterns: &BTreeMap<PatternName, Regex>,
    file_host: &dyn FileHost,
    downloads: &Arc<tokio::sync::Semaphore>,
) -> Result<Vec<ScannedFile>> {
    let mut scanned_files = Vec::new();
    let mut budget = Budget::default();
    let patterns = Arc::new(patterns.clone());
    for file in files {
        (file.size >= 0
            && file.size as u64 <= ENV.PROJECT_FLAG_SCAN_MAX_ARTIFACT_BYTES)
            .then_some(())
            .ok_or_else(|| {
                eyre!(
                    "file {} exceeds declared artifact size budget",
                    file.file_id.0
                )
            })
            .wrap_err_with(|| {
                eyre!(
                    "declared size {} bytes, maximum {} bytes",
                    file.size,
                    ENV.PROJECT_FLAG_SCAN_MAX_ARTIFACT_BYTES
                )
            })?;
        (file.size as u64
                <= ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES - budget.bytes).then_some(()).ok_or_else(|| eyre!("file {} exceeds remaining declared byte budget",
            file.file_id.0)).wrap_err_with(|| eyre!("declared size {} bytes, remaining {} bytes, maximum {} bytes", file.size, ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES - budget.bytes, ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES))?;
        let cdn_prefix = format!("{}/", ENV.CDN_URL.trim_end_matches('/'));
        let key = file.url.strip_prefix(&cdn_prefix).ok_or_else(|| {
            eyre::eyre!("project file url is outside the configured cdn")
        })?;
        let key = urlencoding::decode(key)
            .wrap_err("decoding project file storage key")?;
        ensure!(
            !key.is_empty()
                && !key.contains('\\')
                && key.split('/').all(|part| !matches!(part, "" | "." | "..")),
            "invalid project file storage key"
        );
        let permit = Arc::clone(downloads)
            .acquire_owned()
            .await
            .wrap_err("acquiring project file download permit")?;
        let data = file_host
            .read_file(&key, FileHostPublicity::Public)
            .await
            .wrap_err_with(|| {
            format!("reading project file {} from storage", file.file_id.0)
        })?;

        (data.len() as u64 <= ENV.PROJECT_FLAG_SCAN_MAX_ARTIFACT_BYTES)
            .then_some(())
            .ok_or_else(|| eyre!("project file exceeds artifact size budget"))
            .wrap_err_with(|| {
                eyre!(
                    "actual size {} bytes, maximum {} bytes",
                    data.len(),
                    ENV.PROJECT_FLAG_SCAN_MAX_ARTIFACT_BYTES
                )
            })?;
        budget.consume(data.len() as u64)?;
        let path = format!("{}/{}", file.version_id.0, file.filename);
        let patterns = Arc::clone(&patterns);
        let (artifact_leaves, next_budget) =
            tokio::task::spawn_blocking(move || {
                let _permit = permit;
                let mut leaves = Vec::new();
                scan_contents(
                    &data,
                    &path,
                    0,
                    &patterns,
                    &mut budget,
                    &mut leaves,
                )?;
                eyre::Ok((leaves, budget))
            })
            .await
            .wrap_err("joining project file content scan")??;
        budget = next_budget;
        scanned_files.extend(artifact_leaves.into_iter().map(
            |(file_path, sha256, found_patterns)| ScannedFile {
                version_id: file.version_id.into(),
                file_id: file.file_id.into(),
                file_path,
                sha256,
                found_patterns,
            },
        ));
    }
    Ok(scanned_files)
}

fn is_archive(data: &[u8], path: &str) -> bool {
    data.starts_with(b"PK\x03\x04")
        || data.starts_with(b"PK\x05\x06")
        || data.starts_with(b"PK\x07\x08")
        || path.rsplit('.').next().is_some_and(|extension| {
            ["zip", "jar", "mrpack"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

fn is_text_prefix(data: &[u8]) -> bool {
    let text = match std::str::from_utf8(data) {
        Ok(text) => text,
        Err(error) if error.error_len().is_none() => {
            std::str::from_utf8(&data[..error.valid_up_to()])
                .unwrap_or_default()
        }
        Err(_) => return false,
    };
    !text.chars().any(|character| {
        character == '\0'
            || (character.is_control() && !character.is_whitespace())
    })
}

fn scan_contents(
    data: &[u8],
    path: &str,
    depth: usize,
    patterns: &BTreeMap<PatternName, Regex>,
    budget: &mut Budget,
    leaves: &mut Vec<ScannedLeaf>,
) -> Result<()> {
    if is_archive(data, path) {
        (depth < ENV.PROJECT_FLAG_SCAN_MAX_DEPTH)
            .then_some(())
            .ok_or_else(|| {
                eyre!("archive nesting depth budget exceeded at {path}")
            })
            .wrap_err_with(|| {
                eyre!(
                    "attempted archive layer {}, maximum {} layers",
                    depth + 1,
                    ENV.PROJECT_FLAG_SCAN_MAX_DEPTH
                )
            })?;
        let mut archive = ZipArchive::new(Cursor::new(data))
            .wrap_err_with(|| format!("opening archive {path}"))?;
        (archive.len() <= ENV.PROJECT_FLAG_SCAN_MAX_ENTRIES - budget.entries).then_some(()).ok_or_else(|| eyre!("archive entry budget exceeded at {path}")).wrap_err_with(|| eyre!("archive contains {} entries, remaining {} entries, maximum {} entries", archive.len(), ENV.PROJECT_FLAG_SCAN_MAX_ENTRIES - budget.entries, ENV.PROJECT_FLAG_SCAN_MAX_ENTRIES))?;
        budget.entries += archive.len();
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .wrap_err_with(|| format!("opening entry {index} in {path}"))?;
            let entry_path = format!(
                "{path}!/{}",
                entry.name().wrap_err("decoding archive entry name")?
            );
            if entry.is_dir() {
                continue;
            }
            let mut contents = Vec::new();
            let mut prefix = [0; PREFIX_BYTES];
            while contents.len() < PREFIX_BYTES {
                let remaining = PREFIX_BYTES - contents.len();
                let count = entry
                    .read(&mut prefix[..remaining])
                    .wrap_err("reading archive entry prefix")?;
                if count == 0 {
                    break;
                }
                budget.consume(count as u64)?;
                contents.extend_from_slice(&prefix[..count]);
            }
            let archive_entry = is_archive(&contents, &entry_path);
            if !archive_entry && !is_text_prefix(&contents) {
                continue;
            }
            let limit = if archive_entry {
                ENV.PROJECT_FLAG_SCAN_MAX_ARTIFACT_BYTES
            } else {
                ENV.PROJECT_FLAG_SCAN_MAX_TEXT_BYTES
            };
            (entry.size() <= limit)
                .then_some(())
                .ok_or_else(|| {
                    eyre!("archive entry size budget exceeded at {entry_path}")
                })
                .wrap_err_with(|| {
                    eyre!(
                        "declared size {} bytes, maximum {limit} bytes",
                        entry.size()
                    )
                })?;
            (entry.size().saturating_sub(contents.len() as u64)
                    <= ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES - budget.bytes).then_some(()).ok_or_else(|| eyre!("remaining byte budget exceeded at {entry_path}")).wrap_err_with(|| eyre!("requested {} bytes, remaining {} bytes, maximum {} bytes", entry.size().saturating_sub(contents.len() as u64), ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES - budget.bytes, ENV.PROJECT_FLAG_SCAN_MAX_TOTAL_BYTES))?;
            loop {
                let count = entry
                    .read(&mut prefix)
                    .wrap_err("reading archive entry contents")?;
                if count == 0 {
                    break;
                }
                (contents.len() as u64 + count as u64 <= limit).then_some(()).ok_or_else(|| eyre!("archive entry size budget exceeded at {entry_path}")).wrap_err_with(|| eyre!("actual size at least {} bytes, maximum {limit} bytes", contents.len() as u64 + count as u64))?;
                budget.consume(count as u64)?;
                contents.extend_from_slice(&prefix[..count]);
            }
            scan_contents(
                &contents,
                &entry_path,
                depth + 1,
                patterns,
                budget,
                leaves,
            )?;
        }
    } else if is_text_prefix(&data[..data.len().min(PREFIX_BYTES)]) {
        (data.len() as u64 <= ENV.PROJECT_FLAG_SCAN_MAX_TEXT_BYTES)
            .then_some(())
            .ok_or_else(|| eyre!("text size budget exceeded at {path}"))
            .wrap_err_with(|| {
                eyre!(
                    "actual size {} bytes, maximum {} bytes",
                    data.len(),
                    ENV.PROJECT_FLAG_SCAN_MAX_TEXT_BYTES
                )
            })?;
        if let Ok(text) = std::str::from_utf8(data)
            && is_text_prefix(data)
        {
            let found_patterns = patterns
                .iter()
                .filter(|(_, pattern)| pattern.is_match(text))
                .map(|(name, _)| name.clone())
                .collect();
            leaves.push((
                path.to_owned(),
                hex::encode(Sha256::digest(data)),
                found_patterns,
            ));
        }
    }
    Ok(())
}
