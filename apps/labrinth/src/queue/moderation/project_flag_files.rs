use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use eyre::{Context, Result, ensure};
use regex::Regex;
use sha2::{Digest, Sha256};

use zip::ZipArchive;

use crate::database::ReadOnlyPgPool;
use crate::database::models::ids::{DBFileId, DBProjectId, DBVersionId};
use crate::env::ENV;
use crate::file_hosting::{FileHost, FileHostPublicity};

use crate::database::models::project_flag_item::PatternName;

use super::project_flag::ScannedFile;

// Bound each buffered artifact and text, and cumulative network/decompression work.
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TEXT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
// Count directories too; four archive layers includes the uploaded archive.
const MAX_ENTRIES: usize = 50_000;
const MAX_DEPTH: usize = 4;
const PREFIX_BYTES: usize = 4096;

type ScannedLeaf = (String, String, Vec<PatternName>);

#[derive(Default)]
struct Budget {
    bytes: u64,
    entries: usize,
}

impl Budget {
    fn consume(&mut self, bytes: u64) -> Result<()> {
        ensure!(
            bytes <= MAX_TOTAL_BYTES - self.bytes,
            "project file scan exceeds total byte budget"
        );
        self.bytes += bytes;
        Ok(())
    }
}

pub(super) async fn scan_project_files(
    project_id: DBProjectId,
    patterns: &BTreeMap<PatternName, Regex>,
    ro_pool: &ReadOnlyPgPool,
    file_host: &dyn FileHost,
) -> Result<Vec<ScannedFile>> {
    let files = sqlx::query!(
        r#"
        SELECT f.id, f.version_id, f.filename, f.url, f.size
        FROM files f
        INNER JOIN versions v ON v.id = f.version_id
        WHERE v.mod_id = $1
        ORDER BY v.date_published DESC, v.id DESC, f.id
        "#,
        project_id.0
    )
    .fetch_all(&**ro_pool)
    .await
    .wrap_err("fetching project files for content scanning")?;

    let mut scanned_files = Vec::new();
    let mut budget = Budget::default();
    for file in files {
        ensure!(
            file.size >= 0 && file.size as u64 <= MAX_ARTIFACT_BYTES,
            "file {} exceeds declared artifact size budget",
            file.id
        );
        ensure!(
            file.size as u64 <= MAX_TOTAL_BYTES - budget.bytes,
            "file {} exceeds remaining declared byte budget",
            file.id
        );
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
        let data = file_host
            .read_file(&key, FileHostPublicity::Public)
            .await
            .wrap_err_with(|| {
            format!("reading project file {} from storage", file.id)
        })?;
        ensure!(
            data.len() as u64 <= MAX_ARTIFACT_BYTES,
            "project file exceeds artifact size budget"
        );
        budget.consume(data.len() as u64)?;
        let path = format!("{}/{}", file.version_id, file.filename);
        let patterns = patterns.clone();
        let (artifact_leaves, next_budget) =
            tokio::task::spawn_blocking(move || {
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
                version_id: DBVersionId(file.version_id).into(),
                file_id: DBFileId(file.id).into(),
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
        ensure!(
            depth < MAX_DEPTH,
            "archive nesting depth budget exceeded at {path}"
        );
        let mut archive = ZipArchive::new(Cursor::new(data))
            .wrap_err_with(|| format!("opening archive {path}"))?;
        ensure!(
            archive.len() <= MAX_ENTRIES - budget.entries,
            "archive entry budget exceeded at {path}"
        );
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
                MAX_ARTIFACT_BYTES
            } else {
                MAX_TEXT_BYTES
            };
            ensure!(
                entry.size() <= limit,
                "archive entry size budget exceeded at {entry_path}"
            );
            ensure!(
                entry.size().saturating_sub(contents.len() as u64)
                    <= MAX_TOTAL_BYTES - budget.bytes,
                "remaining byte budget exceeded at {entry_path}"
            );
            loop {
                let count = entry
                    .read(&mut prefix)
                    .wrap_err("reading archive entry contents")?;
                if count == 0 {
                    break;
                }
                ensure!(
                    contents.len() as u64 + count as u64 <= limit,
                    "archive entry size budget exceeded at {entry_path}"
                );
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
        ensure!(
            data.len() as u64 <= MAX_TEXT_BYTES,
            "text size budget exceeded at {path}"
        );
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
