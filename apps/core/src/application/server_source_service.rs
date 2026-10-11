//! Linked server sources: one engine that installs and updates a server from a full setup
//! (game version, loader, files with hashes). Adapters turn an uploaded `.mrpack` (a pack
//! file, or an instance the App exported) or a Modrinth pack version into a [`Setup`] staged
//! on disk; [`apply`] then reconciles the server against it, only while the server is stopped.
//!
//! Durable state lives in `server_sources` / `server_source_files` (migration 023), so a Core
//! restart resumes pending work. Files the source previously provided are tracked with their
//! last-applied hash; anything else on disk is server content and is never touched.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};

use dashmap::DashMap;
use sha1::Digest;
use tracing::{error, warn};

use crate::{
    application::{
        instance_service::{self, InstanceError},
        instance_status_service,
        state::AppState,
    },
    domain::{
        event::Event,
        instance::{InstanceId, ModLoader},
        modpack::EnvType,
        server_source::{
            ContentKind, ContentOrigin, ServerContent, ServerContentItem,
            ServerSource, ServerSourceStatus, Setup, SetupFile,
            SourceApplyPhase, SourceApplyState,
        },
    },
    infrastructure::minecraft::{
        modrinth_api::{ModrinthClient, ModrinthError},
        mrpack::{extract_metadata, validate_download_url, MrpackError},
        server_properties::read_properties,
    },
    ports::instance_store::StoreError,
};

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("instance not found")]
    NotFound,
    #[error("server is already linked to a source")]
    AlreadyLinked,
    #[error("server is not linked to a source")]
    NotLinked,
    #[error("{0}")]
    Invalid(String),
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("download failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("modrinth: {0}")]
    Modrinth(#[from] ModrinthError),
    #[error("{0}")]
    Mrpack(#[from] MrpackError),
    #[error("{0}")]
    Instance(#[from] InstanceError),
    #[error("store: {0}")]
    Store(#[from] StoreError),
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct SourceRow {
    source_json: String,
    desired_version: String,
    installed_version: Option<String>,
    staged_version: Option<String>,
    state: String,
    error: Option<String>,
    restart_requested: i64,
    updated_at: String,
}

impl SourceRow {
    fn source(&self) -> Result<ServerSource, SourceError> {
        serde_json::from_str(&self.source_json)
            .map_err(|e| SourceError::Invalid(e.to_string()))
    }

    fn status(&self) -> Result<ServerSourceStatus, SourceError> {
        Ok(ServerSourceStatus {
            source: self.source()?,
            desired_version: self.desired_version.clone(),
            installed_version: self.installed_version.clone(),
            state: SourceApplyState::parse(&self.state),
            error: self.error.clone(),
            updated_at: self.updated_at.clone(),
        })
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

pub async fn get_status(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<Option<ServerSourceStatus>, SourceError> {
    load_row(state, instance_id)
        .await?
        .map(|row| row.status())
        .transpose()
}

/// Link an unlinked server to a source and start installing it in the background.
pub async fn link(
    state: &Arc<AppState>,
    instance_id: &str,
    source: ServerSource,
    version: &str,
) -> Result<ServerSourceStatus, SourceError> {
    instance_record(state, instance_id).await?;
    if load_row(state, instance_id).await?.is_some() {
        return Err(SourceError::AlreadyLinked);
    }
    sqlx::query("INSERT INTO server_sources (instance_id, source_json, desired_version, state, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(instance_id)
        .bind(serde_json::to_string(&source).map_err(|e| SourceError::Invalid(e.to_string()))?)
        .bind(version)
        .bind(SourceApplyState::Downloading.as_str())
        .bind(now())
        .execute(&state.pool)
        .await?;
    let status = broadcast(state, instance_id).await?;
    spawn_process(state, instance_id);
    status.ok_or(SourceError::NotLinked)
}

/// Link a new server to an instance before the instance's pack has arrived, so the server shows
/// as linked from the moment it exists. It stays failed, with nothing to apply, until the App
/// uploads the pack with [`link_mrpack`].
pub async fn link_awaiting_upload(
    state: &Arc<AppState>,
    instance_id: &str,
    source: ServerSource,
) -> Result<(), SourceError> {
    sqlx::query("INSERT INTO server_sources (instance_id, source_json, desired_version, state, error, updated_at) VALUES (?, ?, '', ?, ?, ?)")
        .bind(instance_id)
        .bind(serde_json::to_string(&source).map_err(|e| SourceError::Invalid(e.to_string()))?)
        .bind(SourceApplyState::Failed.as_str())
        .bind(AWAITING_UPLOAD)
        .bind(now())
        .execute(&state.pool)
        .await?;
    Ok(())
}

const AWAITING_UPLOAD: &str =
    "The instance has not reached this server yet. Push an update from the instance.";

/// Unlink a server. Inherited files stay on disk and become server content.
pub async fn unlink(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<(), SourceError> {
    let deleted =
        sqlx::query("DELETE FROM server_sources WHERE instance_id = ?")
            .bind(instance_id)
            .execute(&state.pool)
            .await?
            .rows_affected();
    if deleted == 0 {
        return Err(SourceError::NotLinked);
    }
    let _ = tokio::fs::remove_dir_all(staging_root(state, instance_id)).await;
    broadcast(state, instance_id).await?;
    Ok(())
}

/// Record a newer desired version. Core downloads and verifies it now and applies it when the
/// server is stopped; [`apply_now`] restarts a running server instead of waiting.
pub async fn request_source_update(
    state: &Arc<AppState>,
    instance_id: &str,
    version: &str,
) -> Result<ServerSourceStatus, SourceError> {
    let row = load_row(state, instance_id)
        .await?
        .ok_or(SourceError::NotLinked)?;
    if row.installed_version.as_deref() == Some(version)
        && row.desired_version == version
    {
        return row.status();
    }
    sqlx::query("UPDATE server_sources SET desired_version = ?, restart_requested = 0, state = ?, error = NULL, updated_at = ? WHERE instance_id = ?")
        .bind(version)
        .bind(SourceApplyState::Downloading.as_str())
        .bind(now())
        .bind(instance_id)
        .execute(&state.pool)
        .await?;
    let status = broadcast(state, instance_id).await?;
    spawn_process(state, instance_id);
    status.ok_or(SourceError::NotLinked)
}

/// Apply the latest requested version without waiting for the next stop: a running server is
/// stopped, updated, and started again. Also retries a failed update.
pub async fn apply_now(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<ServerSourceStatus, SourceError> {
    let row = load_row(state, instance_id)
        .await?
        .ok_or(SourceError::NotLinked)?;
    // An empty desired version is a server still waiting for its first upload: nothing to retry.
    if row.desired_version.is_empty()
        || row.installed_version.as_deref() == Some(row.desired_version.as_str())
    {
        return row.status();
    }
    let next = if row.staged_version.as_deref()
        == Some(row.desired_version.as_str())
    {
        SourceApplyState::Pending
    } else {
        SourceApplyState::Downloading
    };
    sqlx::query("UPDATE server_sources SET restart_requested = 1, state = ?, error = NULL, updated_at = ? WHERE instance_id = ?")
        .bind(next.as_str())
        .bind(now())
        .bind(instance_id)
        .execute(&state.pool)
        .await?;
    let status = broadcast(state, instance_id).await?;
    spawn_process(state, instance_id);
    status.ok_or(SourceError::NotLinked)
}

/// Mods and datapacks on a server, from the source and added by the server.
pub async fn content(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<ServerContent, SourceError> {
    let record = instance_record(state, instance_id).await?;
    let data_dir = PathBuf::from(&record.data_dir);
    let level = level_name(&data_dir).await;
    let row = load_row(state, instance_id).await?;
    let tracked = tracked_files(state, instance_id).await?;
    let mut disk = Vec::new();
    for (dir, prefix, kind) in [
        (data_dir.join("mods"), "mods", ContentKind::Mod),
        (
            data_dir.join(&level).join("datapacks"),
            "datapacks",
            ContentKind::Datapack,
        ),
    ] {
        let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
            continue;
        };
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_mod = name.ends_with(".jar") || name.ends_with(".jar.disabled");
            if kind == ContentKind::Mod && !is_mod {
                continue;
            }
            disk.push((kind, format!("{prefix}/{name}")));
        }
    }
    Ok(ServerContent {
        source: row.map(|row| row.status()).transpose()?,
        items: content_items(&tracked, &disk),
    })
}

/// Whether the linked source provides this mod file (by its enabled or disabled name).
pub async fn provides(
    state: &Arc<AppState>,
    instance_id: &str,
    filename: &str,
) -> Result<bool, SourceError> {
    let path =
        format!("mods/{}", filename.strip_suffix(".disabled").unwrap_or(filename));
    Ok(tracked_files(state, instance_id)
        .await?
        .iter()
        .any(|file| file.path == path))
}

/// Resume unfinished work after a Core restart.
pub async fn resume(state: Arc<AppState>) {
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT instance_id FROM server_sources WHERE state IN ('downloading', 'pending', 'applying')",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    for id in ids {
        spawn_process(&state, &id);
    }
}

/// Called when a server process exits so a pending update applies.
pub fn on_stopped(state: &Arc<AppState>, instance_id: &InstanceId) {
    spawn_process(state, &instance_id.to_string());
}

// ── Worker ───────────────────────────────────────────────────────────────────

/// Instances with a running worker. One worker per server; it re-reads the desired version
/// after every step, so only the latest requested version is ever applied.
static WORKERS: LazyLock<DashMap<String, ()>> = LazyLock::new(DashMap::new);

fn spawn_process(state: &Arc<AppState>, instance_id: &str) {
    tokio::spawn(process(Arc::clone(state), instance_id.to_string()));
}

async fn process(state: Arc<AppState>, instance_id: String) {
    loop {
        if WORKERS.insert(instance_id.clone(), ()).is_some() {
            return;
        }
        loop {
            match step(&state, &instance_id).await {
                Ok(true) => continue,
                Ok(false) => break,
                Err(err) => {
                    error!("Source update for {instance_id} failed: {err}");
                    let _ = set_failed(&state, &instance_id, &err).await;
                    break;
                }
            }
        }
        WORKERS.remove(&instance_id);
        // A request may have arrived after the last step but before the worker released.
        match load_row(&state, &instance_id).await {
            Ok(Some(row)) if needs_work(&row) => continue,
            _ => return,
        }
    }
}

fn needs_work(row: &SourceRow) -> bool {
    matches!(
        SourceApplyState::parse(&row.state),
        SourceApplyState::Downloading | SourceApplyState::Applying
    ) || (row.state == "pending" && row.staged_version.is_some())
}

/// Advance one step. Returns `true` when another step should run immediately.
async fn step(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<bool, SourceError> {
    let Some(row) = load_row(state, instance_id).await? else {
        return Ok(false);
    };
    let current = SourceApplyState::parse(&row.state);
    if current == SourceApplyState::Failed {
        return Ok(false);
    }
    if row.installed_version.as_deref() == Some(row.desired_version.as_str())
    {
        if current != SourceApplyState::UpToDate {
            set_state(state, instance_id, SourceApplyState::UpToDate).await?;
        }
        return Ok(false);
    }
    let version = row.desired_version.clone();
    if row.staged_version.as_deref() != Some(version.as_str()) {
        set_state(state, instance_id, SourceApplyState::Downloading).await?;
        progress(state, instance_id, &version, SourceApplyPhase::Downloading);
        let dir = staging_dir(state, instance_id, &version);
        // Only the extracted files are rebuilt; an uploaded archive in `dir` is the input.
        let _ = tokio::fs::remove_dir_all(dir.join("files")).await;
        tokio::fs::create_dir_all(dir.join("files")).await?;
        let setup = stage(state, &row, &version, &dir).await?;
        progress(state, instance_id, &version, SourceApplyPhase::Verifying);
        tokio::fs::write(
            dir.join("setup.json"),
            serde_json::to_vec(&setup)
                .map_err(|e| SourceError::Invalid(e.to_string()))?,
        )
        .await?;
        sqlx::query("UPDATE server_sources SET staged_version = ?, state = ?, updated_at = ? WHERE instance_id = ? AND desired_version = ?")
            .bind(&version)
            .bind(SourceApplyState::Pending.as_str())
            .bind(now())
            .bind(instance_id)
            .bind(&version)
            .execute(&state.pool)
            .await?;
        broadcast(state, instance_id).await?;
        return Ok(true);
    }

    let iid: InstanceId = instance_id.parse().map_err(|_| SourceError::NotFound)?;
    let running = state.instances.contains_key(&iid);
    // Only a server this update stopped is started again.
    let restart = running && row.restart_requested != 0;
    if running {
        if !restart {
            if current != SourceApplyState::Pending {
                set_state(state, instance_id, SourceApplyState::Pending).await?;
            }
            progress(state, instance_id, &version, SourceApplyPhase::WaitingForStop);
            return Ok(false);
        }
        progress(state, instance_id, &version, SourceApplyPhase::Stopping);
        instance_status_service::stop_instance(state, &iid).await?;
        wait_for_stop(state, &iid).await?;
    }
    if !apply(state, &iid, &row, &version).await? {
        // Started again before the apply took the lock. Wait for the next stop.
        set_state(state, instance_id, SourceApplyState::Pending).await?;
        return Ok(false);
    }
    if restart {
        progress(state, instance_id, &version, SourceApplyPhase::Starting);
        if let Err(err) = instance_status_service::start_instance(state, &iid).await {
            warn!("Restart after source update failed for {instance_id}: {err}");
        }
    }
    Ok(true)
}

async fn wait_for_stop(
    state: &Arc<AppState>,
    id: &InstanceId,
) -> Result<(), SourceError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    while state.instances.contains_key(id) {
        if tokio::time::Instant::now() >= deadline {
            return Err(SourceError::Invalid(
                "Timed out waiting for the server to stop".into(),
            ));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

/// Apply the staged version while holding the instance operation lock, so a start cannot race
/// it. Returns `false` if the server is running. On any failure the previous files are
/// restored and `installed_version` is unchanged.
async fn apply(
    state: &Arc<AppState>,
    iid: &InstanceId,
    row: &SourceRow,
    version: &str,
) -> Result<bool, SourceError> {
    let lock = state
        .instance_operation_locks
        .entry(iid.clone())
        .or_default()
        .clone();
    let _guard = lock.lock().await;
    if state.instances.contains_key(iid) {
        return Ok(false);
    }
    let instance_id = iid.to_string();
    set_state(state, &instance_id, SourceApplyState::Applying).await?;
    progress(state, &instance_id, version, SourceApplyPhase::Applying);

    let staged = staging_dir(state, &instance_id, version);
    let setup: Setup = serde_json::from_slice(
        &tokio::fs::read(staged.join("setup.json")).await?,
    )
    .map_err(|e| SourceError::Invalid(e.to_string()))?;
    let record = state.instance_store.get(iid).await?;
    let data_dir = PathBuf::from(&record.data_dir);
    let level = level_name(&data_dir).await;
    let previous = tracked_files(state, &instance_id).await?;

    let mut on_disk = HashMap::new();
    for file in setup.files.iter().filter(|f| !f.client_only) {
        if let Ok(bytes) =
            tokio::fs::read(data_dir.join(disk_path(&file.path, &level))).await
        {
            on_disk.insert(file.path.clone(), sha1_hex(&bytes));
        }
    }
    let plan = plan(&previous, &setup.files, &on_disk);

    // Back up everything the plan touches so a failure can restore it.
    let backup = staged.join("backup");
    let mut existed = Vec::new();
    for path in plan.remove.iter().chain(&plan.write) {
        let target = data_dir.join(disk_path(path, &level));
        if tokio::fs::metadata(&target).await.is_ok_and(|m| m.is_file()) {
            copy_file(&target, &backup.join(path)).await?;
            existed.push(path.clone());
        }
    }

    let result = async {
        for path in &plan.remove {
            match tokio::fs::remove_file(data_dir.join(disk_path(path, &level))).await {
                Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
                    return Err(SourceError::Io(err))
                }
                _ => {}
            }
        }
        for path in &plan.write {
            copy_file(
                &staged.join("files").join(path),
                &data_dir.join(disk_path(path, &level)),
            )
            .await?;
        }
        if record.game_version != setup.game_version
            || record.loader != setup.loader
            || record.loader_version != setup.loader_version
        {
            instance_service::change_version(
                state,
                iid,
                Some(setup.game_version.clone()),
                Some(setup.loader.clone()),
                Some(setup.loader_version.clone()),
            )
            .await?;
        }
        Ok::<(), SourceError>(())
    }
    .await;

    if let Err(err) = result {
        for path in plan.remove.iter().chain(&plan.write) {
            let target = data_dir.join(disk_path(path, &level));
            if existed.contains(path) {
                let _ = copy_file(&backup.join(path), &target).await;
            } else {
                let _ = tokio::fs::remove_file(&target).await;
            }
        }
        let _ = tokio::fs::remove_dir_all(&backup).await;
        return Err(err);
    }

    let source = match row.source()? {
        ServerSource::Modrinth { project_id, .. } => ServerSource::Modrinth {
            project_id,
            version_id: version.to_string(),
        },
        other => other,
    };
    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM server_source_files WHERE instance_id = ?")
        .bind(&instance_id)
        .execute(&mut *tx)
        .await?;
    for file in &setup.files {
        sqlx::query("INSERT INTO server_source_files (instance_id, path, kind, sha1, client_only, project_id, version_id) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&instance_id)
            .bind(&file.path)
            .bind(kind_str(file.kind))
            .bind(&file.sha1)
            .bind(file.client_only as i64)
            .bind(&file.project_id)
            .bind(&file.version_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE server_sources SET source_json = ?, installed_version = ?, staged_version = NULL, state = ?, error = NULL, restart_requested = 0, updated_at = ? WHERE instance_id = ?")
        .bind(serde_json::to_string(&source).map_err(|e| SourceError::Invalid(e.to_string()))?)
        .bind(version)
        .bind(SourceApplyState::UpToDate.as_str())
        .bind(now())
        .bind(&instance_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    // Also drops versions that were superseded before they applied, keeping a newer upload.
    let desired = load_row(state, &instance_id)
        .await?
        .map(|row| staging_dir(state, &instance_id, &row.desired_version));
    if let Ok(mut entries) =
        tokio::fs::read_dir(staging_root(state, &instance_id)).await
    {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if desired.as_ref() != Some(&entry.path()) || entry.path() == staged {
                let _ = tokio::fs::remove_dir_all(entry.path()).await;
            }
        }
    }
    broadcast(state, &instance_id).await?;
    Ok(true)
}

/// What an apply changes on disk. Paths are setup paths (see [`disk_path`]).
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ApplyPlan {
    /// Files the previous version provided that the next one dropped.
    pub remove: Vec<String>,
    /// Files that are missing or differ from the next version.
    pub write: Vec<String>,
}

/// Plan an apply. Only paths the previous version provided can be removed, so server-added
/// files are never touched. Client-only files are never written.
pub(crate) fn plan(
    previous: &[SetupFile],
    next: &[SetupFile],
    on_disk: &HashMap<String, String>,
) -> ApplyPlan {
    let installed: HashSet<&str> = next
        .iter()
        .filter(|f| !f.client_only)
        .map(|f| f.path.as_str())
        .collect();
    let mut remove: Vec<String> = previous
        .iter()
        .filter(|f| !f.client_only && !installed.contains(f.path.as_str()))
        .map(|f| f.path.clone())
        .collect();
    let mut write: Vec<String> = next
        .iter()
        .filter(|f| !f.client_only)
        .filter(|f| on_disk.get(&f.path) != Some(&f.sha1))
        .map(|f| f.path.clone())
        .collect();
    remove.sort();
    write.sort();
    ApplyPlan { remove, write }
}

/// Merge tracked source files with what is on disk. `disk` holds setup paths such as
/// `mods/a.jar`, `mods/b.jar.disabled`, or `datapacks/c.zip`.
pub(crate) fn content_items(
    tracked: &[SetupFile],
    disk: &[(ContentKind, String)],
) -> Vec<ServerContentItem> {
    let on_disk: HashSet<&str> = disk.iter().map(|(_, p)| p.as_str()).collect();
    let tracked_paths: HashSet<&str> =
        tracked.iter().map(|f| f.path.as_str()).collect();
    let mut items: Vec<ServerContentItem> = tracked
        .iter()
        .filter(|f| f.kind != ContentKind::Other)
        .map(|f| ServerContentItem {
            kind: f.kind,
            filename: file_name(&f.path),
            origin: ContentOrigin::Source,
            client_only: f.client_only,
            disabled: !f.client_only
                && !on_disk.contains(f.path.as_str())
                && on_disk.contains(format!("{}.disabled", f.path).as_str()),
            project_id: f.project_id.clone(),
            version_id: f.version_id.clone(),
            name: None,
            icon_url: None,
        })
        .collect();
    for (kind, path) in disk {
        let canonical = path.strip_suffix(".disabled").unwrap_or(path);
        if tracked_paths.contains(canonical) {
            continue;
        }
        items.push(ServerContentItem {
            kind: *kind,
            filename: file_name(path),
            origin: ContentOrigin::Server,
            client_only: false,
            disabled: path.ends_with(".disabled"),
            project_id: None,
            version_id: None,
            name: None,
            icon_url: None,
        });
    }
    items.sort_by(|a, b| a.filename.cmp(&b.filename));
    items
}

// ── Adapters ─────────────────────────────────────────────────────────────────

/// Download and verify a source version into `dir/files`, returning its setup.
async fn stage(
    state: &Arc<AppState>,
    row: &SourceRow,
    version: &str,
    dir: &Path,
) -> Result<Setup, SourceError> {
    let files = dir.join("files");
    match row.source()? {
        ServerSource::Modrinth { .. } => {
            let modrinth = ModrinthClient::new(state.http.clone());
            let pack = modrinth.get_version(version).await?;
            let file = pack
                .files
                .iter()
                .find(|f| f.primary)
                .or_else(|| pack.files.first())
                .ok_or_else(|| {
                    SourceError::Invalid("version has no downloadable file".into())
                })?;
            validate_download_url(&file.url)?;
            let bytes = download(state, &file.url).await?;
            verify(&file.filename, &sha1_hex(&bytes), file.hashes.sha1.as_deref())?;
            let archive = dir.join("pack.mrpack");
            tokio::fs::write(&archive, &bytes).await?;
            stage_mrpack(state, &archive, &files).await
        }
        ServerSource::Mrpack { .. } | ServerSource::Instance { .. } => {
            stage_mrpack(state, &dir.join("pack.mrpack"), &files).await
        }
    }
}

/// Link the server to an uploaded `.mrpack`, or update it when it is already linked. The
/// archive is stored before the worker stages it, so a Core restart can still finish the
/// install. `instance` is the id and name of the App instance the pack was exported from.
pub async fn link_mrpack(
    state: &Arc<AppState>,
    instance_id: &str,
    archive: &Path,
    instance: Option<(String, String)>,
) -> Result<ServerSourceStatus, SourceError> {
    let index = extract_metadata(archive).await?;
    // The version is the archive's hash, so every different upload is a new version even when
    // the pack's own version string did not change.
    let version = sha1_hex(&tokio::fs::read(archive).await?);
    let dir = staging_dir(state, instance_id, &version);
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::copy(archive, dir.join("pack.mrpack")).await?;
    let source = match instance {
        Some((id, name)) => ServerSource::Instance {
            instance_id: id,
            name,
        },
        None => ServerSource::Mrpack {
            filename: "modpack.mrpack".to_string(),
            name: index.name,
        },
    };
    match load_row(state, instance_id).await? {
        Some(_) => {
            sqlx::query("UPDATE server_sources SET source_json = ? WHERE instance_id = ?")
                .bind(serde_json::to_string(&source).map_err(|e| SourceError::Invalid(e.to_string()))?)
                .bind(instance_id)
                .execute(&state.pool)
                .await?;
            request_source_update(state, instance_id, &version).await
        }
        None => link(state, instance_id, source, &version).await,
    }
}

async fn stage_mrpack(
    state: &Arc<AppState>,
    archive: &Path,
    files_dir: &Path,
) -> Result<Setup, SourceError> {
    let index = extract_metadata(archive).await?;
    let game_version = index
        .dependencies
        .get("minecraft")
        .filter(|v| index.game == "minecraft" && !v.trim().is_empty())
        .cloned()
        .ok_or_else(|| {
            SourceError::Invalid("Modpack must specify a Minecraft version".into())
        })?;
    let (loader, loader_version) = mrpack_loader(&index.dependencies);
    // A pack's own `env` is often missing or says "required" for client mods, so a mod is also
    // left out when its Modrinth project says servers are unsupported, as for a mod added by hand.
    let project_ids: Vec<String> = index
        .files
        .iter()
        .filter_map(|file| {
            file.project_id.clone().or_else(|| {
                file.downloads
                    .first()
                    .and_then(|url| modrinth_ids(url))
                    .map(|ids| ids.0)
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let server_unsupported: BTreeSet<String> =
        match ModrinthClient::new(state.http.clone()).get_projects(&project_ids).await {
            Ok(projects) => projects
                .into_iter()
                .filter(|p| p.server_side.as_deref() == Some("unsupported"))
                .map(|p| p.id)
                .collect(),
            Err(error) => {
                warn!("could not check which pack mods are client-only: {error}");
                BTreeSet::new()
            }
        };
    let mut files = BTreeMap::new();
    for file in &index.files {
        let path = clean_path(&file.path)?;
        let Some(kind) = content_kind(&path) else {
            continue;
        };
        let ids = file.downloads.first().and_then(|url| modrinth_ids(url));
        let project_id = file.project_id.clone().or(ids.clone().map(|i| i.0));
        let client_only = matches!(
            file.env.as_ref().map(|env| &env.server),
            Some(EnvType::Unsupported)
        ) || project_id
            .as_ref()
            .is_some_and(|id| server_unsupported.contains(id));
        let mut sha1 = file.hashes.sha1.clone().unwrap_or_default();
        if !client_only {
            let url = file.downloads.first().ok_or_else(|| {
                SourceError::Invalid(format!("{} has no download", file.path))
            })?;
            validate_download_url(url)?;
            let bytes = download(state, url).await?;
            let actual = sha1_hex(&bytes);
            verify(&path, &actual, file.hashes.sha1.as_deref())?;
            write_file(&files_dir.join(&path), &bytes).await?;
            sha1 = actual;
        }
        files.insert(
            path.clone(),
            SetupFile {
                path,
                kind,
                sha1,
                client_only,
                project_id,
                version_id: file.version_id.clone().or(ids.map(|i| i.1)),
            },
        );
    }
    for (path, bytes) in read_overrides(archive).await? {
        let Some(kind) = content_kind(&path) else {
            continue;
        };
        write_file(&files_dir.join(&path), &bytes).await?;
        files.insert(
            path.clone(),
            SetupFile {
                path,
                kind,
                sha1: sha1_hex(&bytes),
                client_only: false,
                project_id: None,
                version_id: None,
            },
        );
    }
    Ok(Setup {
        name: index.name,
        version_number: index.version_id,
        game_version,
        loader,
        loader_version,
        files: files.into_values().collect(),
    })
}

async fn download(
    state: &Arc<AppState>,
    url: &str,
) -> Result<bytes::Bytes, SourceError> {
    Ok(state
        .http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

async fn read_overrides(
    archive: &Path,
) -> Result<Vec<(String, Vec<u8>)>, SourceError> {
    let archive = archive.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)
            .map_err(|e| SourceError::Invalid(e.to_string()))?;
        let mut out = Vec::new();
        // server-overrides come last so they win over overrides for the same path.
        for prefix in ["overrides/", "server-overrides/"] {
            for i in 0..zip.len() {
                let mut entry = zip
                    .by_index(i)
                    .map_err(|e| SourceError::Invalid(e.to_string()))?;
                let name = entry.name().replace('\\', "/");
                let Some(rel) = name.strip_prefix(prefix) else {
                    continue;
                };
                if rel.is_empty() || entry.is_dir() {
                    continue;
                }
                let rel = clean_path(rel)?;
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                out.push((rel, bytes));
            }
        }
        Ok(out)
    })
    .await
    .map_err(|e| SourceError::Invalid(e.to_string()))?
}

/// Project and version ids from a Modrinth CDN url
/// (`https://cdn.modrinth.com/data/<project>/versions/<version>/<file>`). Standard packs carry
/// no ids, and the content list needs them to show names and icons.
pub(crate) fn modrinth_ids(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("https://cdn.modrinth.com/data/")?;
    let mut parts = rest.split('/');
    let project = parts.next()?;
    let version = (parts.next()? == "versions").then(|| parts.next())??;
    Some((project.to_string(), version.to_string()))
}

/// Which content a setup path is, or `None` for files a server never installs.
pub(crate) fn content_kind(path: &str) -> Option<ContentKind> {
    if path.starts_with("resourcepacks/")
        || path.starts_with("shaderpacks/")
        || path.ends_with(".disabled")
    {
        None
    } else if path.starts_with("mods/") && path.ends_with(".jar") {
        Some(ContentKind::Mod)
    } else if path.starts_with("datapacks/") {
        Some(ContentKind::Datapack)
    } else {
        Some(ContentKind::Other)
    }
}

/// Setup paths put datapacks under `datapacks/`; on disk they live in the world folder.
fn disk_path(path: &str, level: &str) -> String {
    if path.starts_with("datapacks/") {
        format!("{level}/{path}")
    } else {
        path.to_string()
    }
}

async fn level_name(data_dir: &Path) -> String {
    read_properties(data_dir)
        .await
        .ok()
        .and_then(|props| props.get("level-name").cloned())
        .filter(|name| clean_path(name).is_ok())
        .unwrap_or_else(|| "world".to_string())
}

fn clean_path(path: &str) -> Result<String, SourceError> {
    let path = path.replace('\\', "/");
    let bad = path.is_empty()
        || path.starts_with('/')
        || path.contains(':')
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..");
    if bad {
        return Err(SourceError::Invalid(format!("unsafe path in source: {path}")));
    }
    Ok(path)
}

fn mrpack_loader(deps: &HashMap<String, String>) -> (ModLoader, Option<String>) {
    for (key, loader) in [
        ("fabric-loader", ModLoader::Fabric),
        ("quilt-loader", ModLoader::Quilt),
        ("neoforge", ModLoader::NeoForge),
        ("forge", ModLoader::Forge),
    ] {
        if let Some(version) = deps.get(key) {
            return (loader, Some(version.clone()));
        }
    }
    (ModLoader::Vanilla, None)
}

fn verify(
    name: &str,
    actual: &str,
    expected: Option<&str>,
) -> Result<(), SourceError> {
    match expected {
        Some(expected) if !expected.eq_ignore_ascii_case(actual) => {
            Err(SourceError::Invalid(format!("hash mismatch for {name}")))
        }
        _ => Ok(()),
    }
}

fn sha1_hex(bytes: &[u8]) -> String {
    hex::encode(sha1::Sha1::digest(bytes))
}

fn file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn kind_str(kind: ContentKind) -> &'static str {
    match kind {
        ContentKind::Mod => "mod",
        ContentKind::Datapack => "datapack",
        ContentKind::Other => "other",
    }
}

async fn write_file(path: &Path, bytes: &[u8]) -> Result<(), SourceError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, bytes).await?;
    Ok(())
}

async fn copy_file(from: &Path, to: &Path) -> Result<(), SourceError> {
    if let Some(parent) = to.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::copy(from, to).await?;
    Ok(())
}

fn staging_root(state: &AppState, instance_id: &str) -> PathBuf {
    state.config.data_dir.join("source_staging").join(instance_id)
}

fn staging_dir(state: &AppState, instance_id: &str, version: &str) -> PathBuf {
    let safe: String = version
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '_' })
        .collect();
    staging_root(state, instance_id).join(safe)
}

async fn instance_record(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<crate::domain::instance::InstanceRecord, SourceError> {
    let iid: InstanceId = instance_id.parse().map_err(|_| SourceError::NotFound)?;
    state.instance_store.get(&iid).await.map_err(|e| match e {
        StoreError::NotFound(_) => SourceError::NotFound,
        other => SourceError::Store(other),
    })
}

async fn load_row(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<Option<SourceRow>, SourceError> {
    Ok(
        sqlx::query_as("SELECT * FROM server_sources WHERE instance_id = ?")
            .bind(instance_id)
            .fetch_optional(&state.pool)
            .await?,
    )
}

async fn tracked_files(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<Vec<SetupFile>, SourceError> {
    let rows: Vec<(String, String, String, i64, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT path, kind, sha1, client_only, project_id, version_id FROM server_source_files WHERE instance_id = ?")
            .bind(instance_id)
            .fetch_all(&state.pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(path, kind, sha1, client_only, project_id, version_id)| SetupFile {
            path,
            kind: match kind.as_str() {
                "mod" => ContentKind::Mod,
                "datapack" => ContentKind::Datapack,
                _ => ContentKind::Other,
            },
            sha1,
            client_only: client_only != 0,
            project_id,
            version_id,
        })
        .collect())
}

async fn set_state(
    state: &Arc<AppState>,
    instance_id: &str,
    value: SourceApplyState,
) -> Result<(), SourceError> {
    sqlx::query("UPDATE server_sources SET state = ?, updated_at = ? WHERE instance_id = ?")
        .bind(value.as_str())
        .bind(now())
        .bind(instance_id)
        .execute(&state.pool)
        .await?;
    broadcast(state, instance_id).await?;
    Ok(())
}

async fn set_failed(
    state: &Arc<AppState>,
    instance_id: &str,
    err: &SourceError,
) -> Result<(), SourceError> {
    sqlx::query("UPDATE server_sources SET state = 'failed', error = ?, updated_at = ? WHERE instance_id = ?")
        .bind(err.to_string())
        .bind(now())
        .bind(instance_id)
        .execute(&state.pool)
        .await?;
    broadcast(state, instance_id).await?;
    Ok(())
}

async fn broadcast(
    state: &Arc<AppState>,
    instance_id: &str,
) -> Result<Option<ServerSourceStatus>, SourceError> {
    let status = get_status(state, instance_id).await?;
    if let Ok(iid) = instance_id.parse::<InstanceId>() {
        state.broadcaster.send(Event::SourceStatusChanged {
            instance_id: iid,
            status: status.clone(),
        });
    }
    Ok(status)
}

fn progress(
    state: &Arc<AppState>,
    instance_id: &str,
    version: &str,
    phase: SourceApplyPhase,
) {
    if let Ok(iid) = instance_id.parse::<InstanceId>() {
        state.broadcaster.send(Event::SourceApplyProgress {
            instance_id: iid,
            version: version.to_string(),
            phase,
            progress: None,
        });
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, sha1: &str, client_only: bool) -> SetupFile {
        SetupFile {
            path: path.into(),
            kind: content_kind(path).unwrap(),
            sha1: sha1.into(),
            client_only,
            project_id: None,
            version_id: None,
        }
    }

    #[test]
    fn update_removes_dropped_source_files_and_rewrites_changed_ones() {
        let previous = [
            file("mods/kept.jar", "a", false),
            file("mods/dropped.jar", "b", false),
            file("datapacks/old.zip", "c", false),
        ];
        let next = [
            file("mods/kept.jar", "a2", false),
            file("mods/new.jar", "d", false),
        ];
        let on_disk = HashMap::from([("mods/kept.jar".to_string(), "a".to_string())]);
        let plan = plan(&previous, &next, &on_disk);
        assert_eq!(plan.remove, ["datapacks/old.zip", "mods/dropped.jar"]);
        assert_eq!(plan.write, ["mods/kept.jar", "mods/new.jar"]);
    }

    #[test]
    fn server_added_files_are_never_planned() {
        let previous = [file("mods/source.jar", "a", false)];
        let next = [file("mods/source.jar", "a", false)];
        let on_disk = HashMap::from([("mods/source.jar".to_string(), "a".to_string())]);
        // `mods/server.jar` exists on disk but was never provided by the source.
        assert_eq!(plan(&previous, &next, &on_disk), ApplyPlan::default());
    }

    #[test]
    fn client_only_files_are_skipped_but_listed() {
        let next = [
            file("mods/server.jar", "a", false),
            file("mods/client.jar", "b", true),
        ];
        let plan = plan(&[], &next, &HashMap::new());
        assert_eq!(plan.write, ["mods/server.jar"]);

        let disk = [
            (ContentKind::Mod, "mods/server.jar".to_string()),
            (ContentKind::Mod, "mods/local.jar.disabled".to_string()),
        ];
        let items = content_items(&next, &disk);
        let client = items.iter().find(|i| i.filename == "client.jar").unwrap();
        assert!(client.client_only);
        assert_eq!(client.origin, ContentOrigin::Source);
        let local = items.iter().find(|i| i.filename == "local.jar.disabled").unwrap();
        assert_eq!(local.origin, ContentOrigin::Server);
        assert!(local.disabled);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn reads_ids_from_modrinth_cdn_urls() {
        assert_eq!(
            modrinth_ids("https://cdn.modrinth.com/data/AANobbMI/versions/abc123/sodium.jar"),
            Some(("AANobbMI".into(), "abc123".into()))
        );
        assert_eq!(modrinth_ids("https://example.com/data/a/versions/b/c.jar"), None);
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert!(clean_path("../escape.jar").is_err());
        assert!(clean_path("mods/../../x").is_err());
        assert!(clean_path("C:/x").is_err());
        assert_eq!(clean_path("mods\\a.jar").unwrap(), "mods/a.jar");
        assert_eq!(disk_path("datapacks/a.zip", "world"), "world/datapacks/a.zip");
    }
}
