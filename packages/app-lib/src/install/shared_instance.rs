use super::events::InstallProgressReporter;
use super::model::{
    InstallJobState, InstallPhaseDetails, InstallPhaseId,
    SharedInstanceExternalFileData, SharedInstanceInstallData,
    SharedInstanceInstallModpack,
};
use super::runner::{
    install_pack, modpack_details, update_content_progress, update_progress,
};
use crate::api::instance::{
    CONFIG_BUNDLE_FILE_TYPE, CONFIG_DIRECTORY, CONFIG_FILE_EXTENSIONS,
    CONFIG_SYNC_ENABLED, MAX_CONFIG_BUNDLE_ENTRIES, is_excluded_config_path,
    read_bounded_config_bundle_entry,
};
use crate::api::pack::install_from::CreatePackLocation;
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{
    CachedEntry, ContentSetSyncStatus, ContentSourceKind, InstanceLink,
    ProjectType, SharedInstanceAttachmentInput, SharedInstanceRole, State,
};
use crate::util::fetch::{DownloadReason, REQWEST_CLIENT};
use futures::StreamExt;
use path_util::SafeRelativeUtf8UnixPathBuf;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

const MAX_SHARED_INSTANCE_EXTERNAL_FILE_SIZE: u64 = 500 * 1024 * 1024;
const MAX_SHARED_INSTANCE_INITIAL_BUFFER_SIZE: u64 = 8 * 1024 * 1024;

pub(super) async fn attach_pending_shared_instance(
    instance_id: &str,
    data: &SharedInstanceInstallData,
    state: &State,
) -> crate::Result<()> {
    crate::state::attach_shared_instance(
        instance_id,
        SharedInstanceAttachmentInput {
            id: data.shared_instance_id.clone(),
            role: SharedInstanceRole::Member,
            manager_id: data.manager_id.clone(),
            server_manager_name: data.server_manager_name.clone(),
            server_manager_icon_url: data.server_manager_icon_url.clone(),
            linked_user_id: data.linked_user_id.clone(),
            status: ContentSetSyncStatus::NotReady,
            applied_version: None,
            latest_version: Some(data.version),
        },
        &state.pool,
    )
    .await
}

pub(super) async fn finalize_shared_instance_attachment(
    instance_id: &str,
    data: &SharedInstanceInstallData,
    state: &State,
) -> crate::Result<()> {
    crate::state::attach_shared_instance(
        instance_id,
        SharedInstanceAttachmentInput {
            id: data.shared_instance_id.clone(),
            role: SharedInstanceRole::Member,
            manager_id: data.manager_id.clone(),
            server_manager_name: data.server_manager_name.clone(),
            server_manager_icon_url: data.server_manager_icon_url.clone(),
            linked_user_id: data.linked_user_id.clone(),
            status: ContentSetSyncStatus::UpToDate,
            applied_version: Some(data.version),
            latest_version: Some(data.version),
        },
        &state.pool,
    )
    .await
}

#[derive(Clone, Debug)]
struct CurrentSharedInstanceProject {
    project_id: String,
    version_id: String,
    relative_path: String,
}

#[derive(Clone, Debug)]
struct CurrentSharedInstanceExternalFile {
    relative_path: String,
}

type SharedInstanceExternalFileKey = (ProjectType, String);

#[derive(Default)]
struct CurrentSharedInstanceContent {
    projects: HashMap<String, CurrentSharedInstanceProject>,
    external_files: HashMap<
        SharedInstanceExternalFileKey,
        CurrentSharedInstanceExternalFile,
    >,
}

#[derive(Clone, Debug)]
struct DesiredSharedInstanceProject {
    project_id: String,
    version_id: String,
}

#[derive(Clone, Debug)]
struct DesiredSharedInstanceExternalFile {
    file: SharedInstanceExternalFileData,
    file_type: ProjectType,
}

#[derive(Default)]
struct DesiredSharedInstanceContent {
    projects: HashMap<String, DesiredSharedInstanceProject>,
    external_files: HashMap<
        SharedInstanceExternalFileKey,
        DesiredSharedInstanceExternalFile,
    >,
    config_bundle: Option<SharedInstanceExternalFileData>,
}

struct SharedInstanceProjectUpdate {
    current: CurrentSharedInstanceProject,
    desired: DesiredSharedInstanceProject,
}

#[derive(Default)]
struct SharedInstanceApplyPlan {
    configuration_changed: bool,
    project_removals: Vec<CurrentSharedInstanceProject>,
    project_updates: Vec<SharedInstanceProjectUpdate>,
    project_additions: Vec<DesiredSharedInstanceProject>,
    external_removals: Vec<CurrentSharedInstanceExternalFile>,
    external_updates: Vec<DesiredSharedInstanceExternalFile>,
    external_additions: Vec<DesiredSharedInstanceExternalFile>,
    config_bundle: Option<SharedInstanceExternalFileData>,
}

impl SharedInstanceApplyPlan {
    async fn build(
        metadata: &crate::state::InstanceMetadata,
        data: &SharedInstanceInstallData,
        state: &State,
    ) -> crate::Result<Self> {
        if shared_instance_update_requires_full_apply(metadata, data) {
            return Ok(Self {
                configuration_changed: true,
                ..Default::default()
            });
        }

        let current = current_shared_instance_content(metadata, state).await?;
        let desired = desired_shared_instance_content(data, state).await?;
        let project_removals = current
            .projects
            .values()
            .filter(|current| {
                !desired.projects.contains_key(&current.project_id)
            })
            .cloned()
            .collect();
        let external_removals = current
            .external_files
            .iter()
            .filter(|(key, _)| !desired.external_files.contains_key(*key))
            .map(|(_, current)| current.clone())
            .collect();
        let mut plan = Self {
            project_removals,
            external_removals,
            config_bundle: desired.config_bundle,
            ..Default::default()
        };

        for desired in desired.projects.into_values() {
            match current.projects.get(&desired.project_id) {
                Some(current) if current.version_id != desired.version_id => {
                    plan.project_updates.push(SharedInstanceProjectUpdate {
                        current: current.clone(),
                        desired,
                    });
                }
                None => plan.project_additions.push(desired),
                Some(_) => {}
            }
        }

        for desired in desired.external_files.into_values() {
            let key = (desired.file_type, desired.file.file_name.clone());
            match current.external_files.get(&key) {
                Some(_) => {
                    plan.external_updates.push(desired);
                }
                _ => plan.external_additions.push(desired),
            }
        }

        Ok(plan)
    }

    fn content_change_count(&self) -> u64 {
        (self.project_updates.len()
            + self.project_additions.len()
            + self.external_updates.len()
            + self.external_additions.len()
            + usize::from(self.config_bundle.is_some())) as u64
    }
}

pub(super) async fn apply_shared_instance_update(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    instance_id: &str,
    data: &SharedInstanceInstallData,
) -> crate::Result<()> {
    let metadata = crate::state::instances::commands::get_instance_metadata(
        instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let plan = SharedInstanceApplyPlan::build(&metadata, data, state).await?;

    if plan.configuration_changed {
        remove_existing_shared_instance_content(instance_id, state).await?;
        Box::pin(apply_shared_instance_content(
            job_id,
            job_state,
            state,
            instance_id,
            data,
        ))
        .await?;
        return Ok(());
    }

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

    let content_change_count = plan.content_change_count();
    if content_change_count > 0 {
        update_content_progress(
            job_id,
            job_state,
            state,
            0,
            content_change_count,
        )
        .await?;
    }

    for project in plan.project_removals {
        crate::state::instances::commands::remove_project(
            instance_id,
            &project.relative_path,
            state,
        )
        .await?;
    }

    for file in plan.external_removals {
        crate::state::instances::commands::remove_project(
            instance_id,
            &file.relative_path,
            state,
        )
        .await?;
    }

    let mut completed_content_changes = 0;
    for update in plan.project_updates {
        let new_path =
            crate::state::instances::commands::add_project_from_version(
                instance_id,
                &update.desired.version_id,
                DownloadReason::Update,
                Some(update.current.version_id),
                ContentSourceKind::SharedInstance,
                state,
            )
            .await?;

        if update.current.relative_path != new_path {
            crate::state::instances::commands::remove_project(
                instance_id,
                &update.current.relative_path,
                state,
            )
            .await?;
        }
        completed_content_changes += 1;
        update_content_progress(
            job_id,
            job_state,
            state,
            completed_content_changes,
            content_change_count,
        )
        .await?;
    }

    for project in plan.project_additions {
        crate::state::instances::commands::add_project_from_version(
            instance_id,
            &project.version_id,
            DownloadReason::Standalone,
            None,
            ContentSourceKind::SharedInstance,
            state,
        )
        .await?;
        completed_content_changes += 1;
        update_content_progress(
            job_id,
            job_state,
            state,
            completed_content_changes,
            content_change_count,
        )
        .await?;
    }

    for file in plan
        .external_updates
        .into_iter()
        .chain(plan.external_additions)
    {
        install_shared_instance_external_file(instance_id, &file.file, state)
            .await?;
        completed_content_changes += 1;
        update_content_progress(
            job_id,
            job_state,
            state,
            completed_content_changes,
            content_change_count,
        )
        .await?;
    }

    if let Some(config_bundle) = plan.config_bundle {
        install_shared_instance_external_file(
            instance_id,
            &config_bundle,
            state,
        )
        .await?;
        completed_content_changes += 1;
        update_content_progress(
            job_id,
            job_state,
            state,
            completed_content_changes,
            content_change_count,
        )
        .await?;
    }

    crate::api::instance::edit(
        instance_id,
        crate::state::EditInstance {
            name: Some(data.name.clone()),
            link: Some(shared_instance_link(data.modpack.as_ref())),
            ..Default::default()
        },
    )
    .await?;

    Ok(())
}

fn shared_instance_update_requires_full_apply(
    metadata: &crate::state::InstanceMetadata,
    data: &SharedInstanceInstallData,
) -> bool {
    let (current_modpack_project_id, current_modpack_version_id) =
        match &metadata.link {
            InstanceLink::SharedInstance {
                modpack_project_id,
                modpack_version_id,
            } => (modpack_project_id.as_deref(), modpack_version_id.as_deref()),
            _ => return true,
        };
    let next_modpack_project_id = data
        .modpack
        .as_ref()
        .map(|modpack| modpack.project_id.as_str());
    let next_modpack_version_id = data
        .modpack
        .as_ref()
        .map(|modpack| modpack.version_id.as_str());

    current_modpack_project_id != next_modpack_project_id
        || current_modpack_version_id != next_modpack_version_id
        || metadata.applied_content_set.game_version != data.game_version
        || metadata.applied_content_set.loader != data.loader
        || metadata.applied_content_set.loader_version != data.loader_version
}

async fn current_shared_instance_content(
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<CurrentSharedInstanceContent> {
    let entries = content_rows::get_content_entries(
        &metadata.applied_content_set.id,
        &state.pool,
    )
    .await?;
    let files =
        content_rows::get_instance_files(&metadata.instance.id, &state.pool)
            .await?
            .into_iter()
            .map(|file| (file.id.clone(), file))
            .collect::<HashMap<_, _>>();
    let version_ids_without_project = entries
        .iter()
        .filter(|entry| entry.source_kind == ContentSourceKind::SharedInstance)
        .filter(|entry| entry.project_id.is_none())
        .filter_map(|entry| entry.version_id.clone())
        .collect::<Vec<_>>();
    let versions_by_id =
        shared_instance_versions_by_id(&version_ids_without_project, state)
            .await?;
    let mut content = CurrentSharedInstanceContent::default();

    for entry in entries {
        if entry.source_kind != ContentSourceKind::SharedInstance {
            continue;
        }

        let Some(file_id) = entry.file_id.as_ref() else {
            continue;
        };
        let Some(file) = files.get(file_id) else {
            continue;
        };

        if let Some(version_id) = entry.version_id.clone() {
            let Some(project_id) = entry.project_id.clone().or_else(|| {
                versions_by_id
                    .get(&version_id)
                    .map(|version| version.project_id.clone())
            }) else {
                continue;
            };

            content.projects.insert(
                project_id.clone(),
                CurrentSharedInstanceProject {
                    project_id,
                    version_id,
                    relative_path: file.relative_path.clone(),
                },
            );
        } else {
            content.external_files.insert(
                (entry.project_type, file.file_name.clone()),
                CurrentSharedInstanceExternalFile {
                    relative_path: file.relative_path.clone(),
                },
            );
        }
    }

    Ok(content)
}

async fn desired_shared_instance_content(
    data: &SharedInstanceInstallData,
    state: &State,
) -> crate::Result<DesiredSharedInstanceContent> {
    let versions_by_id =
        shared_instance_versions_by_id(&data.modrinth_ids, state).await?;
    let mut content = DesiredSharedInstanceContent::default();

    for version_id in &data.modrinth_ids {
        let version = versions_by_id.get(version_id).ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Shared instance version {version_id} was not found"
            ))
        })?;
        content.projects.insert(
            version.project_id.clone(),
            DesiredSharedInstanceProject {
                project_id: version.project_id.clone(),
                version_id: version.id.clone(),
            },
        );
    }

    for file in &data.external_files {
        if file.file_type == CONFIG_BUNDLE_FILE_TYPE {
            if CONFIG_SYNC_ENABLED {
                content.config_bundle = Some(file.clone());
            }
            continue;
        }
        let file_type =
            ProjectType::from_name(&file.file_type).ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Unknown shared instance file type {}",
                    file.file_type
                ))
            })?;
        content.external_files.insert(
            (file_type, file.file_name.clone()),
            DesiredSharedInstanceExternalFile {
                file: file.clone(),
                file_type,
            },
        );
    }

    Ok(content)
}

async fn shared_instance_versions_by_id(
    version_ids: &[String],
    state: &State,
) -> crate::Result<HashMap<String, crate::state::Version>> {
    if version_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut ids = version_ids.iter().map(String::as_str).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    let versions = CachedEntry::get_version_many(
        &ids,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let versions_by_id = versions
        .into_iter()
        .map(|version| (version.id.clone(), version))
        .collect::<HashMap<_, _>>();

    for id in ids {
        if !versions_by_id.contains_key(id) {
            return Err(crate::ErrorKind::InputError(format!(
                "Shared instance version {id} was not found"
            ))
            .into());
        }
    }

    Ok(versions_by_id)
}

pub(super) async fn apply_shared_instance_content(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    instance_id: &str,
    data: &SharedInstanceInstallData,
) -> crate::Result<()> {
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
        Box::pin(install_pack(
            job_id,
            job_state,
            location,
            instance_id.to_string(),
            DownloadReason::Modpack,
        ))
        .await?;
    } else {
        crate::api::instance::edit(
            instance_id,
            crate::state::EditInstance {
                content_set_patch: Some(crate::state::AppliedContentSetPatch {
                    source_kind: Some(ContentSourceKind::SharedInstance),
                    game_version: Some(data.game_version.clone()),
                    protocol_version: Some(None),
                    loader: Some(data.loader),
                    loader_version: Some(data.loader_version.clone()),
                }),
                ..Default::default()
            },
        )
        .await?;
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
                instance_id,
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
    }

    if !data.modrinth_ids.is_empty() || !data.external_files.is_empty() {
        let content_change_count =
            data.modrinth_ids.len() as u64 + data.external_files.len() as u64;
        update_content_progress(
            job_id,
            job_state,
            state,
            0,
            content_change_count,
        )
        .await?;
        let mut completed_content_changes = 0;
        for version_id in &data.modrinth_ids {
            crate::state::instances::commands::add_project_from_version(
                instance_id,
                version_id,
                DownloadReason::Standalone,
                None,
                ContentSourceKind::SharedInstance,
                state,
            )
            .await?;
            completed_content_changes += 1;
            update_content_progress(
                job_id,
                job_state,
                state,
                completed_content_changes,
                content_change_count,
            )
            .await?;
        }

        for file in &data.external_files {
            install_shared_instance_external_file(instance_id, file, state)
                .await?;
            completed_content_changes += 1;
            update_content_progress(
                job_id,
                job_state,
                state,
                completed_content_changes,
                content_change_count,
            )
            .await?;
        }
    }

    crate::api::instance::edit(
        instance_id,
        crate::state::EditInstance {
            name: Some(data.name.clone()),
            link: Some(shared_instance_link(data.modpack.as_ref())),
            ..Default::default()
        },
    )
    .await?;

    Ok(())
}

pub(super) async fn remove_existing_shared_instance_content(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let metadata = crate::state::instances::commands::get_instance_metadata(
        instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
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
        if !entry.source_kind.is_shared_instance_managed() {
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

pub(super) fn shared_instance_pack_location(
    modpack: SharedInstanceInstallModpack,
) -> CreatePackLocation {
    CreatePackLocation::FromVersionId {
        project_id: modpack.project_id,
        version_id: modpack.version_id,
        title: modpack.title,
        icon_url: modpack.icon_url,
    }
}

pub(super) fn shared_instance_link(
    modpack: Option<&SharedInstanceInstallModpack>,
) -> InstanceLink {
    InstanceLink::SharedInstance {
        modpack_project_id: modpack.map(|modpack| modpack.project_id.clone()),
        modpack_version_id: modpack.map(|modpack| modpack.version_id.clone()),
    }
}

async fn install_shared_instance_external_file(
    instance_id: &str,
    file: &SharedInstanceExternalFileData,
    state: &State,
) -> crate::Result<()> {
    if file.file_type == CONFIG_BUNDLE_FILE_TYPE && !CONFIG_SYNC_ENABLED {
        return Ok(());
    }

    validate_shared_instance_external_file_name(&file.file_name)?;

    if file.file_size > MAX_SHARED_INSTANCE_EXTERNAL_FILE_SIZE {
        return Err(crate::ErrorKind::InputError(format!(
            "Shared instance external file {} exceeds the maximum size",
            file.file_name
        ))
        .into());
    }

    let response = REQWEST_CLIENT.get(&file.url).send().await?;

    if !response.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "Shared instance external file download failed with status {}",
            response.status()
        ))
        .into());
    }

    if response
        .content_length()
        .is_some_and(|content_length| content_length != file.file_size)
    {
        return Err(crate::ErrorKind::OtherError(format!(
            "Shared instance external file {} has an unexpected size",
            file.file_name
        ))
        .into());
    }

    let mut stream = response.bytes_stream();
    let initial_capacity = usize::try_from(
        file.file_size.min(MAX_SHARED_INSTANCE_INITIAL_BUFFER_SIZE),
    )
    .map_err(|_| {
        crate::ErrorKind::InputError(format!(
            "Shared instance external file {} is too large",
            file.file_name
        ))
    })?;
    let mut bytes = Vec::with_capacity(initial_capacity);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let downloaded_size = (bytes.len() as u64)
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Shared instance external file {} is too large",
                    file.file_name
                ))
            })?;
        if downloaded_size > file.file_size {
            return Err(crate::ErrorKind::OtherError(format!(
                "Shared instance external file {} has an unexpected size",
                file.file_name
            ))
            .into());
        }
        bytes.extend_from_slice(&chunk);
    }

    if bytes.len() as u64 != file.file_size {
        return Err(crate::ErrorKind::OtherError(format!(
            "Shared instance external file {} has an unexpected size",
            file.file_name
        ))
        .into());
    }
    let bytes = bytes::Bytes::from(bytes);

    if file.file_type == CONFIG_BUNDLE_FILE_TYPE {
        return install_shared_instance_config_bundle(
            instance_id,
            bytes,
            state,
        )
        .await;
    }

    let project_type =
        ProjectType::from_name(&file.file_type).ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Unknown shared instance file type {}",
                file.file_type
            ))
        })?;

    crate::state::instances::commands::add_project_bytes(
        instance_id,
        &file.file_name,
        bytes,
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

fn validate_shared_instance_external_file_name(
    file_name: &str,
) -> crate::Result<()> {
    let path = SafeRelativeUtf8UnixPathBuf::try_from(file_name.to_string())
        .map_err(|_| {
            crate::ErrorKind::InputError(format!(
                "Shared instance external file {file_name} has an invalid file name"
            ))
        })?;
    if file_name == "."
        || file_name.contains('/')
        || file_name.contains('\\')
        || path.components().count() != 1
    {
        return Err(crate::ErrorKind::InputError(format!(
            "Shared instance external file {file_name} has an invalid file name"
        ))
        .into());
    }

    Ok(())
}

async fn install_shared_instance_config_bundle(
    instance_id: &str,
    bytes: bytes::Bytes,
    state: &State,
) -> crate::Result<()> {
    let files =
        tokio::task::spawn_blocking(move || read_config_bundle(bytes.as_ref()))
            .await??;
    let metadata = crate::state::instances::commands::get_instance_metadata(
        instance_id,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let config_path = state
        .directories
        .instances_dir()
        .join(metadata.instance.path)
        .join(CONFIG_DIRECTORY);
    crate::util::io::create_dir_all(&config_path).await?;

    for (relative_path, bytes) in files {
        let path = config_path.join(relative_path);
        if let Some(parent) = path.parent() {
            crate::util::io::create_dir_all(parent).await?;
        }
        crate::util::io::write(path, bytes).await?;
    }

    Ok(())
}

fn read_config_bundle(bytes: &[u8]) -> crate::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|error| {
            crate::ErrorKind::InputError(format!(
                "Invalid shared instance config bundle: {error}"
            ))
        })?;
    if archive.len() > MAX_CONFIG_BUNDLE_ENTRIES {
        return Err(crate::ErrorKind::InputError(
            "Shared instance config bundle contains too many entries"
                .to_string(),
        )
        .into());
    }
    let mut files = Vec::new();
    let mut total_size = 0;

    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|error| {
            crate::ErrorKind::InputError(format!(
                "Invalid shared instance config bundle entry: {error}"
            ))
        })?;
        if file.is_dir() {
            continue;
        }
        let path = file.enclosed_name().ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Shared instance config bundle contains an unsafe path"
                    .to_string(),
            )
        })?;
        if is_excluded_config_path(&path) {
            continue;
        }
        if !is_supported_config_file(&path) {
            return Err(crate::ErrorKind::InputError(format!(
                "Shared instance config bundle contains unsupported file {}",
                path.display()
            ))
            .into());
        }
        let declared_size = file.size();
        let bytes = read_bounded_config_bundle_entry(
            file,
            declared_size,
            &mut total_size,
        )?;
        files.push((path.to_path_buf(), bytes));
    }

    Ok(files)
}

fn is_supported_config_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            CONFIG_FILE_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}
