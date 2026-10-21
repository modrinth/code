use super::events::InstallProgressReporter;
use super::model::{
    InstallJobState, InstallPhaseDetails, InstallPhaseId,
    InstallProgressSecondary, SharedInstanceExternalFileData,
    SharedInstanceInstallData, SharedInstanceInstallModpack,
};
use super::runner::{
    install_pack, modpack_details, update_content_progress,
    update_content_progress_with_bytes, update_progress,
};
use crate::ErrorKind;
use crate::api::instance::{
    CONFIG_BUNDLE_FILE_TYPE, CONFIG_DIRECTORY, CONFIG_FILE_TYPE,
    CONFIG_SYNC_ENABLED, MAX_CONFIG_BUNDLE_ENTRIES,
    MAX_SHARED_FILES_TOTAL_SIZE, is_shareable_file_path,
    is_supported_config_file, read_bounded_config_bundle_entry,
};
use crate::api::pack::install_from::CreatePackLocation;
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{
    CachedEntry, ContentSetSyncStatus, ContentSourceKind, InstanceLink,
    ProjectType, SharedInstanceAttachmentInput, SharedInstanceRole, State,
};
use crate::util::fetch::{DownloadReason, REQWEST_CLIENT};
use anyhow::{Context, bail};
use futures::StreamExt;
use path_util::SafeRelativeUtf8UnixPathBuf;
use sqlx::SqlitePool;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use uuid::Uuid;

const MAX_SHARED_INSTANCE_EXTERNAL_FILE_SIZE: u64 = 500 * 1024 * 1024;
const MAX_SHARED_INSTANCE_INITIAL_BUFFER_SIZE: u64 = 8 * 1024 * 1024;
const SHARED_INSTANCE_CONTENT_DOWNLOAD_CONCURRENCY: usize = 4;
const SHARED_INSTANCE_FILE_DOWNLOAD_CONCURRENCY: usize = 64;
const SHARED_INSTANCE_PROGRESS_INTERVAL: Duration = Duration::from_millis(500);

pub(super) async fn attach_pending_shared_instance(
    instance_id: &str,
    data: &SharedInstanceInstallData,
    pool: &SqlitePool,
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
        pool,
    )
    .await
}

pub(super) async fn finalize_shared_instance_attachment(
    instance_id: &str,
    data: &SharedInstanceInstallData,
    pool: &SqlitePool,
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
        pool,
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
    sha1: String,
    missing: bool,
}

impl CurrentSharedInstanceExternalFile {
    fn matches(&self, desired: &SharedInstanceExternalFileData) -> bool {
        !self.missing
            && desired
                .sha1
                .as_deref()
                .is_some_and(|sha1| self.sha1.eq_ignore_ascii_case(sha1))
    }
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
    config_files: Vec<SharedInstanceExternalFileData>,
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
    config_files: Vec<SharedInstanceExternalFileData>,
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
                !desired.projects.contains_key(&current.version_id)
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
            config_files: desired.config_files,
            ..Default::default()
        };

        let mut additions = desired
            .projects
            .into_values()
            .filter(|desired| {
                !current.projects.contains_key(&desired.version_id)
            })
            .collect::<Vec<_>>();
        while let Some(desired) = additions.pop() {
            let replacements = plan
                .project_removals
                .iter()
                .enumerate()
                .filter(|(_, current)| current.project_id == desired.project_id)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let unambiguous = !additions
                .iter()
                .any(|other| other.project_id == desired.project_id)
                && !plan
                    .project_additions
                    .iter()
                    .any(|other| other.project_id == desired.project_id);
            if replacements.len() == 1 && unambiguous {
                plan.project_updates.push(SharedInstanceProjectUpdate {
                    current: plan.project_removals.remove(replacements[0]),
                    desired,
                });
            } else {
                plan.project_additions.push(desired);
            }
        }

        for desired in desired.external_files.into_values() {
            let key = (desired.file_type, desired.file.file_name.clone());
            match current.external_files.get(&key) {
                Some(current) if current.matches(&desired.file) => {}
                Some(_) => plan.external_updates.push(desired),
                None => plan.external_additions.push(desired),
            }
        }

        Ok(plan)
    }

    fn content_change_count(&self) -> u64 {
        (self.project_updates.len()
            + self.project_additions.len()
            + self.external_updates.len()
            + self.external_additions.len()
            + usize::from(self.config_bundle.is_some())
            + self.config_files.len()) as u64
    }
}

pub(super) fn apply_shared_instance_update<'a>(
    job_id: Uuid,
    job_state: &'a mut InstallJobState,
    state: &'a State,
    instance_id: &'a str,
    data: &'a SharedInstanceInstallData,
) -> impl Future<Output = crate::Result<()>> + Send + 'a {
    Box::pin(apply_shared_instance_update_inner(
        job_id,
        job_state,
        state,
        instance_id,
        data,
    ))
}

async fn apply_shared_instance_update_inner(
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
    .ok_or(ErrorKind::InputError("unknown instance".to_string()))?;
    let plan = SharedInstanceApplyPlan::build(&metadata, data, state).await?;

    if plan.configuration_changed {
        crate::api::instance::prepare_instance_update(instance_id).await?;
        remove_existing_shared_instance_content(instance_id, state).await?;
        apply_shared_instance_content(
            job_id,
            job_state,
            state,
            instance_id,
            data,
        )
        .await?;
        if data.modpack.is_none() {
            if let Err(error) =
                crate::api::instance::capture_game_options_pack_base(
                    instance_id,
                    None,
                )
                .await
            {
                tracing::warn!(
                    "The shared instance was updated, but its local options.txt could not be restored after removing the previous pack: {error}"
                );
            }
            Box::pin(
                crate::api::instance::reconcile_instance_after_pack_update(
                    instance_id,
                ),
            )
            .await?;
        }
        return Ok(());
    }

    update_progress(
        job_id,
        job_state,
        &state.pool,
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
            &state.pool,
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

    install_shared_instance_downloads(
        job_id,
        job_state,
        state,
        instance_id,
        plan.project_updates
            .iter()
            .map(|update| SharedInstanceModInstall {
                version_id: &update.desired.version_id,
                reason: DownloadReason::Update,
                replaces: Some(&update.current),
            })
            .chain(plan.project_additions.iter().map(|project| {
                SharedInstanceModInstall {
                    version_id: &project.version_id,
                    reason: DownloadReason::Standalone,
                    replaces: None,
                }
            }))
            .collect(),
        plan.external_updates
            .iter()
            .chain(&plan.external_additions)
            .map(|file| &file.file)
            .chain(&plan.config_bundle)
            .chain(&plan.config_files)
            .collect(),
        content_change_count,
    )
    .await?;

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

pub(super) fn shared_instance_update_requires_full_apply(
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
                version_id.clone(),
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
                    sha1: file.sha1.clone(),
                    missing: file.missing,
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
        let version =
            versions_by_id.get(version_id).ok_or(ErrorKind::InputError(
                format!("shared instance version {version_id} was not found"),
            ))?;
        content.projects.insert(
            version.id.clone(),
            DesiredSharedInstanceProject {
                project_id: version.project_id.clone(),
                version_id: version.id.clone(),
            },
        );
    }

    for file in &data.external_files {
        if file.file_type == CONFIG_FILE_TYPE {
            if CONFIG_SYNC_ENABLED {
                content.config_files.push(file.clone());
            }
            continue;
        }
        if file.file_type == CONFIG_BUNDLE_FILE_TYPE {
            if CONFIG_SYNC_ENABLED {
                content.config_bundle = Some(file.clone());
            }
            continue;
        }
        let file_type = ProjectType::from_name(&file.file_type).ok_or(
            ErrorKind::InputError(format!(
                "unknown shared instance file type {}",
                file.file_type
            )),
        )?;
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
            bail!(ErrorKind::InputError(format!(
                "shared instance version {id} was not found"
            )));
        }
    }

    Ok(versions_by_id)
}

pub(super) fn apply_shared_instance_content<'a>(
    job_id: Uuid,
    job_state: &'a mut InstallJobState,
    state: &'a State,
    instance_id: &'a str,
    data: &'a SharedInstanceInstallData,
) -> impl Future<Output = crate::Result<()>> + Send + 'a {
    Box::pin(apply_shared_instance_content_inner(
        job_id,
        job_state,
        state,
        instance_id,
        data,
    ))
}

async fn apply_shared_instance_content_inner(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    instance_id: &str,
    data: &SharedInstanceInstallData,
) -> crate::Result<()> {
    update_progress(
        job_id,
        job_state,
        &state.pool,
        InstallPhaseId::PreparingInstance,
        InstallPhaseDetails::Instance {
            name: data.name.clone(),
        },
    )
    .await?;

    if let Some(modpack) = data.modpack.clone() {
        crate::api::instance::edit(
            instance_id,
            crate::state::EditInstance {
                link: Some(shared_instance_link(Some(&modpack))),
                ..Default::default()
            },
        )
        .await?;
        let location = shared_instance_pack_location(modpack);
        update_progress(
            job_id,
            job_state,
            &state.pool,
            InstallPhaseId::ResolvingPack,
            modpack_details(&location),
        )
        .await?;
        install_pack(
            job_id,
            job_state,
            location,
            instance_id.to_string(),
            DownloadReason::Modpack,
            &state.pool,
        )
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
            &state.pool,
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
            .ok_or(ErrorKind::InputError("unknown instance".to_string()))?;
        crate::launcher::install_minecraft_with_reporter(
            &context,
            false,
            Some(InstallProgressReporter::new(
                job_id,
                job_state.clone(),
                state.pool.clone(),
            )),
            state,
        )
        .await?;
    }

    if !data.modrinth_ids.is_empty() || !data.external_files.is_empty() {
        let content_change_count =
            data.modrinth_ids.len() as u64 + data.external_files.len() as u64;
        install_shared_instance_downloads(
            job_id,
            job_state,
            state,
            instance_id,
            data.modrinth_ids
                .iter()
                .map(|version_id| SharedInstanceModInstall {
                    version_id,
                    reason: DownloadReason::Standalone,
                    replaces: None,
                })
                .collect(),
            data.external_files.iter().collect(),
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

pub(super) async fn remove_existing_shared_instance_content(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let metadata = crate::state::instances::commands::get_instance_metadata(
        instance_id,
        &state.pool,
    )
    .await?
    .ok_or(ErrorKind::InputError("unknown instance".to_string()))?;
    let entries = content_rows::get_content_entries(
        &metadata.applied_content_set.id,
        &state.pool,
    )
    .await?;
    let managed_ids = entries
        .into_iter()
        .filter(|entry| entry.source_kind.is_shared_instance_managed())
        .filter_map(|entry| entry.file_id)
        .collect::<HashSet<_>>();
    for file in
        content_rows::get_instance_files(instance_id, &state.pool).await?
    {
        if managed_ids.contains(&file.id) {
            crate::state::instances::commands::remove_project(
                instance_id,
                &file.relative_path,
                state,
            )
            .await?;
        }
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

/// Bytes downloaded across every external file of an install, reported as the
/// secondary progress so the download manager can show size, speed and ETA.
struct DownloadedBytes {
    current: AtomicU64,
    total: u64,
}

impl DownloadedBytes {
    fn add(&self, bytes: u64) {
        self.current.fetch_add(bytes, Ordering::Relaxed);
    }

    fn progress(&self) -> InstallProgressSecondary {
        InstallProgressSecondary {
            current: self.current.load(Ordering::Relaxed).min(self.total),
            total: self.total,
        }
    }
}

struct SharedInstanceModInstall<'a> {
    version_id: &'a str,
    reason: DownloadReason,
    replaces: Option<&'a CurrentSharedInstanceProject>,
}

/// Mods and content files are installed before individual shared files, so
/// those files are checked against the content just installed and override
/// any config bundle entries at the same path. Larger downloads start first so
/// a big download is not left running alone at the end.
async fn install_shared_instance_downloads(
    job_id: Uuid,
    job_state: &mut InstallJobState,
    state: &State,
    instance_id: &str,
    mods: Vec<SharedInstanceModInstall<'_>>,
    files: Vec<&SharedInstanceExternalFileData>,
    total: u64,
) -> crate::Result<()> {
    let version_ids = mods
        .iter()
        .map(|install| install.version_id.to_string())
        .collect::<Vec<_>>();
    let versions = shared_instance_versions_by_id(&version_ids, state).await?;
    let mut mods = mods
        .into_iter()
        .map(|install| {
            let size = versions
                .get(install.version_id)
                .and_then(|version| {
                    version
                        .files
                        .iter()
                        .find(|file| file.primary)
                        .or_else(|| version.files.first())
                })
                .map_or(0, |file| u64::from(file.size));
            (install, size)
        })
        .collect::<Vec<_>>();
    mods.sort_by_key(|(_, size)| Reverse(*size));
    let (mut shared_files, mut content_files): (Vec<_>, Vec<_>) = files
        .into_iter()
        .partition(|file| file.file_type == CONFIG_FILE_TYPE);
    content_files.sort_by_key(|file| Reverse(file.file_size));
    shared_files.sort_by_key(|file| Reverse(file.file_size));
    let bytes = DownloadedBytes {
        current: AtomicU64::new(0),
        total: mods.iter().map(|(_, size)| size).sum::<u64>()
            + content_files
                .iter()
                .chain(&shared_files)
                .map(|file| file.file_size)
                .sum::<u64>(),
    };
    let mut completed = 0;

    report_shared_instance_installs(
        futures::stream::iter(
            mods.into_iter()
                .map(|(install, size)| {
                    install_shared_instance_mod(
                        instance_id,
                        install,
                        size,
                        state,
                        &bytes,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .buffer_unordered(SHARED_INSTANCE_CONTENT_DOWNLOAD_CONCURRENCY),
        job_id,
        job_state,
        &state.pool,
        &mut completed,
        total,
        &bytes,
    )
    .await?;

    report_shared_instance_installs(
        futures::stream::iter(
            content_files
                .into_iter()
                .map(|file| {
                    install_shared_instance_external_file(
                        instance_id,
                        file,
                        state,
                        &bytes,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .buffer_unordered(SHARED_INSTANCE_CONTENT_DOWNLOAD_CONCURRENCY),
        job_id,
        job_state,
        &state.pool,
        &mut completed,
        total,
        &bytes,
    )
    .await?;

    if shared_files.is_empty() {
        return Ok(());
    }
    let writer =
        crate::api::instance::InstanceFileWriter::new(state, instance_id)
            .await?;
    report_shared_instance_installs(
        futures::stream::iter(
            shared_files
                .into_iter()
                .map(|file| {
                    install_shared_instance_file(&writer, file, state, &bytes)
                })
                .collect::<Vec<_>>(),
        )
        .buffer_unordered(SHARED_INSTANCE_FILE_DOWNLOAD_CONCURRENCY),
        job_id,
        job_state,
        &state.pool,
        &mut completed,
        total,
        &bytes,
    )
    .await
}

async fn install_shared_instance_mod(
    instance_id: &str,
    install: SharedInstanceModInstall<'_>,
    size: u64,
    state: &State,
    downloaded: &DownloadedBytes,
) -> crate::Result<()> {
    let counted = AtomicU64::new(0);
    let mut on_progress = |current: u64,
                           _total: u64|
     -> std::pin::Pin<
        Box<dyn Future<Output = crate::Result<()>> + Send>,
    > {
        let previous = counted.fetch_max(current, Ordering::Relaxed);
        downloaded.add(current.saturating_sub(previous));
        Box::pin(std::future::ready(Ok(())))
    };
    let project =
        crate::state::instances::commands::download_project_version_with_progress(
            instance_id,
            install.version_id,
            install.reason,
            install.replaces.map(|current| current.version_id.clone()),
            state,
            Some(&mut on_progress),
        )
        .await?;
    downloaded.add(size.saturating_sub(counted.load(Ordering::Relaxed)));

    let new_path =
        crate::state::instances::commands::add_downloaded_project_version(
            instance_id,
            project,
            ContentSourceKind::SharedInstance,
            state,
        )
        .await?;
    if let Some(current) = install.replaces
        && current.relative_path != new_path
    {
        crate::state::instances::commands::remove_project(
            instance_id,
            &current.relative_path,
            state,
        )
        .await?;
    }
    Ok(())
}

async fn report_shared_instance_installs(
    installs: impl futures::Stream<Item = crate::Result<()>>,
    job_id: Uuid,
    job_state: &mut InstallJobState,
    pool: &SqlitePool,
    completed: &mut u64,
    total: u64,
    bytes: &DownloadedBytes,
) -> crate::Result<()> {
    let mut installs = std::pin::pin!(installs);
    let mut ticker = tokio::time::interval(SHARED_INSTANCE_PROGRESS_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            result = installs.next() => match result {
                Some(result) => {
                    result?;
                    *completed += 1;
                }
                None => break,
            },
            _ = ticker.tick() => {
                update_content_progress_with_bytes(
                    job_id,
                    job_state,
                    pool,
                    *completed,
                    total,
                    Some(bytes.progress()),
                )
                .await?;
            }
        }
    }
    update_content_progress_with_bytes(
        job_id,
        job_state,
        pool,
        *completed,
        total,
        Some(bytes.progress()),
    )
    .await
}

async fn install_shared_instance_external_file(
    instance_id: &str,
    file: &SharedInstanceExternalFileData,
    state: &State,
    downloaded: &DownloadedBytes,
) -> crate::Result<()> {
    if file.file_type == CONFIG_BUNDLE_FILE_TYPE && !CONFIG_SYNC_ENABLED {
        downloaded.add(file.file_size);
        return Ok(());
    }

    validate_shared_instance_external_file_name(&file.file_name)?;

    if file.file_size > MAX_SHARED_INSTANCE_EXTERNAL_FILE_SIZE {
        bail!(ErrorKind::InputError(format!(
            "shared instance external file {} exceeds the maximum size",
            file.file_name
        )));
    }

    let response = REQWEST_CLIENT.get(&file.url).send().await?;

    if !response.status().is_success() {
        bail!(
            "shared instance external file download failed with status {}",
            response.status()
        );
    }

    if response
        .content_length()
        .is_some_and(|content_length| content_length != file.file_size)
    {
        bail!(
            "shared instance external file {} has an unexpected size",
            file.file_name
        );
    }

    let mut stream = response.bytes_stream();
    let initial_capacity = usize::try_from(
        file.file_size.min(MAX_SHARED_INSTANCE_INITIAL_BUFFER_SIZE),
    )
    .context(ErrorKind::InputError(format!(
        "shared instance external file {} is too large",
        file.file_name
    )))?;
    let mut bytes = Vec::with_capacity(initial_capacity);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let downloaded_size = (bytes.len() as u64)
            .checked_add(chunk.len() as u64)
            .ok_or(ErrorKind::InputError(format!(
                "shared instance external file {} is too large",
                file.file_name
            )))?;
        if downloaded_size > file.file_size {
            bail!(
                "shared instance external file {} has an unexpected size",
                file.file_name
            );
        }
        downloaded.add(chunk.len() as u64);
        bytes.extend_from_slice(&chunk);
    }

    if bytes.len() as u64 != file.file_size {
        bail!(
            "shared instance external file {} has an unexpected size",
            file.file_name
        );
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

    let project_type = ProjectType::from_name(&file.file_type).ok_or(
        ErrorKind::InputError(format!(
            "unknown shared instance file type {}",
            file.file_type
        )),
    )?;

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

/// Shared files can be up to the version's size limit, so they are streamed to
/// disk rather than held in memory.
async fn install_shared_instance_file(
    writer: &crate::api::instance::InstanceFileWriter,
    file: &SharedInstanceExternalFileData,
    state: &State,
    downloaded: &DownloadedBytes,
) -> crate::Result<()> {
    if !CONFIG_SYNC_ENABLED {
        downloaded.add(file.file_size);
        return Ok(());
    }
    if !is_shareable_file_path(&file.file_name)
        || file.file_size > MAX_SHARED_FILES_TOTAL_SIZE
    {
        bail!(ErrorKind::InputError(format!(
            "shared instance file {} is unsupported or too large",
            file.file_name
        )));
    }
    if shared_instance_file_is_current(writer, file, state).await? {
        downloaded.add(file.file_size);
        return Ok(());
    }

    let response = REQWEST_CLIENT.get(&file.url).send().await?;
    if !response.status().is_success() {
        bail!(
            "shared instance file download failed with status {}",
            response.status()
        );
    }
    if response
        .content_length()
        .is_some_and(|content_length| content_length != file.file_size)
    {
        bail!(
            "shared instance file {} has an unexpected size",
            file.file_name
        );
    }

    let reader = tokio_util::io::StreamReader::new(
        response.bytes_stream().map(|chunk| {
            let chunk = chunk
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            downloaded.add(chunk.len() as u64);
            Ok::<_, std::io::Error>(chunk)
        }),
    );
    writer
        .write_from_reader(state, &file.file_name, reader, file.file_size)
        .await
}

/// Whether the instance already has this exact shared file, so the download
/// can be skipped. Without a hash from the server it is treated as changed.
async fn shared_instance_file_is_current(
    writer: &crate::api::instance::InstanceFileWriter,
    file: &SharedInstanceExternalFileData,
    state: &State,
) -> crate::Result<bool> {
    let Some(expected_sha1) = file.sha1.as_deref() else {
        return Ok(false);
    };
    let path = writer.destination(state, &file.file_name).await?;
    let is_same_size =
        tokio::fs::symlink_metadata(&path)
            .await
            .is_ok_and(|metadata| {
                metadata.is_file() && metadata.len() == file.file_size
            });
    if !is_same_size {
        return Ok(false);
    }

    let (_, sha1) = crate::util::fetch::sha1_file_async(&path).await?;
    Ok(sha1.eq_ignore_ascii_case(expected_sha1))
}

fn validate_shared_instance_external_file_name(
    file_name: &str,
) -> crate::Result<()> {
    let path = SafeRelativeUtf8UnixPathBuf::try_from(file_name.to_string())
        .context(ErrorKind::InputError(format!(
            "shared instance external file {file_name} has an invalid file name"
        )))?;
    if file_name == "."
        || file_name.contains('/')
        || file_name.contains('\\')
        || path.components().count() != 1
    {
        bail!(ErrorKind::InputError(format!(
            "shared instance external file {file_name} has an invalid file name"
        )));
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
    .ok_or(ErrorKind::InputError("unknown instance".to_string()))?;
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
        .context(ErrorKind::InputError(
            "invalid shared instance config bundle".to_string(),
        ))?;
    if archive.len() > MAX_CONFIG_BUNDLE_ENTRIES {
        bail!(ErrorKind::InputError(
            "shared instance config bundle contains too many entries"
                .to_string(),
        ));
    }
    let mut files = Vec::new();
    let mut total_size = 0;

    for index in 0..archive.len() {
        let file = archive.by_index(index).context(ErrorKind::InputError(
            "invalid shared instance config bundle entry".to_string(),
        ))?;
        if file.is_dir() {
            continue;
        }
        let path = file.enclosed_name().ok_or(ErrorKind::InputError(
            "shared instance config bundle contains an unsafe path".to_string(),
        ))?;
        if !is_supported_config_file(&path) {
            bail!(ErrorKind::InputError(format!(
                "shared instance config bundle contains unsupported file {}",
                path.display()
            )));
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
