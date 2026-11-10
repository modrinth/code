//! 1. The user picks a new directory. The move runs on the next startup, before
//!    the rest of the app state is created.
//! 2. The old directory is walked. Folders and links are created at the
//!    destination, and every file is added to a list. Rebuildable caches are
//!    left out; `caches/icons` and `caches/game-locales` are kept because
//!    instances and saved settings point at them. Files already at the
//!    destination on the first attempt are recorded as protected.
//! 3. Files are copied in parallel, without syncing each one to disk. A copy
//!    keeps its source's modification time, so an unprotected file already at
//!    the destination with the same time is skipped. A protected file is only
//!    accepted if its contents match.
//! 4. Every copy is hashed and compared with its source. Mismatches are copied
//!    again, up to three attempts, after which the move fails. Protected files
//!    are never replaced.
//! 5. Every new file and folder is synced to disk once.
//! 6. Paths in the database are rewritten and the move is marked as copied.
//! 7. The old directory is deleted.
//!
//! Until step 6, the old directory is untouched and remains the source of
//! truth. If the move is interrupted before then, the next startup runs it
//! again and step 4 catches any copy left damaged. Cancelling only resets
//! settings.
//!
//! After step 6, the new directory is the source of truth. If the app stops
//! during step 7, the next startup finishes deleting the old directory.

use crate::event::emit::{emit_loading, init_loading};
use crate::state::content_store;
use crate::{ErrorKind, LoadingBarType};
mod managed_content;

use crate::state::content_store::{
    FileHashes, hash_file, normalize, relative_link, remove_instance_file,
    sync_directory,
};
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{Instance, JavaVersion};
use crate::util::content_hash::{copy_and_hash, hash_file_with_progress};
use anyhow::bail;
use futures::stream::{self, StreamExt, TryStreamExt};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const PARALLEL_FILES: usize = 8;
const COPY_ATTEMPTS: usize = 3;

type ProgressFn<'a> = &'a (dyn Fn(u64) + Send + Sync);

#[derive(Default)]
struct MovePlan {
    files: Vec<(PathBuf, PathBuf)>,
    directories: Vec<PathBuf>,
    /// Targets that existed before the move wrote anything. They are never replaced.
    protected: HashSet<PathBuf>,
    bytes: u64,
    required: u64,
}

impl MovePlan {
    async fn add_file(&mut self, source: PathBuf, target: PathBuf, len: u64) {
        self.bytes = self.bytes.saturating_add(len);
        if fs::symlink_metadata(&target).await.is_ok() {
            self.protected.insert(target.clone());
        } else {
            self.required = self.required.saturating_add(len);
        }
        self.files.push((source, target));
    }
}

const PROTECTED_TARGETS_KEY: &str = "store_directory_move_protected";
const PRESERVED_CACHE_ENTRIES: [&str; 2] = ["icons", "game-locales"];
const COPY_PROGRESS_LIMIT: f64 = 0.99;

const MOVED_APP_DIRECTORIES: [&str; 6] = [
    "store",
    "profiles",
    "meta",
    "caches",
    "icons",
    "synced-options",
];
pub(crate) async fn relocate_tree(from: &Path, to: &Path) -> crate::Result<()> {
    if !fs::try_exists(from).await? {
        return Ok(());
    }
    if fs::symlink_metadata(from).await?.file_type().is_symlink() {
        bail!(ErrorKind::InputError(
            "a shared-data root must not be a symlink during migration"
                .to_string(),
        ));
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).await?;
    }
    if !fs::try_exists(to).await?
        && crate::util::io::is_same_disk(from, to.parent().unwrap_or(to))
            .unwrap_or(false)
    {
        fs::rename(from, to).await?;
        if let Some(parent) = to.parent() {
            sync_directory(parent).await?;
        }
        return Ok(());
    }
    let mut plan = MovePlan::default();
    plan_tree(
        from,
        to,
        &[(from.to_path_buf(), to.to_path_buf())],
        &HashSet::new(),
        &mut plan,
    )
    .await?;
    copy_planned(&plan, to, &|_| {}).await
}

async fn plan_tree(
    from: &Path,
    to: &Path,
    mappings: &[(PathBuf, PathBuf)],
    skipped_paths: &HashSet<PathBuf>,
    plan: &mut MovePlan,
) -> crate::Result<()> {
    let mut pending = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((source, target)) = pending.pop() {
        if skipped_paths.contains(&source) {
            continue;
        }
        let metadata = fs::symlink_metadata(&source).await?;
        if metadata.file_type().is_symlink() {
            let link = fs::read_link(&source).await?;
            let resolved = normalize(
                &source
                    .parent()
                    .ok_or(ErrorKind::InputError(
                        "invalid symlink".to_string(),
                    ))?
                    .join(&link),
            );
            let mapped = remap(&resolved, mappings);
            let link = relative_link(
                &mapped,
                target.parent().ok_or(ErrorKind::InputError(
                    "invalid symlink destination".to_string(),
                ))?,
            );
            if let Ok(existing) = fs::symlink_metadata(&target).await {
                if existing.file_type().is_symlink()
                    && fs::read_link(&target).await? == link
                {
                    continue;
                }
                if existing.is_file()
                    && !existing.file_type().is_symlink()
                    && hash_file(&resolved).await? == hash_file(&target).await?
                {
                    continue;
                }
                bail!(ErrorKind::InputError(format!(
                    "migration destination conflicts with {}",
                    target.display()
                )));
            }
            create_link(&link, &target, &resolved).await?;
        } else if metadata.is_dir() {
            if let Ok(existing) = fs::symlink_metadata(&target).await
                && (!existing.is_dir() || existing.file_type().is_symlink())
            {
                bail!(ErrorKind::InputError(
                    "migration destination contains a directory link or conflicting file"
                        .to_string(),
                ));
            }
            fs::create_dir_all(&target).await?;
            let mut entries = fs::read_dir(&source).await?;
            while let Some(entry) = entries.next_entry().await? {
                pending.push((entry.path(), target.join(entry.file_name())));
            }
            plan.directories.push(target);
        } else if metadata.is_file() {
            plan.add_file(source, target, metadata.len()).await;
        }
    }
    Ok(())
}

async fn copy_planned(
    plan: &MovePlan,
    root: &Path,
    on_progress: ProgressFn<'_>,
) -> crate::Result<()> {
    let mut pending = (0..plan.files.len()).collect::<Vec<_>>();
    for attempt in 0..COPY_ATTEMPTS {
        let copied: Vec<_> = stream::iter(pending)
            .map(|index| async move {
                let (source, target) = &plan.files[index];
                copy_file(
                    source,
                    target,
                    plan.protected.contains(target),
                    attempt > 0,
                    on_progress,
                )
                .await
                .map(|hashes| (index, hashes))
            })
            .buffer_unordered(PARALLEL_FILES)
            .try_collect()
            .await?;
        let verified: Vec<_> = stream::iter(copied)
            .map(|(index, hashes)| async move {
                let (source, target) = &plan.files[index];
                verify_file(source, target, hashes, on_progress)
                    .await
                    .map(|matches| (index, matches))
            })
            .buffer_unordered(PARALLEL_FILES)
            .try_collect()
            .await?;
        pending = verified
            .into_iter()
            .filter_map(|(index, matches)| (!matches).then_some(index))
            .collect();
        if pending.is_empty() {
            break;
        }
        tracing::warn!(
            files = pending.len(),
            attempt,
            "Files failed verification during app directory move"
        );
    }
    if let Some((source, _)) = pending.first().map(|&index| &plan.files[index])
    {
        bail!(ErrorKind::InputError(format!(
            "{} could not be copied after {COPY_ATTEMPTS} attempts",
            source.display()
        )));
    }
    let paths = plan
        .files
        .iter()
        .map(|(_, target)| target)
        .chain(&plan.directories)
        .chain(std::iter::once(&root.to_path_buf()))
        .cloned()
        .collect::<Vec<_>>();
    stream::iter(paths)
        .map(|path| async move {
            Ok::<_, crate::Error>(
                tokio::task::spawn_blocking(move || flush_path(&path))
                    .await??,
            )
        })
        .buffer_unordered(PARALLEL_FILES)
        .try_collect::<Vec<_>>()
        .await?;
    #[cfg(target_os = "macos")]
    fs::File::open(root).await?.sync_all().await?;
    Ok(())
}

/// `None` when the file was already copied. Only targets the move wrote may be
/// trusted by modification time or replaced; `protected` targets must match.
async fn copy_file(
    source: &Path,
    target: &Path,
    protected: bool,
    replace: bool,
    on_progress: ProgressFn<'_>,
) -> crate::Result<Option<FileHashes>> {
    let metadata = fs::symlink_metadata(source).await?;
    if let Ok(existing) = fs::symlink_metadata(target).await {
        let replace = replace && !protected;
        if !existing.is_file()
            || (!replace
                && (protected || !is_copied_file(&metadata, &existing))
                && hash_file(source).await? != hash_file(target).await?)
        {
            bail!(ErrorKind::InputError(format!(
                "migration would overwrite different data at {}",
                target.display()
            )));
        }
        if !replace {
            on_progress(metadata.len());
            return Ok(None);
        }
        remove_instance_file(target).await?;
    }
    let parent = target.parent().ok_or(ErrorKind::InputError(
        "invalid migration target".to_string(),
    ))?;
    let temporary =
        parent.join(format!(".modrinth-move-{}.tmp", uuid::Uuid::new_v4()));
    let modified = metadata.modified()?;
    let mut output = fs::File::create(&temporary).await?;
    let hashes =
        copy_and_hash(fs::File::open(source).await?, &mut output, on_progress)
            .await?;
    output.flush().await?;
    output.into_std().await.set_modified(modified)?;
    fs::set_permissions(&temporary, metadata.permissions()).await?;
    let after = fs::metadata(source).await?;
    if after.len() != hashes.size || after.modified()? != modified {
        bail!(ErrorKind::InputError(format!(
            "{} changed during the app directory move",
            source.display()
        )));
    }
    fs::rename(&temporary, target).await?;
    Ok(Some(hashes))
}

async fn verify_file(
    source: &Path,
    target: &Path,
    hashes: Option<FileHashes>,
    on_progress: ProgressFn<'_>,
) -> crate::Result<bool> {
    let expected = match hashes {
        Some(hashes) => hashes,
        None => hash_file(source).await?,
    };
    Ok(hash_file_with_progress(target, on_progress)
        .await
        .is_ok_and(|actual| actual == expected))
}

/// - Linux and other Unix: `fsync`, the same as `sync_all`.
/// - macOS: `fsync`, as `sync_all` is a slow `F_FULLFSYNC` that flushes the
///   drive's cache. `copy_planned` issues it once after every path is flushed.
/// - Windows: `sync_all` (`FlushFileBuffers`) on files. Directories can't be
///   flushed there and are skipped.
fn flush_path(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        let file = std::fs::File::open(path)?;
        if unsafe { libc::fsync(file.as_raw_fd()) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(path)?;
        if metadata.is_dir() {
            return Ok(());
        }
        let permissions = metadata.permissions();
        if permissions.readonly() {
            let mut writable = permissions.clone();
            writable.set_readonly(false);
            std::fs::set_permissions(path, writable)?;
        }
        let result = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .and_then(|file| file.sync_all());
        if permissions.readonly() {
            std::fs::set_permissions(path, permissions)?;
        }
        result
    }
}

/// Everything in `caches` except `icons`, which instances reference by absolute
/// path, and `game-locales`, whose snapshots saved settings reference.
async fn rebuildable_cache_entries(root: &Path) -> crate::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    let mut caches = match fs::read_dir(root.join("caches")).await {
        Ok(caches) => caches,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(entries);
        }
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = caches.next_entry().await? {
        if !PRESERVED_CACHE_ENTRIES
            .iter()
            .any(|preserved| entry.file_name() == *preserved)
        {
            entries.push(entry.path());
        }
    }
    Ok(entries)
}

/// Copies keep the source's mtime, so for an unprotected target a match means
/// an earlier attempt wrote this file.
fn is_copied_file(
    source: &std::fs::Metadata,
    target: &std::fs::Metadata,
) -> bool {
    matches!(
        (source.modified(), target.modified()),
        (Ok(source), Ok(target)) if source == target
    )
}

async fn create_link(
    link: &Path,
    target: &Path,
    original: &Path,
) -> crate::Result<()> {
    #[cfg(unix)]
    {
        let _ = original;
        fs::symlink(link, target).await?;
    }
    #[cfg(windows)]
    {
        if fs::metadata(original).await?.is_dir() {
            fs::symlink_dir(link, target).await?;
        } else {
            match fs::symlink_file(link, target).await {
                Ok(()) => {}
                Err(error)
                    if crate::state::content_store::link_unavailable(
                        &error,
                    ) =>
                {
                    crate::state::content_store::writable_copy(
                        original, target,
                    )
                    .await?;
                    fs::File::options()
                        .write(true)
                        .open(target)
                        .await?
                        .sync_all()
                        .await?;
                    if hash_file(original).await? != hash_file(target).await? {
                        bail!(ErrorKind::InputError(
                            "symlink fallback copy changed during migration"
                                .to_string(),
                        ));
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}

fn remap(path: &Path, mappings: &[(PathBuf, PathBuf)]) -> PathBuf {
    let map = |path: &Path| {
        mappings.iter().find_map(|(from, to)| {
            strip_directory_prefix(path, from).map(|suffix| to.join(suffix))
        })
    };
    map(&normalize(path))
        .or_else(|| {
            std::fs::canonicalize(path).ok().and_then(|path| map(&path))
        })
        .unwrap_or_else(|| path.to_path_buf())
}

fn strip_directory_prefix<'a>(
    path: &'a Path,
    directory: &Path,
) -> Option<&'a Path> {
    let mut components = path.components();
    for expected in directory.components() {
        let actual = components.next()?;
        #[cfg(windows)]
        let matches = {
            use std::path::{Component, Prefix};
            match (actual, expected) {
                (Component::Prefix(actual), Component::Prefix(expected)) => {
                    match (actual.kind(), expected.kind()) {
                        (
                            Prefix::Disk(a) | Prefix::VerbatimDisk(a),
                            Prefix::Disk(b) | Prefix::VerbatimDisk(b),
                        ) => a.eq_ignore_ascii_case(&b),
                        (
                            Prefix::UNC(a, share_a)
                            | Prefix::VerbatimUNC(a, share_a),
                            Prefix::UNC(b, share_b)
                            | Prefix::VerbatimUNC(b, share_b),
                        ) => {
                            a.eq_ignore_ascii_case(b)
                                && share_a.eq_ignore_ascii_case(share_b)
                        }
                        _ => actual == expected,
                    }
                }
                _ => actual == expected,
            }
        };
        #[cfg(not(windows))]
        let matches = actual == expected;
        if !matches {
            return None;
        }
    }
    Some(components.as_path())
}

fn rewrite_value(value: &mut Value, mappings: &[(PathBuf, PathBuf)]) {
    match value {
        Value::String(value) => {
            let path = Path::new(value);
            if path.is_absolute() {
                *value = dunce::simplified(&remap(path, mappings))
                    .to_string_lossy()
                    .into_owned();
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_value(value, mappings);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                rewrite_value(value, mappings);
            }
        }
        _ => {}
    }
}

async fn rewrite_database_paths(
    pool: &SqlitePool,
    mappings: &[(PathBuf, PathBuf)],
    checkpoint: &str,
) -> crate::Result<()> {
    let java_versions = JavaVersion::get_all_registered(pool).await?;
    let instances = instance_rows::list_instances(pool).await?;
    let mut overrides_by_instance = std::collections::HashMap::new();
    for instance in &instances {
        overrides_by_instance.insert(
            instance.id.clone(),
            instance_rows::get_instance_launch_overrides(&instance.id, pool)
                .await?,
        );
    }
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    for (_, mut java) in java_versions {
        java.path = dunce::simplified(&remap(Path::new(&java.path), mappings))
            .to_string_lossy()
            .into_owned();
        java.upsert(&mut *tx).await?;
    }
    for instance in instances {
        let mut value = serde_json::to_value(&instance)?;
        rewrite_value(&mut value, mappings);
        let updated: Instance = serde_json::from_value(value)?;
        let overrides = overrides_by_instance.remove(&instance.id).flatten();
        instance_rows::update_instance(&updated, &mut tx).await?;
        if let Some(overrides) = overrides {
            let mut value = serde_json::to_value(&overrides)?;
            rewrite_value(&mut value, mappings);
            instance_rows::upsert_instance_launch_overrides(
                &serde_json::from_value(value)?,
                &mut tx,
            )
            .await?;
        }
    }
    let jobs = sqlx::query!(
        "SELECT id, json(state) AS \"state!: String\" FROM install_jobs"
    )
    .fetch_all(&mut *tx)
    .await?;
    for job in jobs {
        let mut state: Value = serde_json::from_str(&job.state)?;
        rewrite_value(&mut state, mappings);
        let state = serde_json::to_string(&state)?;
        sqlx::query!(
            "UPDATE install_jobs SET state = jsonb(?) WHERE id = ?",
            state,
            job.id
        )
        .execute(&mut *tx)
        .await?;
    }
    content_store::set_setting(
        &mut *tx,
        "store_directory_move_copied",
        checkpoint,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn move_app_directory(
    from: &Path,
    to: &Path,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let original_from = normalize(from);
    fs::create_dir_all(to).await?;
    let from = fs::canonicalize(from).await?;
    let to = fs::canonicalize(to).await?;
    if from == to {
        return Ok(());
    }
    if from.starts_with(&to) || to.starts_with(&from) {
        bail!(ErrorKind::InputError(
            "the new app directory cannot contain, or be inside, the old app directory"
                .to_string(),
        ));
    }
    let processes = sqlx::query!("SELECT pid, start_time FROM processes")
        .fetch_all(pool)
        .await?;
    let start_times = crate::state::running_process_start_times(
        processes.iter().map(|process| process.pid).collect(),
    )
    .await?;
    if processes.iter().any(|process| {
        start_times.get(&process.pid).is_some_and(|started_at| {
            started_at.abs_diff(process.start_time) <= 2
        })
    }) {
        bail!(ErrorKind::InputError(
            "close Minecraft instances before moving the app directory"
                .to_string(),
        ));
    }
    let mappings = [&from, &original_from]
        .into_iter()
        .flat_map(|source| {
            let destination = &to;
            MOVED_APP_DIRECTORIES.iter().map(move |directory| {
                (source.join(directory), destination.join(directory))
            })
        })
        .collect::<Vec<_>>();
    let checkpoint = serde_json::to_string(&(from.clone(), to.clone()))?;
    if let Some(previous) =
        content_store::setting(pool, "store_directory_move").await?
        && !previous.is_empty()
        && previous != checkpoint
    {
        bail!(ErrorKind::InputError(
            "finish or cancel the previous app-directory move before choosing another destination"
                .to_string(),
        ));
    }
    content_store::set_setting(pool, "store_directory_move", &checkpoint)
        .await?;
    let started = Instant::now();
    let loading = init_loading(
        LoadingBarType::DirectoryMove {
            old: dunce::simplified(&from).to_string_lossy().into_owned(),
            new: dunce::simplified(&to).to_string_lossy().into_owned(),
        },
        1.0,
        "Moving app directory",
    )
    .await?;
    let managed_files = managed_content::prepare(pool, &from, &to).await?;
    let mut skipped_paths: HashSet<PathBuf> = managed_files
        .iter()
        .map(|file| file.source.clone())
        .collect();
    skipped_paths.extend(rebuildable_cache_entries(&from).await?);
    let mut plan = MovePlan::default();
    for directory in MOVED_APP_DIRECTORIES {
        let source = from.join(directory);
        match fs::symlink_metadata(&source).await {
            Ok(metadata)
                if !metadata.is_dir() || metadata.file_type().is_symlink() =>
            {
                bail!(ErrorKind::InputError(format!(
                    "the shared-data root {} must be a directory without a symbolic link before moving the app directory",
                    source.display(),
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => return Err(error.into()),
        }
        plan_tree(
            &source,
            &to.join(directory),
            &mappings,
            &skipped_paths,
            &mut plan,
        )
        .await?;
    }
    for (source, target, len) in
        managed_content::checkpoint(pool, managed_files, &from, &to).await?
    {
        plan.add_file(source, target, len).await;
    }
    match content_store::setting(pool, PROTECTED_TARGETS_KEY)
        .await?
        .filter(|saved| !saved.is_empty())
        .map(|saved| serde_json::from_str::<(String, HashSet<PathBuf>)>(&saved))
        .transpose()?
    {
        Some((saved_checkpoint, protected))
            if saved_checkpoint == checkpoint =>
        {
            plan.protected = protected;
        }
        _ => {
            content_store::set_setting(
                pool,
                PROTECTED_TARGETS_KEY,
                &serde_json::to_string(&(&checkpoint, &plan.protected))?,
            )
            .await?;
        }
    }
    ensure_move_space(&to, plan.required)?;
    tracing::info!(
        from = %from.display(),
        to = %to.display(),
        files = plan.files.len(),
        bytes = plan.bytes,
        "Moving app directory"
    );
    let total = plan.bytes.saturating_mul(2).max(1) as f64;
    let processed = AtomicU64::new(0);
    let fraction =
        |bytes: u64| (bytes as f64 / total).min(1.0) * COPY_PROGRESS_LIMIT;
    let on_progress = |bytes: u64| {
        let before = processed.fetch_add(bytes, Ordering::Relaxed);
        let _ = emit_loading(
            &loading,
            fraction(before.saturating_add(bytes)) - fraction(before),
            None,
        );
    };
    copy_planned(&plan, &to, &on_progress).await?;
    rewrite_database_paths(pool, &mappings, &checkpoint).await?;
    tracing::info!(elapsed = ?started.elapsed(), "Copied app directory");
    Ok(())
}

pub(crate) async fn finish_app_directory_move(
    from: &Path,
    to: &Path,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let from = fs::canonicalize(from).await?;
    let to = fs::canonicalize(to).await?;
    if from == to {
        return Ok(());
    }
    if from.starts_with(&to) || to.starts_with(&from) {
        bail!(ErrorKind::InputError(
            "invalid app-directory cleanup roots".to_string()
        ));
    }
    let expected = serde_json::to_string(&(from.clone(), to.clone()))?;
    if content_store::setting(pool, "store_directory_move_copied")
        .await?
        .as_deref()
        != Some(expected.as_str())
    {
        bail!(ErrorKind::InputError(
            "the app-directory copy has not completed; preserving the source"
                .to_string(),
        ));
    }
    tracing::info!(from = %from.display(), "Removing old app directory data");
    let started = Instant::now();
    managed_content::commit(pool).await?;
    for directory in MOVED_APP_DIRECTORIES {
        let source = from.join(directory);
        if fs::try_exists(&source).await?
            && fs::try_exists(to.join(directory)).await?
        {
            remove_migrated_tree(&source).await?;
        }
    }
    content_store::set_setting(pool, "store_directory_move", "").await?;
    content_store::set_setting(pool, "store_directory_move_copied", "").await?;
    content_store::set_setting(pool, PROTECTED_TARGETS_KEY, "").await?;
    managed_content::clear_checkpoint(pool).await?;
    tracing::info!(elapsed = ?started.elapsed(), "Finished app directory move");
    Ok(())
}

fn ensure_move_space(destination: &Path, required: u64) -> crate::Result<()> {
    if fs4::available_space(destination)?
        < required.saturating_add(64 * 1024 * 1024)
    {
        bail!(ErrorKind::NotEnoughSpace { required });
    }
    Ok(())
}

pub(crate) async fn resume_completed_move(
    destination: &Path,
    pool: &SqlitePool,
) -> crate::Result<()> {
    if let Some(checkpoint) =
        content_store::setting(pool, "store_directory_move").await?
        && !checkpoint.is_empty()
    {
        let (from, to): (PathBuf, PathBuf) = serde_json::from_str(&checkpoint)?;
        if fs::canonicalize(destination).await? == to {
            tracing::info!("Resuming interrupted app directory move cleanup");
            finish_app_directory_move(&from, &to, pool).await?;
        }
    }
    Ok(())
}

pub(crate) async fn cancel_move(pool: &SqlitePool) -> crate::Result<()> {
    if let Some(checkpoint) =
        content_store::setting(pool, "store_directory_move").await?
        && !checkpoint.is_empty()
    {
        let (from, to): (PathBuf, PathBuf) = serde_json::from_str(&checkpoint)?;
        let settings = crate::state::Settings::get(pool).await?;
        let committed = match settings.prev_custom_dir.as_deref() {
            Some(previous) => fs::canonicalize(previous).await? == to,
            None => false,
        };
        if committed {
            bail!(ErrorKind::InputError(
                "the move has committed; finish its cleanup before moving back"
                    .to_string(),
            ));
        }
        let mappings = MOVED_APP_DIRECTORIES
            .iter()
            .map(|directory| (to.join(directory), from.join(directory)))
            .collect::<Vec<_>>();
        if content_store::setting(pool, "store_directory_move_copied")
            .await?
            .as_deref()
            == Some(checkpoint.as_str())
        {
            rewrite_database_paths(pool, &mappings, "").await?;
        }
        content_store::set_setting(pool, "store_directory_move", "").await?;
        content_store::set_setting(pool, "store_directory_move_copied", "")
            .await?;
        content_store::set_setting(pool, PROTECTED_TARGETS_KEY, "").await?;
        managed_content::clear_checkpoint(pool).await?;
    }
    Ok(())
}

pub(crate) async fn remove_migrated_tree(root: &Path) -> crate::Result<()> {
    if fs::symlink_metadata(root).await?.file_type().is_symlink() {
        bail!(ErrorKind::InputError(
            "a migrated directory was replaced by a link; preserving it"
                .to_string(),
        ));
    }
    #[cfg(windows)]
    {
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            let metadata = fs::symlink_metadata(&path).await?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                let mut entries = fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    pending.push(entry.path());
                }
            } else if metadata.is_file() && metadata.permissions().readonly() {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(false);
                fs::set_permissions(path, permissions).await?;
            }
        }
    }
    fs::remove_dir_all(root).await?;
    Ok(())
}

// dont usually need tests but cant fk this one up
#[cfg(test)]
mod tests {
    use super::{MovePlan, copy_planned, plan_tree};
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    async fn copy_tree(from: &Path, to: &Path) -> crate::Result<u64> {
        copy_tree_with(from, to, None).await
    }

    /// Runs again with the protected targets recorded by the first attempt.
    async fn resume_tree(from: &Path, to: &Path) -> crate::Result<u64> {
        copy_tree_with(from, to, Some(HashSet::new())).await
    }

    async fn copy_tree_with(
        from: &Path,
        to: &Path,
        protected: Option<HashSet<std::path::PathBuf>>,
    ) -> crate::Result<u64> {
        let mut plan = MovePlan::default();
        plan_tree(
            from,
            to,
            &[(from.to_path_buf(), to.to_path_buf())],
            &HashSet::new(),
            &mut plan,
        )
        .await?;
        if let Some(protected) = protected {
            plan.protected = protected;
        }
        let progress = AtomicU64::new(0);
        copy_planned(&plan, to, &|bytes| {
            progress.fetch_add(bytes, Ordering::Relaxed);
        })
        .await?;
        Ok(progress.into_inner())
    }

    fn modified(path: &Path) -> std::time::SystemTime {
        fs::metadata(path).unwrap().modified().unwrap()
    }

    #[tokio::test]
    async fn copies_tree_with_metadata_and_full_progress() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        let large: Vec<u8> = (0..600_000).map(|i| (i % 251) as u8).collect();
        fs::create_dir_all(from.join("mods/nested")).unwrap();
        fs::create_dir_all(from.join("empty")).unwrap();
        fs::write(from.join("mods/large.jar"), &large).unwrap();
        fs::write(from.join("mods/nested/config.toml"), b"key = 1").unwrap();
        let mut permissions =
            fs::metadata(from.join("mods/nested/config.toml"))
                .unwrap()
                .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(from.join("mods/nested/config.toml"), permissions)
            .unwrap();

        let progress = copy_tree(&from, &to).await.unwrap();

        assert_eq!(fs::read(to.join("mods/large.jar")).unwrap(), large);
        assert_eq!(
            fs::read(to.join("mods/nested/config.toml")).unwrap(),
            b"key = 1"
        );
        assert!(to.join("empty").is_dir());
        assert_eq!(
            modified(&to.join("mods/large.jar")),
            modified(&from.join("mods/large.jar"))
        );
        assert!(
            fs::metadata(to.join("mods/nested/config.toml"))
                .unwrap()
                .permissions()
                .readonly()
        );
        assert_eq!(progress, 2 * (large.len() as u64 + 7));
        assert!(fs::read_dir(to.join("mods")).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        }));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn second_run_skips_copied_files() {
        use std::os::unix::fs::MetadataExt;
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("file.bin"), b"contents").unwrap();
        copy_tree(&from, &to).await.unwrap();
        let inode = fs::metadata(to.join("file.bin")).unwrap().ino();

        resume_tree(&from, &to).await.unwrap();

        assert_eq!(fs::metadata(to.join("file.bin")).unwrap().ino(), inode);
    }

    #[tokio::test]
    async fn recopies_damaged_copy() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("file.bin"), b"original contents").unwrap();
        copy_tree(&from, &to).await.unwrap();
        fs::write(to.join("file.bin"), b"damaged").unwrap();
        fs::File::options()
            .write(true)
            .open(to.join("file.bin"))
            .unwrap()
            .set_modified(modified(&from.join("file.bin")))
            .unwrap();

        resume_tree(&from, &to).await.unwrap();

        assert_eq!(
            fs::read(to.join("file.bin")).unwrap(),
            b"original contents"
        );
    }

    #[tokio::test]
    async fn refuses_to_overwrite_existing_file_with_same_mtime() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        fs::create_dir_all(&from).unwrap();
        fs::create_dir_all(&to).unwrap();
        fs::write(from.join("file.bin"), b"source").unwrap();
        fs::write(to.join("file.bin"), b"someone else's data").unwrap();
        fs::File::options()
            .write(true)
            .open(to.join("file.bin"))
            .unwrap()
            .set_modified(modified(&from.join("file.bin")))
            .unwrap();

        assert!(copy_tree(&from, &to).await.is_err());
        assert_eq!(
            fs::read(to.join("file.bin")).unwrap(),
            b"someone else's data"
        );
    }

    #[tokio::test]
    async fn refuses_to_overwrite_different_file() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        fs::create_dir_all(&from).unwrap();
        fs::create_dir_all(&to).unwrap();
        fs::write(from.join("file.bin"), b"source").unwrap();
        fs::write(to.join("file.bin"), b"someone else's data").unwrap();

        assert!(copy_tree(&from, &to).await.is_err());
        assert_eq!(
            fs::read(to.join("file.bin")).unwrap(),
            b"someone else's data"
        );
    }

    #[tokio::test]
    async fn accepts_identical_existing_file() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        fs::create_dir_all(&from).unwrap();
        fs::create_dir_all(&to).unwrap();
        fs::write(from.join("file.bin"), b"same").unwrap();
        fs::write(to.join("file.bin"), b"same").unwrap();

        copy_tree(&from, &to).await.unwrap();

        assert_eq!(fs::read(to.join("file.bin")).unwrap(), b"same");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn remaps_links_into_the_new_directory() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("from");
        let to = root.path().join("to");
        fs::create_dir_all(from.join("store")).unwrap();
        fs::write(from.join("store/blob"), b"blob").unwrap();
        std::os::unix::fs::symlink(from.join("store/blob"), from.join("link"))
            .unwrap();

        copy_tree(&from, &to).await.unwrap();

        assert!(fs::read_link(to.join("link")).unwrap().is_relative());
        assert_eq!(
            fs::canonicalize(to.join("link")).unwrap(),
            fs::canonicalize(to.join("store/blob")).unwrap()
        );
    }
}
