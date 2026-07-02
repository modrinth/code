use super::events::{InstallProgressReporter, emit_install_job};
use super::model::{
    InstallCleanup, InstallErrorView, InstallJobDisplay, InstallJobSnapshot,
    InstallJobState, InstallJobStatus, InstallPhaseDetails, InstallPhaseId,
    InstallPostInstallEdit, InstallRequest, InstallRollbackState,
    InstallTarget, SharedInstanceExternalFileData, SharedInstanceInstallData,
    SharedInstanceInstallModpack,
};
use super::{recovery, store};
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
    ContentSetSyncStatus, ContentSourceKind, InstanceInstallStage,
    InstanceLink, ModLoader, ProjectType, SharedInstanceRole, State,
};
use crate::util::fetch::{DownloadReason, REQWEST_CLIENT};
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

    job.state.target = job.state.request.target();
    job.state.cleanup = job.state.request.cleanup();
    job.state.rollback = None;
    job.state.error = None;
    job.state.progress.phase = InstallPhaseId::PreparingInstance;
    job.state.progress.progress = None;
    job.state.progress.details = InstallPhaseDetails::Empty;
    prepare_initial_instance(&mut job.state, &state).await?;

    let record = store::update_status(
        job_id,
        InstallJobStatus::Queued,
        &job.state,
        &state,
    )
    .await?;
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

    job.state.error = Some(InstallErrorView {
        code: "canceled".to_string(),
        message: "Install was canceled".to_string(),
    });
    recovery::apply_cleanup(&job.state, &state).await?;
    clear_deleted_new_instance_id(&mut job.state);
    let record = store::update_status(
        job_id,
        InstallJobStatus::Canceled,
        &job.state,
        &state,
    )
    .await?;
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
                        preview
                            .icon
                            .as_ref()
                            .map(|path| path.to_string_lossy().to_string())
                            .or_else(|| preview.icon_url.clone()),
                    )
                } else {
                    (
                        data.game_version.clone(),
                        data.loader,
                        data.loader_version.clone(),
                        None,
                    )
                };
            let metadata = crate::api::instance::create(
                data.name,
                game_version,
                loader,
                loader_version,
                icon_path,
                InstanceLink::SharedInstance,
            )
            .await?;
            set_display(
                job_state,
                metadata.instance.name,
                metadata.instance.icon_path,
            );
            set_instance_id(job_state, metadata.instance.id);
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
        } => {
            prepare_existing_rollback(job_state, state, &instance_id).await?;
        }
    }

    Ok(())
}

fn spawn_job(job_id: Uuid) {
    tokio::spawn(async move {
        if let Err(error) = run_job(job_id).await {
            tracing::error!(
                "Install job {job_id} failed to update state: {error}"
            );
        }
    });
}

async fn run_job(job_id: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let job = store::get_required(job_id, &state).await?;

    if job.status != InstallJobStatus::Queued {
        return Ok(());
    }

    let mut job_state = job.state.clone();
    let record = store::update_status(
        job_id,
        InstallJobStatus::Running,
        &job_state,
        &state,
    )
    .await?;
    emit_install_job(&record.snapshot()).await?;

    let result = run_request(job_id, &mut job_state, &state).await;

    match result {
        Ok(instance_id) => {
            if let Some(instance_id) = instance_id {
                set_instance_id(&mut job_state, instance_id);
            }
            job_state.progress.phase = InstallPhaseId::Finalizing;
            job_state.progress.progress = None;
            job_state.progress.details = InstallPhaseDetails::Empty;
            job_state.error = None;
            let record = store::update_status(
                job_id,
                InstallJobStatus::Succeeded,
                &job_state,
                &state,
            )
            .await?;
            emit_install_job(&record.snapshot()).await?;
        }
        Err(error) => {
            job_state.progress.phase = InstallPhaseId::RollingBack;
            job_state.progress.progress = None;
            job_state.progress.details = InstallPhaseDetails::Empty;
            job_state.error = Some(install_error_view(&error));
            recovery::apply_cleanup(&job_state, &state).await?;
            clear_deleted_new_instance_id(&mut job_state);
            let record = store::update_status(
                job_id,
                InstallJobStatus::Failed,
                &job_state,
                &state,
            )
            .await?;
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
            install_pack(
                job_id,
                job_state,
                location,
                instance_id.clone(),
                DownloadReason::Modpack,
            )
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
            update_progress(
                job_id,
                job_state,
                state,
                InstallPhaseId::PreparingInstance,
                InstallPhaseDetails::Instance {
                    name: data.name.clone(),
                },
            )
            .await?;

            if let Some(modpack) = data.modpack.clone() {
                let location = shared_instance_pack_location(modpack);
                update_progress(
                    job_id,
                    job_state,
                    state,
                    InstallPhaseId::ResolvingPack,
                    modpack_details(&location),
                )
                .await?;
                install_pack(
                    job_id,
                    job_state,
                    location,
                    instance_id.clone(),
                    DownloadReason::Modpack,
                )
                .await?;
            } else {
                update_progress(
                    job_id,
                    job_state,
                    state,
                    InstallPhaseId::DownloadingMinecraft,
                    InstallPhaseDetails::Minecraft {
                        game_version: data.game_version.clone(),
                        loader: data.loader,
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
                        crate::ErrorKind::InputError(
                            "Unknown instance".to_string(),
                        )
                    })?;
                crate::launcher::install_minecraft_with_reporter(
                    &context,
                    false,
                    Some(InstallProgressReporter::new(
                        job_id,
                        job_state.clone(),
                    )),
                )
                .await?;
            }

            if !data.modrinth_ids.is_empty() || !data.external_files.is_empty()
            {
                update_progress(
                    job_id,
                    job_state,
                    state,
                    InstallPhaseId::DownloadingContent,
                    InstallPhaseDetails::Empty,
                )
                .await?;
                for version_id in &data.modrinth_ids {
                    crate::state::instances::commands::add_project_from_version(
                        &instance_id,
                        version_id,
                        DownloadReason::Standalone,
                        None,
                        ContentSourceKind::SharedInstance,
                        state,
                    )
                    .await?;
                }

                for file in &data.external_files {
                    install_shared_instance_external_file(
                        &instance_id,
                        file,
                        state,
                    )
                    .await?;
                }
            }

            crate::state::attach_shared_instance(
                &instance_id,
                &data.shared_instance_id,
                SharedInstanceRole::Member,
                ContentSetSyncStatus::UpToDate,
                Some(data.version),
                Some(data.version),
                &state.pool,
            )
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
            let disabled_project_ids = remove_existing_pack_content(
                job_id,
                job_state,
                state,
                &instance_id,
            )
            .await?;
            install_pack(
                job_id,
                job_state,
                location,
                instance_id.clone(),
                DownloadReason::Modpack,
            )
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
        content_rows::remove_content_entries_for_file(
            &metadata.applied_content_set.id,
            &file.id,
            &state.pool,
        )
        .await?;
        content_rows::remove_instance_file_by_relative_path(
            instance_id,
            &file.relative_path,
            &state.pool,
        )
        .await?;
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

async fn install_pack(
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
            generate_pack_from_file(path, instance_id.clone()).await?
        }
    };

    install_zipped_mrpack_files_with_reporter(
        create_pack,
        false,
        reason,
        reporter,
    )
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

    crate::state::instances::commands::set_instance_install_stage(
        instance_id,
        InstanceInstallStage::MinecraftInstalling,
        &state.pool,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

async fn update_progress(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    phase: InstallPhaseId,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    job_state.progress.phase = phase;
    job_state.progress.progress = None;
    job_state.progress.details = details;
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

fn install_error_view(error: &crate::Error) -> InstallErrorView {
    match error.raw.as_ref() {
        ErrorKind::FetchError(_)
        | ErrorKind::ApiIsDownError(_)
        | ErrorKind::WSError(_)
        | ErrorKind::WSClosedError(_) => InstallErrorView {
            code: "network_error".to_string(),
            message: "network_error".to_string(),
        },
        _ => InstallErrorView {
            code: "unknown_error".to_string(),
            message: "unknown_error".to_string(),
        },
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

fn modpack_details(location: &CreatePackLocation) -> InstallPhaseDetails {
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

fn shared_instance_pack_location(
    modpack: SharedInstanceInstallModpack,
) -> CreatePackLocation {
    CreatePackLocation::FromVersionId {
        project_id: modpack.project_id,
        version_id: modpack.version_id,
        title: modpack.title,
        icon_url: modpack.icon_url,
    }
}

async fn install_shared_instance_external_file(
    instance_id: &str,
    file: &SharedInstanceExternalFileData,
    state: &State,
) -> crate::Result<()> {
    let project_type =
        ProjectType::from_name(&file.file_type).ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Unknown shared instance file type {}",
                file.file_type
            ))
        })?;
    let response = REQWEST_CLIENT.get(&file.url).send().await?;

    if !response.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "Shared instance external file download failed with status {}",
            response.status()
        ))
        .into());
    }

    crate::state::instances::commands::add_project_bytes(
        instance_id,
        &file.file_name,
        response.bytes().await?,
        None,
        Some(project_type),
        ContentSourceKind::SharedInstance,
        None,
        None,
        state,
    )
    .await?;

    Ok(())
}
