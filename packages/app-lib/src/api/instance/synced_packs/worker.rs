use super::super::synced_options::{
    get_global_options, instance_dir, instance_is_running, sha1_bytes,
};
use super::PackLibrary;
use super::storage::{read_library, write_library};
use crate::state::instances::adapters::{filesystem, sqlite::content_rows};
use crate::state::{InstanceMetadata, State};
use parking_lot::Mutex;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::{MutexGuard, oneshot};
use tokio::time::Instant;
use tracing_error::InstrumentError;

#[derive(Default)]
struct Queue {
    instances: BTreeSet<String>,
    waiters: BTreeMap<String, Vec<oneshot::Sender<crate::Result<()>>>>,
    retry_at: BTreeMap<String, Instant>,
    all: bool,
    running: bool,
}

#[derive(Default)]
pub(crate) struct PackSyncWorker {
    queue: Mutex<Queue>,
    pub(super) revision: AtomicU64,
}

pub(crate) fn queue_reconciliation(instance_id: &str) {
    if let Some(state) = State::get_if_initialized() {
        state
            .pack_sync_worker
            .queue
            .lock()
            .instances
            .insert(instance_id.to_owned());
        start(state);
    }
}

pub(super) fn queue_all() {
    if let Some(state) = State::get_if_initialized() {
        state.pack_sync_worker.queue.lock().all = true;
        start(state);
    }
}

pub(crate) async fn flush(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let (sender, receiver) = oneshot::channel();
    {
        let mut queue = state.pack_sync_worker.queue.lock();
        if let Some(retry_at) = queue.retry_at.get(instance_id) {
            let remaining = retry_at.saturating_duration_since(Instant::now());
            if !remaining.is_zero() {
                return Err(crate::ErrorKind::Ratelimited {
                    retry_in_seconds: remaining.as_secs()
                        + u64::from(remaining.subsec_nanos() != 0),
                }
                .into());
            }
        }
        queue.instances.insert(instance_id.to_owned());
        queue
            .waiters
            .entry(instance_id.to_owned())
            .or_default()
            .push(sender);
    }
    start(state);
    receiver.await.map_err(|_| {
        crate::ErrorKind::OtherError(
            "Pack sync worker stopped before completing the request."
                .to_owned(),
        )
    })?
}

fn start(state: Arc<State>) {
    {
        let mut queue = state.pack_sync_worker.queue.lock();
        if queue.running {
            return;
        }
        queue.running = true;
    }
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let all =
                std::mem::take(&mut state.pack_sync_worker.queue.lock().all);
            if all {
                match crate::state::list_instances(&state.pool).await {
                    Ok(instances) => {
                        state.pack_sync_worker.queue.lock().instances.extend(
                            instances
                                .into_iter()
                                .map(|metadata| metadata.instance.id),
                        );
                    }
                    Err(error) => {
                        tracing::warn!(
                            "Could not enumerate pack sync targets: {error}"
                        );
                        state.pack_sync_worker.queue.lock().all = true;
                        tokio::time::sleep(std::time::Duration::from_secs(1))
                            .await;
                    }
                }
            }
            let (instance_id, mut pending) = {
                let mut queue = state.pack_sync_worker.queue.lock();
                let next = queue
                    .waiters
                    .keys()
                    .chain(queue.instances.iter())
                    .find(|id| !queue.retry_at.contains_key(*id))
                    .cloned();
                let Some(instance_id) = next else {
                    if queue.all {
                        continue;
                    }
                    queue.running = false;
                    break;
                };
                queue.instances.remove(&instance_id);
                let pending =
                    queue.waiters.remove(&instance_id).unwrap_or_default();
                (instance_id, pending)
            };
            let result =
                super::reconciliation::run_queued(&instance_id, &state).await;
            if let Err(error) = &result {
                match error.raw.as_ref() {
                    crate::ErrorKind::PackSyncChanged => {
                        let mut queue = state.pack_sync_worker.queue.lock();
                        queue.instances.insert(instance_id.clone());
                        if !pending.is_empty() {
                            queue
                                .waiters
                                .entry(instance_id)
                                .or_default()
                                .extend(pending);
                        }
                        continue;
                    }
                    crate::ErrorKind::Ratelimited { retry_in_seconds } => {
                        let retry_at = Instant::now()
                            + Duration::from_secs((*retry_in_seconds).max(1));
                        {
                            let mut queue = state.pack_sync_worker.queue.lock();
                            queue.instances.insert(instance_id.clone());
                            queue
                                .retry_at
                                .insert(instance_id.clone(), retry_at);
                            pending.extend(
                                queue
                                    .waiters
                                    .remove(&instance_id)
                                    .unwrap_or_default(),
                            );
                        }
                        let state = state.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep_until(retry_at).await;
                            state
                                .pack_sync_worker
                                .queue
                                .lock()
                                .retry_at
                                .remove(&instance_id);
                            start(state);
                        });
                    }
                    _ => tracing::warn!(
                        "Could not reconcile synced packs for {instance_id}: {error}"
                    ),
                }
            }
            for sender in pending {
                let result =
                    result.as_ref().map(|_| ()).map_err(|error| crate::Error {
                        raw: error.raw.clone(),
                        source: error.raw.clone().in_current_span(),
                    });
                let _ = sender.send(result);
            }
        }
    });
}

/// Releases the canonical-state lock for preparation and rejects outdated results.
pub(super) struct Preparation<'a> {
    state: &'a State,
    guard: Option<MutexGuard<'a, ()>>,
    metadata: &'a InstanceMetadata,
}

impl<'a> Preparation<'a> {
    pub(super) async fn new(
        state: &'a State,
        metadata: &'a InstanceMetadata,
    ) -> Self {
        Self {
            state,
            metadata,
            guard: Some(state.lock_synced_options().await),
        }
    }

    pub(super) async fn run<T>(
        &mut self,
        library: &PackLibrary,
        work: impl Future<Output = crate::Result<T>>,
    ) -> crate::Result<T> {
        write_library(library, self.state).await?;
        let revision =
            self.state.pack_sync_worker.revision.load(Ordering::Acquire);
        self.validate_metadata().await?;
        let before = fingerprint(self.metadata, self.state).await?;
        self.guard.take();
        let result = work.await;
        self.guard = Some(self.state.lock_synced_options().await);
        self.validate_metadata().await?;
        if self.state.pack_sync_worker.revision.load(Ordering::Acquire)
            != revision
            || fingerprint(self.metadata, self.state).await? != before
        {
            return Err(crate::ErrorKind::PackSyncChanged.into());
        }
        result
    }

    async fn validate_metadata(&self) -> crate::Result<()> {
        let current = crate::state::get_instance(
            &self.metadata.instance.id,
            &self.state.pool,
        )
        .await?;
        if current.as_ref().is_none_or(|current| {
            metadata_key(current) != metadata_key(self.metadata)
        }) {
            return Err(crate::ErrorKind::PackSyncChanged.into());
        }
        Ok(())
    }

    pub(super) async fn library(&self) -> crate::Result<PackLibrary> {
        read_library(self.state).await
    }
}

fn metadata_key(metadata: &InstanceMetadata) -> serde_json::Value {
    let content_set = &metadata.applied_content_set;
    serde_json::json!([
        metadata.instance.path,
        metadata.instance.install_stage,
        content_set.id,
        content_set.source_kind,
        content_set.game_version,
        content_set.loader,
        content_set.loader_version,
        metadata.synced_options,
        super::super::projects::ensure_metadata_content_unlocked(metadata)
            .is_ok(),
    ])
}

async fn fingerprint(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Vec<(String, String)>> {
    let directory = state.directories.instances_dir();
    let instance_path = metadata.instance.path.clone();
    let mut files = tokio::task::spawn_blocking(move || {
        filesystem::scan_content_files(&directory, &instance_path)
    })
    .await??
    .into_iter()
    .map(|file| (file.relative_path, file.hash_cache_key))
    .collect::<Vec<_>>();
    let options = instance_dir(metadata, state).join("options.txt");
    match tokio::fs::read(options).await {
        Ok(bytes) => files.push(("options.txt".to_owned(), sha1_bytes(&bytes))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    files.push((
        "global-options".to_owned(),
        serde_json::to_string(&get_global_options().await?)?,
    ));
    files.push((
        "running".to_owned(),
        instance_is_running(metadata, state).await?.to_string(),
    ));
    files.push((
        "pending".to_owned(),
        super::super::synced_options::pending::contains(
            &metadata.instance.id,
            crate::state::SyncedOption::ResourcePacks,
            state,
        )
        .await?
        .to_string(),
    ));
    for id in content_rows::get_locked_instance_file_ids(
        &metadata.instance.id,
        &state.pool,
    )
    .await?
    {
        files.push(("locked-file".to_owned(), id));
    }
    let entries = content_rows::get_content_entries(
        &metadata.applied_content_set.id,
        &state.pool,
    )
    .await?;
    for entry in entries {
        files.push((
            format!("source:{}", entry.id),
            format!("{:?}:{}", entry.file_id, entry.source_kind.as_str()),
        ));
    }
    files.sort();
    Ok(files)
}
