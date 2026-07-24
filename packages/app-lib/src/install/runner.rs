use super::events::{InstallProgressReporter, emit_install_job};
use super::model::{
    InstallCleanup, InstallErrorContext, InstallErrorView, InstallJobDisplay,
    InstallJobEventKind, InstallJobSnapshot, InstallJobState, InstallJobStatus,
    InstallPhaseDetails, InstallPhaseId, InstallPostInstallEdit,
    InstallProgress, InstallRequest, InstallRollbackState, InstallTarget,
    SharedInstanceInstallData,
};
use super::shared_instance::{
    apply_shared_instance_content, apply_shared_instance_update,
    attach_pending_shared_instance, finalize_shared_instance_attachment,
    shared_instance_link, shared_instance_pack_location,
};
use super::{diagnostics, recovery, store};
use crate::ErrorKind;
use crate::api::pack::install_from::{
    CreatePackLocation, generate_pack_from_file,
    generate_pack_from_version_id_with_reporter, get_instance_from_pack,
};
use crate::api::pack::install_mrpack::install_zipped_mrpack_files_with_reporter;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{
    ContentSourceKind, InstanceInstallStage, InstanceLink, ModLoader, State,
};
use crate::util::fetch::DownloadReason;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn create_instance(
    name: String,
    game_version: String,
    loader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    link: InstanceLink,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::CreateInstance {
        name,
        game_version,
        loader,
        loader_version,
        icon_path,
        link,
    })
    .await
}

pub async fn create_modpack_instance(
    location: CreatePackLocation,
    post_install_edit: Option<InstallPostInstallEdit>,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::CreateModpackInstance {
        location,
        post_install_edit,
    })
    .await
}

pub async fn create_shared_instance(
    data: SharedInstanceInstallData,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::CreateSharedInstance { data }).await
}

pub async fn update_shared_instance(
    instance_id: String,
    data: SharedInstanceInstallData,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::UpdateSharedInstance { instance_id, data }).await
}

pub async fn import_instance(
    launcher_type: crate::api::pack::import::ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::ImportInstance {
        launcher_type,
        base_path,
        instance_folder,
    })
    .await
}

pub async fn duplicate_instance(
    source_instance_id: String,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::DuplicateInstance { source_instance_id }).await
}

pub async fn install_existing_instance(
    instance_id: String,
    force: bool,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::InstallExistingInstance { instance_id, force }).await
}

pub async fn install_pack_to_existing_instance(
    instance_id: String,
    location: CreatePackLocation,
    post_install_edit: Option<InstallPostInstallEdit>,
) -> crate::Result<InstallJobSnapshot> {
    start(InstallRequest::InstallPackToExistingInstance {
        instance_id,
        location,
        post_install_edit,
    })
    .await
}

pub async fn list_jobs(
    include_finished: bool,
) -> crate::Result<Vec<InstallJobSnapshot>> {
    let state = State::get().await?;
    Ok(store::list(include_finished, &state)
        .await?
        .into_iter()
        .map(|job| job.snapshot())
        .collect())
}

pub async fn get_job(job_id: Uuid) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    Ok(store::get_required(job_id, &state).await?.snapshot())
}

pub async fn job_support_details(job_id: Uuid) -> crate::Result<String> {
    let state = State::get().await?;
    let job = store::get_required(job_id, &state).await?;
    diagnostics::build_job_support_details(&job, &state).await
}

pub async fn retry_job(job_id: Uuid) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let mut job = store::get_required(job_id, &state).await?;

    if !matches!(
        job.status,
        InstallJobStatus::Failed | InstallJobStatus::Interrupted
    ) {
        return Err(crate::ErrorKind::InputError(
            "Only failed or interrupted install jobs can be retried"
                .to_string(),
        )
        .into());
    }

    if job.state.rollback_error.is_some() {
        recovery::apply_cleanup(&job.state, &state).await?;
        recovery::clear_staging_dir(&job.state).await;
    }

    job.state.target = job.state.request.target();
    job.state.cleanup = job.state.request.cleanup();
    job.state.rollback = None;
    job.state.paths.staging_dir = None;
    job.state.error = None;
    job.state.rollback_error = None;
    job.state.context = None;
    job.state.progress.phase = InstallPhaseId::PreparingInstance;
    job.state.progress.progress = None;
    job.state.progress.details = InstallPhaseDetails::Empty;
    prepare_initial_instance(&mut job.state, &state).await?;
    job.state.record_event(InstallJobEventKind::JobQueued {
        kind: job.state.request.kind(),
    });

    let record = store::update_status(
        job_id,
        InstallJobStatus::Queued,
        &job.state,
        &state,
    )
    .await?;
    lock_existing_instance_if_needed(&job.state, &state).await?;
    emit_install_job(&record.snapshot()).await?;
    spawn_job(job_id);

    Ok(record.snapshot())
}

pub async fn cancel_job(job_id: Uuid) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let mut job = store::get_required(job_id, &state).await?;

    if job.status != InstallJobStatus::Queued {
        return Err(crate::ErrorKind::InputError(
            "Only queued install jobs can be canceled".to_string(),
        )
        .into());
    }

    let canceled_phase = job.state.progress.phase;
    job.state.error = Some(InstallErrorView::from_message(
        "canceled",
        canceled_phase,
        "Install was canceled",
    ));
    job.state.record_event(InstallJobEventKind::JobCanceled {
        phase: canceled_phase,
    });
    job.state
        .record_event(InstallJobEventKind::RollbackStarted {
            cleanup: job.state.cleanup.clone(),
        });
    match recovery::apply_cleanup(&job.state, &state).await {
        Ok(()) => job
            .state
            .record_event(InstallJobEventKind::RollbackCompleted),
        Err(error) => {
            job.state.rollback_error = Some(InstallErrorView::from_error(
                "rollback_error",
                InstallPhaseId::RollingBack,
                &error,
                None,
            ));
            job.state.record_event(InstallJobEventKind::RollbackFailed {
                message: error.to_string(),
            });
        }
    }
    clear_deleted_new_instance_id(&mut job.state);
    let record = store::update_status(
        job_id,
        InstallJobStatus::Canceled,
        &job.state,
        &state,
    )
    .await?;
    if job.state.rollback_error.is_none() {
        recovery::clear_staging_dir(&job.state).await;
    }
    emit_install_job(&record.snapshot()).await?;

    Ok(record.snapshot())
}

pub async fn dismiss_job(job_id: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    store::dismiss(job_id, &state).await
}

async fn start(request: InstallRequest) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let id = Uuid::new_v4();
    let mut job_state = InstallJobState::new(request);
    prepare_initial_instance(&mut job_state, &state).await?;
    let record =
        store::insert(id, &job_state, InstallJobStatus::Queued, &state).await?;
    lock_existing_instance_if_needed(&job_state, &state).await?;
    emit_install_job(&record.snapshot()).await?;
    spawn_job(id);
    Ok(record.snapshot())
}

async fn prepare_initial_instance(
    job_state: &mut InstallJobState,
    state: &State,
) -> crate::Result<()> {
    match job_state.request.clone() {
        InstallRequest::CreateInstance {
            name,
            game_version,
            loader,
            loader_version,
            icon_path,
            link,
        } => {
            let metadata = crate::api::instance::create(
                name,
                game_version,
                loader,
                loader_version,
                icon_path,
                link,
            )
            .await?;
            set_display(
                job_state,
                metadata.instance.name,
                metadata.instance.icon_path,
            );
            set_instance_id(job_state, metadata.instance.id);
        }
        InstallRequest::CreateModpackInstance {
            location,
            post_install_edit,
        } => {
            let preview = get_instance_from_pack(location).await?;
            let name = post_install_edit
                .as_ref()
                .and_then(|edit| edit.name.clone())
                .unwrap_or_else(|| preview.name.clone());
            let icon_path = match post_install_edit
                .as_ref()
                .and_then(|edit| edit.icon_path.as_ref())
            {
                Some(icon_path) => icon_path.clone(),
                None => preview
                    .icon
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
                    .or_else(|| preview.icon_url.clone()),
            };
            let link = post_install_edit
                .as_ref()
                .and_then(|edit| edit.link.clone())
                .or_else(|| preview.link.clone())
                .unwrap_or(InstanceLink::Unmanaged);
            let metadata = crate::api::instance::create(
                name,
                preview.game_version,
                preview.modloader,
                preview.loader_version,
                icon_path,
                link,
            )
            .await?;
            set_display(
                job_state,
                metadata.instance.name,
                metadata.instance.icon_path,
            );
            set_instance_id(job_state, metadata.instance.id);
        }
        InstallRequest::CreateSharedInstance { data } => {
            let shared_link = shared_instance_link(data.modpack.as_ref());
            let (game_version, loader, loader_version, icon_path) =
                if let Some(modpack) = data.modpack.clone() {
                    let preview = get_instance_from_pack(
                        shared_instance_pack_location(modpack),
                    )
                    .await?;
                    (
                        preview.game_version,
                        preview.modloader,
                        preview.loader_version,
                        data.instance_icon_url
                            .clone()
                            .or_else(|| {
                                preview.icon.as_ref().map(|path| {
                                    path.to_string_lossy().to_string()
                                })
                            })
                            .or_else(|| preview.icon_url.clone()),
                    )
                } else {
                    (
                        data.game_version.clone(),
                        data.loader,
                        data.loader_version.clone(),
                        data.instance_icon_url.clone(),
                    )
                };
            let metadata = crate::api::instance::create(
                data.name.clone(),
                game_version,
                loader,
                loader_version,
                icon_path,
                shared_link,
            )
            .await?;
            set_display(
                job_state,
                metadata.instance.name,
                metadata.instance.icon_path,
            );
            let instance_id = metadata.instance.id;
            attach_pending_shared_instance(&instance_id, &data, state).await?;
            emit_instance(&instance_id, InstancePayloadType::Edited).await?;
            set_instance_id(job_state, instance_id);
        }
        InstallRequest::ImportInstance {
            instance_folder, ..
        } => {
            let metadata = crate::api::instance::create(
                instance_folder,
                "1.19.4".to_string(),
                ModLoader::Vanilla,
                Some("latest".to_string()),
                None,
                InstanceLink::Unmanaged,
            )
            .await?;
            set_display(
                job_state,
                metadata.instance.name,
                metadata.instance.icon_path,
            );
            set_instance_id(job_state, metadata.instance.id);
        }
        InstallRequest::DuplicateInstance { source_instance_id } => {
            let metadata =
                crate::state::get_instance(&source_instance_id, &state.pool)
                    .await?
                    .ok_or_else(|| {
                        crate::ErrorKind::InputError(
                            "Unknown instance".to_string(),
                        )
                    })?;
            let created = crate::api::instance::create(
                metadata.instance.name,
                metadata.applied_content_set.game_version,
                metadata.applied_content_set.loader,
                metadata.applied_content_set.loader_version,
                metadata.instance.icon_path,
                metadata.link,
            )
            .await?;
            set_display(
                job_state,
                created.instance.name,
                created.instance.icon_path,
            );
            set_instance_id(job_state, created.instance.id);
        }
        InstallRequest::InstallExistingInstance { instance_id, .. }
        | InstallRequest::InstallPackToExistingInstance {
            instance_id, ..
        }
        | InstallRequest::UpdateSharedInstance { instance_id, .. } => {
            prepare_existing_rollback(job_state, state, &instance_id).await?;
        }
    }

    Ok(())
}

fn spawn_job(job_id: Uuid) {
    tokio::spawn(async move {
        if let Err(error) = Box::pin(run_job(job_id)).await {
            tracing::error!(
                "Install job {job_id} failed to update state: {error}"
            );
        }
    });
}

async fn run_job(job_id: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let mut job = store::get_required(job_id, &state).await?;

    if job.status != InstallJobStatus::Queued {
        return Ok(());
    }

    let _install_permit = state.install_job_semaphore.acquire().await?;
    job = store::get_required(job_id, &state).await?;

    if job.status != InstallJobStatus::Queued {
        return Ok(());
    }

    let mut job_state = job.state.clone();
    job_state.record_event(InstallJobEventKind::JobStarted);
    let record = store::update_status(
        job_id,
        InstallJobStatus::Running,
        &job_state,
        &state,
    )
    .await?;
    emit_install_job(&record.snapshot()).await?;

    let result = Box::pin(run_request(job_id, &mut job_state, &state)).await;
    if let Ok(record) = store::get_required(job_id, &state).await {
        job_state = record.state;
    }

    let result = match result {
        Ok(instance_id) => {
            if let Some(instance_id) = instance_id {
                set_instance_id(&mut job_state, instance_id);
            }
            finalize_existing_instance_success(&job_state, &state).await
        }
        Err(error) => Err(error),
    };

    match result {
        Ok(()) => {
            job_state.record_event(InstallJobEventKind::JobSucceeded {
                instance_id: current_instance_id(&job_state),
            });
            job_state.progress.phase = InstallPhaseId::Finalizing;
            job_state.progress.progress = None;
            job_state.progress.details = InstallPhaseDetails::Empty;
            job_state.error = None;
            job_state.rollback_error = None;
            job_state.context = None;
            let record = store::update_status(
                job_id,
                InstallJobStatus::Succeeded,
                &job_state,
                &state,
            )
            .await?;
            recovery::clear_staging_dir(&job_state).await;
            emit_install_job(&record.snapshot()).await?;
        }
        Err(error) => {
            let failed_phase = job_state.progress.phase;
            let error_view = install_error_view(
                failed_phase,
                &error,
                job_state.context.clone(),
            );
            job_state.record_event(InstallJobEventKind::Failed {
                phase: failed_phase,
                code: error_view.code.clone(),
                message: error_view.message.clone(),
            });
            job_state.error = Some(error_view);
            job_state.progress.phase = InstallPhaseId::RollingBack;
            job_state.progress.progress = None;
            job_state.progress.details = InstallPhaseDetails::Empty;
            job_state.record_event(InstallJobEventKind::RollbackStarted {
                cleanup: job_state.cleanup.clone(),
            });
            if let Err(rollback_error) =
                recovery::apply_cleanup(&job_state, &state).await
            {
                tracing::error!(
                    "Error rolling back failed install job {job_id}: {rollback_error}"
                );
                job_state.rollback_error = Some(install_error_view(
                    InstallPhaseId::RollingBack,
                    &rollback_error,
                    None,
                ));
                job_state.record_event(InstallJobEventKind::RollbackFailed {
                    message: rollback_error.to_string(),
                });
            } else {
                job_state.record_event(InstallJobEventKind::RollbackCompleted);
            }
            clear_deleted_new_instance_id(&mut job_state);
            let record = store::update_status(
                job_id,
                InstallJobStatus::Failed,
                &job_state,
                &state,
            )
            .await?;
            if job_state.rollback_error.is_none() {
                recovery::clear_staging_dir(&job_state).await;
            }
            emit_install_job(&record.snapshot()).await?;
            return Err(error);
        }
    }

    Ok(())
}

async fn run_request(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
) -> crate::Result<Option<String>> {
    match job_state.request.clone() {
        InstallRequest::CreateInstance {
            name,
            game_version,
            loader,
            loader_version: _,
            icon_path: _,
            link: _,
        } => {
            let Some(instance_id) = current_instance_id(job_state) else {
                return Err(crate::ErrorKind::InputError(
                    "Install job is missing its instance id".to_string(),
                )
                .into());
            };
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::PreparingInstance,
                InstallPhaseDetails::Instance { name: name.clone() },
            )
            .await?;
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::DownloadingMinecraft,
                InstallPhaseDetails::Minecraft {
                    game_version,
                    loader,
                },
            )
            .await?;
            let context =
                crate::state::instances::commands::get_instance_launch_context(
                    &instance_id,
                    &state.pool,
                )
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError("Unknown instance".to_string())
                })?;
            crate::launcher::install_minecraft_with_reporter(
                &context,
                false,
                Some(InstallProgressReporter::new(job_id, job_state.clone())),
            )
            .await?;
            Ok(Some(instance_id))
        }
        InstallRequest::CreateModpackInstance {
            location,
            post_install_edit,
        } => {
            let Some(instance_id) = current_instance_id(job_state) else {
                return Err(crate::ErrorKind::InputError(
                    "Install job is missing its instance id".to_string(),
                )
                .into());
            };
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::ResolvingPack,
                modpack_details(&location),
            )
            .await?;
            Box::pin(install_pack(
                job_id,
                job_state,
                location,
                instance_id.clone(),
                DownloadReason::Modpack,
            ))
            .await?;
            apply_post_install_edit(&instance_id, post_install_edit).await?;
            Ok(Some(instance_id))
        }
        InstallRequest::CreateSharedInstance { data } => {
            let Some(instance_id) = current_instance_id(job_state) else {
                return Err(crate::ErrorKind::InputError(
                    "Install job is missing its instance id".to_string(),
                )
                .into());
            };
            Box::pin(apply_shared_instance_content(
                job_id,
                job_state,
                state,
                &instance_id,
                &data,
            ))
            .await?;

            finalize_shared_instance_attachment(&instance_id, &data, state)
                .await?;
            emit_instance(&instance_id, InstancePayloadType::Edited).await?;

            Ok(Some(instance_id))
        }
        InstallRequest::ImportInstance {
            launcher_type,
            base_path,
            instance_folder,
        } => {
            let Some(instance_id) = current_instance_id(job_state) else {
                return Err(crate::ErrorKind::InputError(
                    "Install job is missing its instance id".to_string(),
                )
                .into());
            };
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::PreparingInstance,
                InstallPhaseDetails::Import {
                    launcher_type,
                    instance_folder: instance_folder.clone(),
                },
            )
            .await?;
            crate::api::pack::import::import_instance_with_reporter(
                &instance_id,
                launcher_type,
                base_path,
                instance_folder,
                InstallProgressReporter::new(job_id, job_state.clone()),
            )
            .await?;
            Ok(Some(instance_id))
        }
        InstallRequest::DuplicateInstance { source_instance_id } => {
            let Some(instance_id) = current_instance_id(job_state) else {
                return Err(crate::ErrorKind::InputError(
                    "Install job is missing its instance id".to_string(),
                )
                .into());
            };
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::PreparingInstance,
                InstallPhaseDetails::Empty,
            )
            .await?;
            let state = State::get().await?;
            crate::api::pack::import::copy_dotminecraft_with_reporter(
                &instance_id,
                crate::api::instance::get_full_path(&source_instance_id)
                    .await?,
                &state.io_semaphore,
                InstallProgressReporter::new(job_id, job_state.clone()),
                InstallPhaseDetails::Empty,
            )
            .await?;
            let context =
                crate::state::instances::commands::get_instance_launch_context(
                    &instance_id,
                    &state.pool,
                )
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError("Unknown instance".to_string())
                })?;
            crate::launcher::install_minecraft_with_reporter(
                &context,
                false,
                Some(InstallProgressReporter::new(job_id, job_state.clone())),
            )
            .await?;
            emit_instance(&instance_id, InstancePayloadType::Edited).await?;
            Ok(Some(instance_id))
        }
        InstallRequest::InstallExistingInstance { instance_id, force } => {
            prepare_existing_rollback(job_state, state, &instance_id).await?;
            lock_existing_instance(&instance_id, state).await?;
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::DownloadingMinecraft,
                InstallPhaseDetails::Empty,
            )
            .await?;
            let context =
                crate::state::instances::commands::get_instance_launch_context(
                    &instance_id,
                    &state.pool,
                )
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError("Unknown instance".to_string())
                })?;
            crate::launcher::install_minecraft_with_reporter(
                &context,
                force,
                Some(InstallProgressReporter::new(job_id, job_state.clone())),
            )
            .await?;
            Ok(Some(instance_id))
        }
        InstallRequest::InstallPackToExistingInstance {
            instance_id,
            location,
            post_install_edit,
        } => {
            prepare_existing_rollback(job_state, state, &instance_id).await?;
            lock_existing_instance(&instance_id, state).await?;
            let disabled_project_ids = remove_existing_pack_content(
                job_id,
                job_state,
                state,
                &instance_id,
            )
            .await?;
            Box::pin(install_pack(
                job_id,
                job_state,
                location,
                instance_id.clone(),
                DownloadReason::Modpack,
            ))
            .await?;
            restore_disabled_projects(
                &instance_id,
                disabled_project_ids,
                state,
            )
            .await?;
            apply_post_install_edit(&instance_id, post_install_edit).await?;
            Ok(Some(instance_id))
        }
        InstallRequest::UpdateSharedInstance { instance_id, data } => {
            prepare_existing_rollback(job_state, state, &instance_id).await?;
            lock_existing_instance(&instance_id, state).await?;
            let rollback_instance = job_state
                .rollback
                .as_ref()
                .map(|rollback| rollback.instance.clone())
                .ok_or_else(|| {
                    crate::ErrorKind::OtherError(
                        "Shared instance update rollback state is missing"
                            .to_string(),
                    )
                })?;
            let staging_dir = recovery::prepare_shared_instance_update_backup(
                job_id,
                &rollback_instance,
                state,
            )
            .await?;
            job_state.paths.staging_dir = Some(staging_dir);
            let record = store::update_state(job_id, job_state, state).await?;
            emit_install_job(&record.snapshot()).await?;
            let disabled_project_ids =
                disabled_project_ids(&instance_id, state).await?;
            Box::pin(apply_shared_instance_update(
                job_id,
                job_state,
                state,
                &instance_id,
                &data,
            ))
            .await?;
            restore_disabled_projects(
                &instance_id,
                disabled_project_ids,
                state,
            )
            .await?;
            finalize_shared_instance_attachment(&instance_id, &data, state)
                .await?;
            emit_instance(&instance_id, InstancePayloadType::Edited).await?;
            Ok(Some(instance_id))
        }
    }
}

async fn apply_post_install_edit(
    instance_id: &str,
    edit: Option<InstallPostInstallEdit>,
) -> crate::Result<()> {
    let Some(edit) = edit else {
        return Ok(());
    };

    if edit.name.is_none() && edit.icon_path.is_none() && edit.link.is_none() {
        return Ok(());
    }

    crate::api::instance::edit(
        instance_id,
        crate::state::instances::commands::EditInstance {
            name: edit.name,
            icon_path: edit.icon_path,
            link: edit.link,
            ..Default::default()
        },
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

async fn disabled_project_ids(
    instance_id: &str,
    state: &State,
) -> crate::Result<HashSet<String>> {
    Ok(crate::state::instances::commands::list_project_files(
        instance_id,
        state,
    )
    .await?
    .into_iter()
    .filter_map(|file| (!file.enabled).then_some(file.project_id?))
    .collect())
}

async fn remove_existing_pack_content(
    job_id: Uuid,
    job_state: &InstallJobState,
    state: &State,
    instance_id: &str,
) -> crate::Result<HashSet<String>> {
    let metadata = crate::state::instances::commands::get_instance_metadata(
        instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let (project_id, version_id) = match &metadata.link {
        InstanceLink::ModrinthModpack {
            project_id,
            version_id,
        } => (project_id.clone(), version_id.clone()),
        InstanceLink::ServerProjectModpack {
            content_project_id,
            content_version_id,
            ..
        } => (content_project_id.clone(), content_version_id.clone()),
        InstanceLink::ImportedModpack { .. } => {
            remove_existing_imported_pack_content(
                instance_id,
                &metadata,
                state,
            )
            .await?;
            return Ok(HashSet::new());
        }
        _ => return Ok(HashSet::new()),
    };

    let disabled_project_ids =
        crate::state::instances::commands::list_project_files(
            instance_id,
            state,
        )
        .await?
        .into_iter()
        .filter_map(|file| (!file.enabled).then_some(file.project_id?))
        .collect::<HashSet<_>>();
    let reporter = InstallProgressReporter::new(job_id, job_state.clone());
    let old_pack = generate_pack_from_version_id_with_reporter(
        project_id.clone(),
        version_id.clone(),
        metadata.instance.name.clone(),
        None,
        instance_id.to_string(),
        DownloadReason::Update,
        reporter,
    )
    .await?;

    crate::api::pack::install_mrpack::remove_all_related_files(
        instance_id.to_string(),
        old_pack.file,
    )
    .await?;

    Ok(disabled_project_ids)
}

async fn remove_existing_imported_pack_content(
    instance_id: &str,
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let _content_lock = state.lock_instance_content(instance_id).await;
    let entries = content_rows::get_content_entries(
        &metadata.applied_content_set.id,
        &state.pool,
    )
    .await?;
    let files = content_rows::get_instance_files(instance_id, &state.pool)
        .await?
        .into_iter()
        .map(|file| (file.id.clone(), file))
        .collect::<std::collections::HashMap<_, _>>();
    let base = state
        .directories
        .instances_dir()
        .join(&metadata.instance.path);

    let mut removed_file_ids = HashSet::new();
    for entry in entries {
        if !matches!(
            entry.source_kind,
            ContentSourceKind::ImportedModpack
                | ContentSourceKind::ModrinthModpack
        ) {
            continue;
        }

        let Some(file_id) = entry.file_id else {
            continue;
        };
        if !removed_file_ids.insert(file_id.clone()) {
            continue;
        }

        let Some(file) = files.get(&file_id) else {
            continue;
        };
        crate::util::io::remove_file(base.join(&file.relative_path)).await?;
        let mut tx = state.pool.begin().await?;
        content_rows::remove_content_entries_for_file(
            &metadata.applied_content_set.id,
            &file.id,
            &mut tx,
        )
        .await?;
        content_rows::remove_instance_file_by_relative_path(
            instance_id,
            &file.relative_path,
            &mut tx,
        )
        .await?;
        tx.commit().await?;
    }

    Ok(())
}

async fn restore_disabled_projects(
    instance_id: &str,
    disabled_project_ids: HashSet<String>,
    state: &State,
) -> crate::Result<()> {
    if disabled_project_ids.is_empty() {
        return Ok(());
    }

    for file in crate::state::instances::commands::list_project_files(
        instance_id,
        state,
    )
    .await?
    {
        if file.enabled
            && let Some(project_id) = &file.project_id
            && disabled_project_ids.contains(project_id)
        {
            crate::state::instances::commands::toggle_disable_project(
                instance_id,
                &file.relative_path,
                Some(false),
                state,
            )
            .await?;
        }
    }

    Ok(())
}

pub(super) async fn install_pack(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    location: CreatePackLocation,
    instance_id: String,
    reason: DownloadReason,
) -> crate::Result<()> {
    let reporter = InstallProgressReporter::new(job_id, job_state.clone());
    reporter
        .update(
            InstallPhaseId::DownloadingPackFile,
            None,
            modpack_details(&location),
        )
        .await?;

    let create_pack = match location {
        CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title,
            icon_url,
        } => {
            reporter
                .set_context(
                    InstallErrorContext::new("download modpack file")
                        .project_id(project_id.clone())
                        .version_id(version_id.clone())
                        .build(),
                )
                .await?;
            generate_pack_from_version_id_with_reporter(
                project_id,
                version_id,
                title,
                icon_url,
                instance_id.clone(),
                reason,
                reporter.clone(),
            )
            .await?
        }
        CreatePackLocation::FromFile { path } => {
            reporter
                .set_context(
                    InstallErrorContext::new("read local modpack file")
                        .source_path(path.display().to_string())
                        .build(),
                )
                .await?;
            generate_pack_from_file(path, instance_id.clone()).await?
        }
    };

    Box::pin(install_zipped_mrpack_files_with_reporter(
        create_pack,
        false,
        reason,
        reporter,
    ))
    .await?;

    Ok(())
}

async fn prepare_existing_rollback(
    job_state: &mut InstallJobState,
    state: &State,
    instance_id: &str,
) -> crate::Result<()> {
    if job_state.rollback.is_some() {
        return Ok(());
    }

    let instance = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Unknown instance {instance_id}"
            ))
        })?;
    if instance.quarantined {
        return Err(crate::ErrorKind::InputError(
            "Content in quarantined instances cannot be changed.".to_string(),
        )
        .into());
    }
    let install_stage = instance.instance.install_stage;
    set_display(
        job_state,
        instance.instance.name.clone(),
        instance.instance.icon_path.clone(),
    );
    job_state.rollback = Some(InstallRollbackState {
        instance,
        install_stage,
    });
    job_state.cleanup = InstallCleanup::RestoreExistingInstance {
        instance_id: instance_id.to_string(),
    };

    Ok(())
}

async fn lock_existing_instance_if_needed(
    job_state: &InstallJobState,
    state: &State,
) -> crate::Result<()> {
    if let InstallCleanup::RestoreExistingInstance { instance_id } =
        &job_state.cleanup
    {
        lock_existing_instance(instance_id, state).await?;
    }

    Ok(())
}

async fn lock_existing_instance(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    crate::state::instances::commands::set_instance_install_stage(
        instance_id,
        InstanceInstallStage::MinecraftInstalling,
        &state.pool,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

async fn finalize_existing_instance_success(
    job_state: &InstallJobState,
    state: &State,
) -> crate::Result<()> {
    if let InstallCleanup::RestoreExistingInstance { instance_id } =
        &job_state.cleanup
    {
        crate::state::instances::commands::set_instance_install_stage(
            instance_id,
            InstanceInstallStage::Installed,
            &state.pool,
        )
        .await?;
        emit_instance(instance_id, InstancePayloadType::Edited).await?;
    }

    Ok(())
}

pub(super) async fn update_progress(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    phase: InstallPhaseId,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    job_state.set_progress(phase, None, details);
    let record = store::update_state(job_id, job_state, state).await?;
    emit_install_job(&record.snapshot()).await?;
    Ok(())
}

pub(super) async fn update_content_progress(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    current: u64,
    total: u64,
) -> crate::Result<()> {
    job_state.progress.phase = InstallPhaseId::DownloadingContent;
    job_state.progress.progress = Some(InstallProgress {
        current,
        total,
        secondary: None,
    });
    job_state.progress.details = InstallPhaseDetails::Empty;
    let record = store::update_state(job_id, job_state, state).await?;
    emit_install_job(&record.snapshot()).await?;
    Ok(())
}

fn set_instance_id(job_state: &mut InstallJobState, instance_id: String) {
    job_state.target = match &job_state.target {
        InstallTarget::ExistingInstance { .. } => {
            InstallTarget::ExistingInstance {
                instance_id: instance_id.clone(),
            }
        }
        InstallTarget::NewInstance { .. } => InstallTarget::NewInstance {
            instance_id: Some(instance_id.clone()),
        },
    };
    job_state.cleanup = match &job_state.cleanup {
        InstallCleanup::RestoreExistingInstance { .. } => {
            InstallCleanup::RestoreExistingInstance { instance_id }
        }
        InstallCleanup::DeleteNewInstance { .. } => {
            InstallCleanup::DeleteNewInstance {
                instance_id: Some(instance_id),
            }
        }
    };
}

fn clear_deleted_new_instance_id(job_state: &mut InstallJobState) {
    if matches!(job_state.cleanup, InstallCleanup::DeleteNewInstance { .. }) {
        job_state.target = InstallTarget::NewInstance { instance_id: None };
        job_state.cleanup =
            InstallCleanup::DeleteNewInstance { instance_id: None };
    }
}

fn set_display(
    job_state: &mut InstallJobState,
    title: String,
    icon: Option<String>,
) {
    job_state.display = Some(InstallJobDisplay { title, icon });
}

fn install_error_view(
    phase: InstallPhaseId,
    error: &crate::Error,
    context: Option<InstallErrorContext>,
) -> InstallErrorView {
    let mut view = InstallErrorView::from_error(
        install_error_code(phase, error),
        phase,
        error,
        context,
    );
    if let ErrorKind::SharedInstanceUnavailable(reason) = error.raw.as_ref() {
        view.reason = Some(*reason);
    }
    view
}

fn install_error_code(
    phase: InstallPhaseId,
    error: &crate::Error,
) -> &'static str {
    use InstallPhaseId::*;

    match error.raw.as_ref() {
        ErrorKind::SharedInstanceUnavailable(_) => {
            "shared_instance_unavailable"
        }
        ErrorKind::InputError(_) => match phase {
            PreparingInstance | Finalizing => "instance_error",
            ResolvingPack | DownloadingPackFile | ReadingPackManifest => {
                "pack_error"
            }
            DownloadingContent => "content_error",
            ExtractingOverrides => "path_error",
            PreparingJava => "java_error",
            DownloadingMinecraft => "instance_error",
            RollingBack => "rollback_error",
            ResolvingMinecraft | ResolvingLoader | RunningLoaderProcessors => {
                "launcher_error"
            }
        },
        ErrorKind::LauncherError(_) => match phase {
            RunningLoaderProcessors => "processor_error",
            PreparingJava => "java_error",
            ResolvingLoader => "loader_error",
            _ => "launcher_error",
        },
        ErrorKind::JREError(_) => "java_error",
        ErrorKind::NoValueFor(_) | ErrorKind::MetadataError(_) => match phase {
            ResolvingLoader => "loader_error",
            PreparingJava => "java_error",
            _ => "metadata_error",
        },
        ErrorKind::FetchError(_)
        | ErrorKind::ApiIsDownError(_)
        | ErrorKind::WSError(_)
        | ErrorKind::WSClosedError(_)
        | ErrorKind::Ratelimited { .. } => "network_error",
        ErrorKind::Any(_)
            if matches!(
                phase,
                DownloadingPackFile
                    | DownloadingContent
                    | ResolvingMinecraft
                    | ResolvingLoader
                    | PreparingJava
                    | DownloadingMinecraft
            ) =>
        {
            "network_error"
        }
        ErrorKind::LabrinthError(_) => "api_error",
        ErrorKind::HashError(_, _) => "hash_error",
        ErrorKind::ZipError(_) => "archive_error",
        ErrorKind::DeserializationError(_) | ErrorKind::StripPrefixError(_) => {
            "path_error"
        }
        ErrorKind::FSError(_)
        | ErrorKind::IOError(_)
        | ErrorKind::StdIOError(_)
        | ErrorKind::UTFError(_) => "filesystem_error",
        ErrorKind::INIError(_) | ErrorKind::JSONError(_) => "parse_error",
        ErrorKind::Sqlx(_) | ErrorKind::SqlxMigrate(_) => "database_error",
        ErrorKind::JoinError(_)
        | ErrorKind::RecvError(_)
        | ErrorKind::AcquireError(_)
        | ErrorKind::EventError(_) => "internal_error",
        ErrorKind::OtherError(_) | ErrorKind::Any(_) => "internal_error",
        _ => "unknown_error",
    }
}

fn current_instance_id(job_state: &InstallJobState) -> Option<String> {
    match &job_state.target {
        InstallTarget::NewInstance { instance_id } => instance_id.clone(),
        InstallTarget::ExistingInstance { instance_id } => {
            Some(instance_id.clone())
        }
    }
}

pub(super) fn modpack_details(
    location: &CreatePackLocation,
) -> InstallPhaseDetails {
    match location {
        CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title,
            ..
        } => InstallPhaseDetails::Modpack {
            project_id: Some(project_id.clone()),
            version_id: Some(version_id.clone()),
            title: Some(title.clone()),
        },
        CreatePackLocation::FromFile { .. } => InstallPhaseDetails::Modpack {
            project_id: None,
            version_id: None,
            title: None,
        },
    }
}
