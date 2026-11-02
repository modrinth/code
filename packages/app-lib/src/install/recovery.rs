use super::events::emit_install_job;
use super::model::{
    InstallCleanup, InstallErrorView, InstallInterruptReason,
    InstallJobDisplay, InstallJobEventKind, InstallJobState, InstallJobStatus,
    InstallPhaseDetails, InstallPhaseId, InstallRequest, InstallTarget,
};
use super::store;
use crate::ErrorKind;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::content_store::ContentStore;
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::state::{
    ContentEntry, ContentSetRemoteRef, ContentSetRemoteRefType,
    ContentSetSyncProvider, ContentSetSyncState, DirectoryInfo, InstanceFile,
    InstanceMetadata, State,
};
use crate::util::fetch::IoSemaphore;
use anyhow::{Context, bail};
use async_walkdir::WalkDir;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::future::Future;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SHARED_INSTANCE_ROLLBACK_FILE: &str = "rollback.json";
const SHARED_INSTANCE_ROLLBACK_INSTANCE_DIR: &str = "instance";

#[derive(Deserialize, Serialize)]
struct SharedInstanceUpdateRollback {
    files: Vec<InstanceFile>,
    entries: Vec<ContentEntry>,
    #[serde(default)]
    bindings: Vec<crate::state::content_store::InstanceFileStorage>,
    #[serde(default)]
    missing_file_ids: std::collections::HashSet<String>,
    #[serde(default)]
    copied_file_ids: std::collections::HashSet<String>,
    #[serde(default)]
    scoped_paths: Option<Vec<String>>,
}

/// With `scoped_paths`, only those instance-relative files and folders are backed
/// up. A path that does not exist yet is removed again on rollback.
pub(super) async fn prepare_instance_update_backup(
    job_id: Uuid,
    metadata: &InstanceMetadata,
    scoped_paths: Option<Vec<String>>,
    state: &State,
) -> crate::Result<PathBuf> {
    let _lease = state.content_store.lease().await;
    let _content_lock = state
        .instance_locks
        .lock_content(&metadata.instance.id)
        .await;
    let _store_lock = state.content_store.files_lock.lock().await;
    if crate::state::instance_has_running_process(&metadata.instance.id, state)
        .await?
    {
        bail!(ErrorKind::InputError(
            "stop this instance before backing it up for an update".to_string(),
        ));
    }
    let owner = job_id.to_string();
    let staging_dir = instance_update_backup_dir(job_id, &state.directories);
    if tokio::fs::try_exists(&staging_dir).await? {
        crate::util::io::remove_dir_all(&staging_dir).await?;
    }
    crate::util::io::create_dir_all(&staging_dir).await?;

    let result = async {
		crate::state::instances::commands::reconcile_instance_renames(
			&metadata.instance,
			state,
		)
		.await?;
        let mut files = content_rows::get_instance_files(
            &metadata.instance.id,
            &state.pool,
        )
        .await?;
        let entries = content_rows::get_content_entries(
            &metadata.applied_content_set.id,
            &state.pool,
        )
        .await?;
        let bindings = crate::state::content_store::instance_storage(
            &state.pool,
            &metadata.instance.id,
        )
        .await?;
		let mut missing_file_ids = std::collections::HashSet::new();
		let mut copied_file_ids = std::collections::HashSet::new();
		for binding in &bindings {
			let file = files
				.iter_mut()
				.find(|file| file.id == binding.file_id)
				.ok_or(ErrorKind::InputError(
					"backup content reference has no file record".to_string(),
				))?;
			let file_status = state.content_store
				.check_instance_file(&metadata.instance, file, binding).await?;
			file.missing = file_status == crate::state::content_store::InstanceFileStatus::Missing;
			if file.missing {
				missing_file_ids.insert(file.id.clone());
				continue;
			}
			let content = state.content_store.file_content(file).await?;
			if file_status != crate::state::content_store::InstanceFileStatus::Healthy
			{
				bail!(ErrorKind::InputError(format!(
					"restore or repair {} before updating this instance; its current content cannot be backed up safely",
					file.relative_path,
				)));
			}
			if !matches!(content, crate::state::content_store::FileContent::Stored { .. }) {
				copied_file_ids.insert(file.id.clone());
			}
		}
        let skipped: std::collections::HashSet<String> = files
            .iter()
            .filter(|file| {
                bindings.iter().any(|binding| binding.file_id == file.id)
					&& !copied_file_ids.contains(&file.id)
            })
            .map(crate::state::content_store::content_file_path)
            .collect();
        let retained = bindings
            .iter()
            .map(|binding| binding.blob_sha512.clone())
            .collect::<Vec<_>>();
        let scoped_paths = scoped_paths.map(outermost_paths);
        let instance_path = state
            .directories
            .instances_dir()
            .join(&metadata.instance.path);
        let backup_path =
            staging_dir.join(SHARED_INSTANCE_ROLLBACK_INSTANCE_DIR);
        match &scoped_paths {
            Some(paths) => {
                for path in paths {
                    let prefix = format!("{path}/");
                    let skipped = skipped
                        .iter()
                        .filter_map(|skipped| skipped.strip_prefix(&prefix))
                        .map(str::to_string)
                        .collect();
                    copy_path(
                        &instance_path.join(path),
                        &backup_path.join(path),
                        &skipped,
                        &state.io_semaphore,
                    )
                    .await?;
                }
            }
            None => {
                copy_directory(
                    &instance_path,
                    &backup_path,
                    &skipped,
                    &state.io_semaphore,
                )
                .await?;
            }
        }
        let snapshot = SharedInstanceUpdateRollback {
            files,
            entries,
            bindings,
			missing_file_ids,
			copied_file_ids,
            scoped_paths,
        };
        crate::util::io::write(
            staging_dir.join(SHARED_INSTANCE_ROLLBACK_FILE),
            serde_json::to_vec(&snapshot)?,
        )
        .await?;
        state
            .content_store
            .retain("rollback", &owner, &retained)
            .await?;

        Ok::<(), crate::Error>(())
    }
    .await;

    if result.is_err() {
        let _ = crate::util::io::remove_dir_all(&staging_dir).await;
        let _ = state.content_store.release("rollback", &owner).await;
    }
    result?;
    Ok(staging_dir)
}

fn instance_update_backup_dir(
    job_id: Uuid,
    directories: &DirectoryInfo,
) -> PathBuf {
    directories.install_backups_dir().join(job_id.to_string())
}

async fn recover_unrecorded_instance_update_backup(
    job: &mut store::InstallJobRecord,
    request: Option<&InstallRequest>,
    state: &State,
) -> crate::Result<()> {
    if job.state.paths.staging_dir.is_some()
        || !matches!(
            request,
            Some(
                InstallRequest::BulkUpdateContent { .. }
                    | InstallRequest::UpdateSharedInstance { .. }
                    | InstallRequest::InstallPackToExistingInstance { .. }
            )
        )
    {
        return Ok(());
    }
    let staging_dir = instance_update_backup_dir(job.id, &state.directories);
    if !tokio::fs::try_exists(&staging_dir).await? {
        return Ok(());
    }
    let snapshot = match crate::util::io::read(
        staging_dir.join(SHARED_INSTANCE_ROLLBACK_FILE),
    )
    .await
    {
        Ok(bytes) => {
            serde_json::from_slice::<SharedInstanceUpdateRollback>(&bytes).ok()
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if snapshot.is_some() {
        job.state.paths.staging_dir = Some(staging_dir);
    } else {
        crate::util::io::remove_dir_all(&staging_dir).await?;
        state
            .content_store
            .release("rollback", &job.id.to_string())
            .await?;
    }
    Ok(())
}

pub(super) async fn clear_staging_dir(
    job_state: &InstallJobState,
    content_store: &ContentStore,
) {
    let Some(staging_dir) = &job_state.paths.staging_dir else {
        return;
    };
    if let Err(error) = crate::util::io::remove_dir_all(staging_dir).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(
            path = %staging_dir.display(),
            "Failed to remove install rollback backup: {error}"
        );
        return;
    }
    if let Some(owner) = staging_dir.file_name().and_then(|name| name.to_str())
        && let Err(error) = content_store.release("rollback", owner).await
    {
        tracing::warn!(
            "Could not release rollback content references: {error}"
        );
    }
}

async fn restore_instance_update(
    staging_dir: &Path,
    rollback: &super::model::InstallRollbackState,
    state: &State,
) -> crate::Result<()> {
    let instance_id = &rollback.instance.instance.id;
    let _content_lock = state.instance_locks.lock_content(instance_id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    let _lease = state.content_store.lease().await;
    if crate::state::instance_has_running_process(instance_id, state).await? {
        bail!(ErrorKind::InputError(
            "stop this instance before restoring its content".to_string(),
        ));
    }
    state.content_store.recover(Some(instance_id)).await?;
    let snapshot = serde_json::from_slice::<SharedInstanceUpdateRollback>(
        &crate::util::io::read(staging_dir.join(SHARED_INSTANCE_ROLLBACK_FILE))
            .await?,
    )?;
    let instance_path = state
        .directories
        .instances_dir()
        .join(&rollback.instance.instance.path);
    let backup_path = staging_dir.join(SHARED_INSTANCE_ROLLBACK_INSTANCE_DIR);
    if !tokio::fs::symlink_metadata(&backup_path).await?.is_dir() {
        bail!(ErrorKind::InputError(
            "the instance backup is missing or is not a directory".to_string(),
        ));
    }
    for binding in &snapshot.bindings {
        if snapshot.missing_file_ids.contains(&binding.file_id) {
            continue;
        }
        if snapshot.copied_file_ids.contains(&binding.file_id) {
            let file = snapshot
                .files
                .iter()
                .find(|file| file.id == binding.file_id)
                .ok_or(ErrorKind::InputError(
                    "backup content reference has no file record".to_string(),
                ))?;
            let path = backup_path
                .join(crate::state::content_store::content_file_path(file));
            if crate::state::content_store::hash_file(&path).await?.sha512
                != binding.blob_sha512
            {
                bail!(ErrorKind::InputError(
                    "the instance backup contains changed content".to_string(),
                ));
            }
            continue;
        }
        if state
            .content_store
            .lookup(Some(&binding.blob_sha512), None)
            .await?
            .is_none()
        {
            bail!(ErrorKind::InputError(
                "repair the backup's shared content before restoring this instance"
                    .to_string(),
            ));
        }
    }
    match &snapshot.scoped_paths {
        Some(paths) => {
            for path in paths {
                let target = instance_path.join(path);
                remove_path(&target).await?;
                copy_path(
                    &backup_path.join(path),
                    &target,
                    &std::collections::HashSet::new(),
                    &state.io_semaphore,
                )
                .await?;
            }
        }
        None => {
            if tokio::fs::try_exists(&instance_path).await? {
                crate::util::io::remove_dir_all(&instance_path).await?;
            }
            copy_directory(
                &backup_path,
                &instance_path,
                &std::collections::HashSet::new(),
                &state.io_semaphore,
            )
            .await?;
        }
    }
    content_rows::restore_instance_content_snapshot(
        &rollback.instance.instance.id,
        &snapshot.files,
        &snapshot.entries,
        &state.pool,
    )
    .await?;
    state
        .content_store
        .restore_instance_files(
            &rollback.instance.instance,
            &snapshot.files,
            &snapshot.bindings,
            &snapshot.missing_file_ids,
            &snapshot.copied_file_ids,
        )
        .await?;
    restore_instance_metadata(&rollback.instance, &state.pool).await?;

    Ok(())
}

async fn restore_instance_metadata(
    metadata: &InstanceMetadata,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let content_set_id = metadata.applied_content_set.id.as_str();
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    instance_rows::update_instance(&metadata.instance, &mut tx).await?;
    content_rows::update_content_set(&metadata.applied_content_set, &mut tx)
        .await?;
    instance_rows::upsert_instance_link(
        &metadata.instance.id,
        &metadata.link,
        &mut tx,
    )
    .await?;
    instance_rows::set_shared_instance_attachment(
        &metadata.instance.id,
        metadata.shared_instance.as_ref(),
        &mut tx,
    )
    .await?;
    instance_rows::replace_instance_groups(
        &metadata.instance.id,
        &metadata.group_ids,
        &mut tx,
    )
    .await?;
    instance_rows::upsert_instance_launch_overrides(
        &metadata.launch_overrides,
        &mut tx,
    )
    .await?;
    content_rows::delete_content_set_remote_ref(
        content_set_id,
        ContentSetRemoteRefType::SharedContentSet,
        &mut tx,
    )
    .await?;
    content_rows::delete_content_set_sync_state(content_set_id, &mut tx)
        .await?;
    if let Some(attachment) = &metadata.shared_instance {
        content_rows::upsert_content_set_remote_ref(
            &ContentSetRemoteRef {
                content_set_id: content_set_id.to_string(),
                ref_type: ContentSetRemoteRefType::SharedContentSet,
                ref_id: attachment.id.clone(),
            },
            &mut tx,
        )
        .await?;
        content_rows::upsert_content_set_sync_state(
            &ContentSetSyncState {
                content_set_id: content_set_id.to_string(),
                provider: ContentSetSyncProvider::SharedInstance,
                applied_update_id: attachment
                    .applied_version
                    .map(|value| value.to_string()),
                latest_available_update_id: attachment
                    .latest_version
                    .map(|value| value.to_string()),
                checked_at: Some(Utc::now()),
                status: attachment.status,
            },
            &mut tx,
        )
        .await?;
    }
    tx.commit().await?;

    Ok(())
}

/// Drops duplicates and paths inside another listed folder.
fn outermost_paths(paths: Vec<String>) -> Vec<String> {
    let all = paths
        .iter()
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();
    let mut outermost = paths
        .iter()
        .filter(|path| {
            !path
                .match_indices('/')
                .any(|(index, _)| all.contains(&path[..index]))
        })
        .cloned()
        .collect::<Vec<_>>();
    outermost.sort();
    outermost.dedup();
    outermost
}

async fn symlink_metadata(
    path: &Path,
) -> crate::Result<Option<std::fs::Metadata>> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Removes a file, folder or symlink without following symlinks.
async fn remove_path(path: &Path) -> crate::Result<()> {
    match symlink_metadata(path).await? {
        Some(metadata) if metadata.is_file() => {
            crate::util::io::remove_file(path).await?;
        }
        Some(_) => crate::util::io::remove_dir_all(path).await?,
        None => {}
    }
    Ok(())
}

/// Copies a file, folder or symlink. Missing paths are skipped.
async fn copy_path(
    source: &Path,
    target: &Path,
    skipped: &std::collections::HashSet<String>,
    io_semaphore: &IoSemaphore,
) -> crate::Result<()> {
    match symlink_metadata(source).await? {
        Some(metadata) if metadata.is_symlink() => {
            copy_symlink(source, target).await
        }
        Some(metadata) if metadata.is_dir() => {
            copy_directory(source, target, skipped, io_semaphore).await
        }
        Some(_) => crate::util::fetch::copy(source, target, io_semaphore).await,
        None => Ok(()),
    }
}

async fn copy_directory(
    source: &Path,
    target: &Path,
    skipped: &std::collections::HashSet<String>,
    io_semaphore: &IoSemaphore,
) -> crate::Result<()> {
    crate::util::io::create_dir_all(target).await?;
    let mut walker = WalkDir::new(source);
    while let Some(entry) = walker.next().await {
        let entry = entry.context(crate::ErrorKind::FSError(
            "failed to read instance backup path".to_string(),
        ))?;
        let entry_path = entry.path();
        let relative_path = entry_path.strip_prefix(source)?;
        let relative = relative_path
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if skipped.contains(&relative) {
            continue;
        }
        let target_path = target.join(relative_path);
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            crate::util::io::create_dir_all(&target_path).await?;
        } else if file_type.is_file() {
            crate::util::fetch::copy(&entry_path, &target_path, io_semaphore)
                .await?;
        } else if file_type.is_symlink() {
            copy_symlink(&entry_path, &target_path).await?;
        }
    }

    Ok(())
}

async fn copy_symlink(source: &Path, target: &Path) -> crate::Result<()> {
    if let Some(parent) = target.parent() {
        crate::util::io::create_dir_all(parent).await?;
    }
    let link_target = tokio::fs::read_link(source).await?;
    let absolute = crate::state::content_store::normalize(
        &source
            .parent()
            .ok_or(ErrorKind::InputError("invalid backup symlink".to_string()))?
            .join(link_target),
    );
    let link_target = crate::state::content_store::relative_link(
        &absolute,
        target.parent().ok_or(ErrorKind::InputError(
            "invalid backup target".to_string(),
        ))?,
    );

    #[cfg(unix)]
    tokio::fs::symlink(link_target, target).await?;

    #[cfg(windows)]
    {
        let metadata = tokio::fs::metadata(source).await?;
        if metadata.is_dir() {
            tokio::fs::symlink_dir(link_target, target).await?;
        } else {
            tokio::fs::symlink_file(link_target, target).await?;
        }
    }

    Ok(())
}

pub async fn recover_interrupted_jobs(state: &State) -> crate::Result<()> {
    let jobs = store::list_interrupted_candidates(&state.pool).await?;

    for job in jobs {
        let job_id = job.id;
        if let Err(error) = recover_interrupted_job(job, state).await {
            tracing::error!(
                "Error recovering interrupted install job {job_id}: {error}"
            );
        }
    }

    if let Err(error) = recover_orphaned_install_stages(state).await {
        tracing::error!(
            "Could not recover orphaned installation states: {error}"
        );
    }
    Ok(())
}

async fn recover_orphaned_install_stages(state: &State) -> crate::Result<()> {
    let jobs = store::list_all(&state.pool).await?;
    for instance in instance_rows::list_instances(&state.pool).await? {
        let needs_recovery = jobs.iter().any(|job| {
            job.instance_id.as_deref() == Some(instance.id.as_str())
                && job.needs_recovery()
        });
        let stage = if needs_recovery {
            crate::state::InstanceInstallStage::MinecraftInstalling
        } else {
            if !matches!(
                instance.install_stage,
                crate::state::InstanceInstallStage::MinecraftInstalling
                    | crate::state::InstanceInstallStage::PackInstalling
            ) || jobs.iter().any(|job| {
                job.instance_id.as_deref() == Some(instance.id.as_str())
                    && !job.status.is_finished()
            }) || crate::state::instance_has_running_process(
                &instance.id,
                state,
            )
            .await?
            {
                continue;
            }
            crate::state::InstanceInstallStage::NotInstalled
        };
        if stage == instance.install_stage {
            continue;
        }
        crate::state::instances::commands::set_instance_install_stage(
            &instance.id,
            stage,
            &state.pool,
        )
        .await?;
        emit_instance(&instance.id, InstancePayloadType::Edited).await?;
    }
    Ok(())
}

fn recover_interrupted_job<'a>(
    job: store::InstallJobRecord,
    state: &'a State,
) -> impl Future<Output = crate::Result<()>> + Send + 'a {
    Box::pin(recover_interrupted_job_inner(job, state))
}

async fn recover_interrupted_job_inner(
    mut job: store::InstallJobRecord,
    state: &State,
) -> crate::Result<()> {
    let request = store::get_request(job.id, &state.pool).await?;
    recover_unrecorded_instance_update_backup(
        &mut job,
        request.as_ref(),
        state,
    )
    .await?;
    if job.state.display.is_none() {
        job.state.display = request
            .as_ref()
            .and_then(|request| display_from_request(request, &job.state));
    }

    if let Some(instance_id) = target_instance_id(&job.state.target)
        && instance_rows::get_instance_by_id(instance_id, &state.pool)
            .await?
            .is_none()
    {
        let canceled_phase = job.state.progress.phase;
        job.state.error = Some(InstallErrorView::from_message(
            "canceled",
            canceled_phase,
            "Install canceled because the instance was deleted",
        ));
        job.state.record_event(InstallJobEventKind::JobCanceled {
            phase: canceled_phase,
        });

        if let Some(record) = store::finish_active(
            job.id,
            InstallJobStatus::Canceled,
            &job.state,
            &state.pool,
        )
        .await?
        {
            store::dismiss(job.id, &state.pool).await?;
            clear_staging_dir(&job.state, &state.content_store).await;
            emit_install_job(&record.snapshot()).await?;
        }

        return Ok(());
    }

    let interrupted_phase = job.state.progress.phase;
    job.state.record_event(InstallJobEventKind::Interrupted {
        reason: InstallInterruptReason::AppClosed,
        phase: interrupted_phase,
    });
    job.state.progress.phase = InstallPhaseId::RollingBack;
    job.state.progress.progress = None;
    job.state.progress.details = InstallPhaseDetails::Empty;
    job.state.error = Some(InstallErrorView::from_message(
        "app_closed",
        interrupted_phase,
        "App closed while install was running",
    ));

    job.state
        .record_event(InstallJobEventKind::RollbackStarted {
            cleanup: job.state.cleanup.clone(),
        });
    let cleanup_succeeded = match apply_cleanup(&job.state, state).await {
        Ok(()) => {
            job.state
                .record_event(InstallJobEventKind::RollbackCompleted);
            clear_deleted_new_instance_id(&mut job.state);
            true
        }
        Err(error) => {
            tracing::error!(
                "Error cleaning up interrupted install job {}: {error}",
                job.id
            );
            job.state.rollback_error = Some(InstallErrorView::from_error(
                "rollback_error",
                InstallPhaseId::RollingBack,
                &error,
                None,
            ));
            job.state.record_event(InstallJobEventKind::RollbackFailed {
                message: error.to_string(),
            });
            false
        }
    };

    if let Some(record) = store::finish_active(
        job.id,
        InstallJobStatus::Interrupted,
        &job.state,
        &state.pool,
    )
    .await?
    {
        if cleanup_succeeded {
            clear_staging_dir(&job.state, &state.content_store).await;
        }
        emit_install_job(&record.snapshot()).await?;
    }

    Ok(())
}

fn target_instance_id(target: &InstallTarget) -> Option<&str> {
    match target {
        InstallTarget::NewInstance { instance_id } => instance_id.as_deref(),
        InstallTarget::ExistingInstance { instance_id } => Some(instance_id),
    }
}

fn clear_deleted_new_instance_id(job_state: &mut InstallJobState) {
    if matches!(job_state.cleanup, InstallCleanup::DeleteNewInstance { .. }) {
        job_state.target = InstallTarget::NewInstance { instance_id: None };
        job_state.cleanup =
            InstallCleanup::DeleteNewInstance { instance_id: None };
    }
}

fn display_from_request(
    request: &InstallRequest,
    state: &InstallJobState,
) -> Option<InstallJobDisplay> {
    match request {
        InstallRequest::CreateInstance { name, icon_path, .. } => {
            Some(InstallJobDisplay {
                title: name.clone(),
                icon: icon_path.clone(),
            })
        }
        InstallRequest::CreateModpackInstance { location, .. } => match location {
            crate::api::pack::install_from::CreatePackLocation::FromVersionId {
                title,
                icon_url,
                ..
            } => Some(InstallJobDisplay {
                title: title.clone(),
                icon: icon_url.clone(),
            }),
            crate::api::pack::install_from::CreatePackLocation::FromFile {
                ..
            } => None,
        },
        InstallRequest::CreateSharedInstance { data } => {
            Some(InstallJobDisplay {
                title: data.name.clone(),
                icon: data
                    .modpack
                    .as_ref()
                    .and_then(|modpack| modpack.icon_url.clone()),
            })
        }
        InstallRequest::ImportInstance {
            instance_folder, ..
        } => Some(InstallJobDisplay {
            title: instance_folder.clone(),
            icon: None,
        }),
        InstallRequest::DuplicateInstance { .. }
        | InstallRequest::InstallExistingInstance { .. }
        | InstallRequest::InstallPackToExistingInstance { .. }
		| InstallRequest::BulkUpdateContent { .. }
        | InstallRequest::UpdateSharedInstance { .. } => {
            state.rollback.as_ref().map(|rollback| InstallJobDisplay {
                title: rollback.instance.instance.name.clone(),
                icon: rollback.instance.instance.icon_path.clone(),
            })
        }
    }
}

pub async fn apply_cleanup(
    job_state: &InstallJobState,
    state: &State,
) -> crate::Result<()> {
    match &job_state.cleanup {
        InstallCleanup::DeleteNewInstance { instance_id } => {
            if let Some(instance_id) = instance_id {
                if crate::state::get_instance(instance_id, &state.pool)
                    .await?
                    .is_some()
                {
                    crate::state::remove_instance(instance_id, state).await?;
                }
                if let Err(error) =
                    emit_instance(instance_id, InstancePayloadType::Removed)
                        .await
                {
                    tracing::warn!(
                        "Failed to emit removed instance {instance_id}: {error}"
                    );
                }
            }
        }
        InstallCleanup::RestoreExistingInstance { instance_id } => {
            if let Some(rollback) = &job_state.rollback {
                if let Some(staging_dir) = &job_state.paths.staging_dir {
                    restore_instance_update(staging_dir, rollback, state)
                        .await?;
                } else {
                    crate::state::instances::commands::set_instance_install_stage(
                        instance_id,
                        rollback.install_stage,
                        &state.pool,
                    )
                    .await?;
                }
                if let Err(error) =
                    emit_instance(instance_id, InstancePayloadType::Edited)
                        .await
                {
                    tracing::warn!(
                        "Failed to emit restored instance {instance_id}: {error}"
                    );
                }
            }
        }
    }

    Ok(())
}
