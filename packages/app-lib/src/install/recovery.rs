use super::events::emit_install_job;
use super::model::{
    InstallCleanup, InstallErrorView, InstallInterruptReason,
    InstallJobDisplay, InstallJobEventKind, InstallJobState, InstallJobStatus,
    InstallPhaseDetails, InstallPhaseId, InstallRequest, InstallTarget,
};
use super::store;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::state::{
    ContentEntry, ContentSetRemoteRef, ContentSetRemoteRefType,
    ContentSetSyncProvider, ContentSetSyncState, InstanceFile,
    InstanceMetadata, State,
};
use async_walkdir::WalkDir;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SHARED_INSTANCE_ROLLBACK_FILE: &str = "rollback.json";
const SHARED_INSTANCE_ROLLBACK_INSTANCE_DIR: &str = "instance";

#[derive(Deserialize, Serialize)]
struct SharedInstanceUpdateRollback {
    files: Vec<InstanceFile>,
    entries: Vec<ContentEntry>,
}

pub(super) async fn prepare_shared_instance_update_backup(
    job_id: Uuid,
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<PathBuf> {
    let staging_dir = state
        .directories
        .metadata_dir()
        .join("install_job_backups")
        .join(job_id.to_string());
    if tokio::fs::try_exists(&staging_dir).await? {
        crate::util::io::remove_dir_all(&staging_dir).await?;
    }
    crate::util::io::create_dir_all(&staging_dir).await?;

    let result = async {
        let files = content_rows::get_instance_files(
            &metadata.instance.id,
            &state.pool,
        )
        .await?;
        let entries = content_rows::get_content_entries(
            &metadata.applied_content_set.id,
            &state.pool,
        )
        .await?;
        let snapshot = SharedInstanceUpdateRollback { files, entries };
        let instance_path = state
            .directories
            .instances_dir()
            .join(&metadata.instance.path);
        copy_directory(
            &instance_path,
            &staging_dir.join(SHARED_INSTANCE_ROLLBACK_INSTANCE_DIR),
            state,
        )
        .await?;
        crate::util::io::write(
            staging_dir.join(SHARED_INSTANCE_ROLLBACK_FILE),
            serde_json::to_vec(&snapshot)?,
        )
        .await?;

        Ok::<(), crate::Error>(())
    }
    .await;

    if result.is_err() {
        let _ = crate::util::io::remove_dir_all(&staging_dir).await;
    }
    result?;
    Ok(staging_dir)
}

pub(super) async fn clear_staging_dir(job_state: &InstallJobState) {
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
    }
}

async fn restore_shared_instance_update(
    staging_dir: &Path,
    rollback: &super::model::InstallRollbackState,
    state: &State,
) -> crate::Result<()> {
    let snapshot = serde_json::from_slice::<SharedInstanceUpdateRollback>(
        &crate::util::io::read(staging_dir.join(SHARED_INSTANCE_ROLLBACK_FILE))
            .await?,
    )?;
    let instance_path = state
        .directories
        .instances_dir()
        .join(&rollback.instance.instance.path);
    if tokio::fs::try_exists(&instance_path).await? {
        crate::util::io::remove_dir_all(&instance_path).await?;
    }
    copy_directory(
        &staging_dir.join(SHARED_INSTANCE_ROLLBACK_INSTANCE_DIR),
        &instance_path,
        state,
    )
    .await?;
    content_rows::restore_instance_content_snapshot(
        &rollback.instance.instance.id,
        &snapshot.files,
        &snapshot.entries,
        &state.pool,
    )
    .await?;
    restore_instance_metadata(&rollback.instance, state).await?;

    Ok(())
}

async fn restore_instance_metadata(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let content_set_id = metadata.applied_content_set.id.as_str();
    let mut tx = state.pool.begin().await?;
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
        &metadata.groups,
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

async fn copy_directory(
    source: &Path,
    target: &Path,
    state: &State,
) -> crate::Result<()> {
    crate::util::io::create_dir_all(target).await?;
    let mut walker = WalkDir::new(source);
    while let Some(entry) = walker.next().await {
        let entry = entry.map_err(|error| {
            crate::ErrorKind::FSError(format!(
                "Failed to read instance backup path: {error}"
            ))
        })?;
        let entry_path = entry.path();
        let relative_path = entry_path.strip_prefix(source)?;
        let target_path = target.join(relative_path);
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            crate::util::io::create_dir_all(&target_path).await?;
        } else if file_type.is_file() {
            crate::util::fetch::copy(
                &entry_path,
                &target_path,
                &state.io_semaphore,
            )
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
    let jobs = store::list_interrupted_candidates(state).await?;

    for mut job in jobs {
        if job.state.display.is_none() {
            job.state.display = display_from_request(&job.state);
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
        if let Err(error) = apply_cleanup(&job.state, state).await {
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
        } else {
            job.state
                .record_event(InstallJobEventKind::RollbackCompleted);
        }
        clear_deleted_new_instance_id(&mut job.state);

        let record = store::update_status(
            job.id,
            InstallJobStatus::Interrupted,
            &job.state,
            state,
        )
        .await?;
        if job.state.rollback_error.is_none() {
            clear_staging_dir(&job.state).await;
        }
        emit_install_job(&record.snapshot()).await?;
    }

    Ok(())
}

fn clear_deleted_new_instance_id(job_state: &mut InstallJobState) {
    if matches!(job_state.cleanup, InstallCleanup::DeleteNewInstance { .. }) {
        job_state.target = InstallTarget::NewInstance { instance_id: None };
        job_state.cleanup =
            InstallCleanup::DeleteNewInstance { instance_id: None };
    }
}

fn display_from_request(state: &InstallJobState) -> Option<InstallJobDisplay> {
    match &state.request {
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
                let _ = crate::state::remove_instance(instance_id, state).await;
                let _ =
                    emit_instance(instance_id, InstancePayloadType::Removed)
                        .await;
            }
        }
        InstallCleanup::RestoreExistingInstance { instance_id } => {
            if let Some(rollback) = &job_state.rollback {
                if matches!(
                    &job_state.request,
                    InstallRequest::UpdateSharedInstance { .. }
                ) && let Some(staging_dir) = &job_state.paths.staging_dir
                {
                    restore_shared_instance_update(
                        staging_dir,
                        rollback,
                        state,
                    )
                    .await?;
                } else {
                    crate::state::instances::commands::set_instance_install_stage(
                        instance_id,
                        rollback.install_stage,
                        &state.pool,
                    )
                    .await?;
                }
                emit_instance(instance_id, InstancePayloadType::Edited).await?;
            }
        }
    }

    Ok(())
}
