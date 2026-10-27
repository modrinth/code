use super::client::*;
use super::diff::*;
use super::install::*;
use super::report::{PublishReport, PublishReportFile};
use super::types::*;
use super::*;
use async_walkdir::{Filtering, WalkDir};
use futures::{FutureExt, StreamExt, TryStreamExt};
use sha2::Digest;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Set by the shared instances backend when it rejects an upload by file type.
const FILE_NOT_ALLOWED_HEADER: &str = "x-file-not-allowed";
const UPLOAD_PROGRESS_INTERVAL: Duration = Duration::from_millis(200);
const UPLOAD_CONCURRENCY: usize = 4;
const UPLOAD_BATCH_MAX_FILES: usize = 256;
const UPLOAD_BATCH_MIN_BYTES: u64 = 16 * 1024 * 1024;
/// Total file size the shared instances backend accepts in one batch.
const UPLOAD_BATCH_MAX_BYTES: u64 = 450 * 1024 * 1024;
const UPLOAD_BATCH_ATTEMPTS: usize = 2;

#[tracing::instrument]
pub async fn unpublish_shared_instance(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(());
    };
    ensure_owner(&attachment)?;

    delete_remote_instance(&attachment.id, &state).await?;
    detach_local_shared_instance(instance_id, metadata.link, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn unlink_shared_instance(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(());
    };
    ensure_member(&attachment)?;

    detach_local_shared_instance(instance_id, metadata.link, &state).await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

pub(super) async fn detach_local_shared_instance(
    instance_id: &str,
    link: InstanceLink,
    state: &State,
) -> crate::Result<()> {
    let link_patch = match link {
        InstanceLink::SharedInstance {
            modpack_project_id: Some(project_id),
            modpack_version_id: Some(version_id),
        } => Some((
            InstanceLink::ModrinthModpack {
                project_id,
                version_id,
            },
            ContentSourceKind::ModrinthModpack,
        )),
        InstanceLink::SharedInstance { .. } => {
            Some((InstanceLink::Unmanaged, ContentSourceKind::Local))
        }
        _ => None,
    };

    if let Some((link, source_kind)) = link_patch {
        crate::state::edit_instance(
            instance_id,
            EditInstance {
                link: Some(link),
                content_set_patch: Some(AppliedContentSetPatch {
                    source_kind: Some(source_kind),
                    ..Default::default()
                }),
                ..Default::default()
            },
            state,
        )
        .await?;
    }

    crate::state::clear_shared_instance(instance_id, &state.pool).await?;

    Ok(())
}

/// `on_progress` receives the uploaded and total bytes of the files the
/// shared instances service asks for.
#[tracing::instrument(skip(on_progress))]
pub async fn publish_shared_instance(
    instance_id: &str,
    file_paths: Vec<String>,
    on_progress: &(dyn Fn(u64, u64) + Send + Sync),
) -> crate::Result<SharedInstanceAttachment> {
    let state = State::get().await?;
    publish_shared_instance_inner(
        instance_id,
        &file_paths,
        on_progress,
        &state,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    shared_attachment(instance_id, &state).await?.ok_or(
        crate::ErrorKind::InputError(
            "shared instance attachment was not persisted".to_string(),
        )
        .into(),
    )
}

#[tracing::instrument]
pub async fn get_shared_instance_publish_preview(
    instance_id: &str,
) -> crate::Result<Option<SharedInstancePublishPreview>> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(None);
    };
    ensure_owner(&attachment)?;

    let (version, snapshot) = tokio::try_join!(
        get_latest_remote_version_optional_unavailable(&attachment.id, &state,),
        collect_publish_snapshot(&metadata, &state),
    )?;
    let version = match version {
        SharedInstanceRemoteResponse::Available(version) => version,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            handle_unavailable_shared_instance_if_current_user(
                instance_id,
                &attachment,
                reason,
                &state,
            )
            .await?;
            return Err(shared_instance_unavailable_error(reason));
        }
    };
    let diffs =
        shared_instance_publish_diffs(&metadata, &version, &snapshot, &state)
            .await?;
    set_shared_instance_publish_status(
        instance_id,
        &attachment,
        version.version,
        !diffs.is_empty(),
        &state,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(Some(SharedInstancePublishPreview {
        shared_instance_id: attachment.id,
        latest_version: version.version,
        diffs,
        files: snapshot.shared_files,
    }))
}

pub(super) async fn remote_publish_content(
    version: &InstanceVersionResponse,
    include_modpack_dependencies: bool,
    state: &State,
) -> crate::Result<(Vec<String>, BTreeSet<ExternalFileKey>)> {
    let mut version_ids = version.modrinth_ids.clone();
    if let Some(modpack_id) =
        version.modpack_id.as_deref().filter(|id| !id.is_empty())
    {
        version_ids.retain(|id| id != modpack_id);

        if include_modpack_dependencies {
            extend_shared_modpack_dependencies(
                &mut version_ids,
                modpack_id,
                state,
            )
            .await?;
        }
    }
    dedupe_strings(&mut version_ids);

    Ok((
        version_ids,
        version
            .external_files
            .iter()
            .filter(|file| {
                !matches!(
                    file.file_type.as_str(),
                    CONFIG_BUNDLE_FILE_TYPE | CONFIG_FILE_TYPE
                )
            })
            .map(|file| {
                shared_external_file_key(&file.file_type, &file.file_name)
            })
            .collect::<crate::Result<_>>()?,
    ))
}

pub(super) async fn modpack_dependency_version_ids(
    modpack_id: &str,
    state: &State,
) -> crate::Result<Vec<String>> {
    let modpack_version = CachedEntry::get_version(
        modpack_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or(crate::ErrorKind::InputError(
        "shared instance modpack version was not found".to_string(),
    ))?;

    Ok(modpack_version
        .dependencies
        .into_iter()
        .filter_map(|dependency| dependency.version_id)
        .collect())
}

/// Adds the modpack's dependencies to the list. If a project is already in the
/// list, keep that version instead of the one bundled with the modpack.
async fn extend_shared_modpack_dependencies(
    version_ids: &mut Vec<String>,
    modpack_id: &str,
    state: &State,
) -> crate::Result<()> {
    let dependency_ids =
        modpack_dependency_version_ids(modpack_id, state).await?;
    let (explicit, inherited) = tokio::try_join!(
        shared_versions_by_id(version_ids, false, state),
        shared_versions_by_id(&dependency_ids, false, state),
    )?;
    version_ids.extend(
        inherited
            .into_values()
            .filter(|version| {
                !explicit
                    .values()
                    .any(|installed| installed.project_id == version.project_id)
            })
            .map(|version| version.id),
    );
    Ok(())
}

pub(super) async fn shared_instance_install_modpack(
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Option<crate::install::SharedInstanceInstallModpack>> {
    let Some(modpack_id) =
        version.modpack_id.as_deref().filter(|id| !id.is_empty())
    else {
        return Ok(None);
    };
    let modpack_version = CachedEntry::get_version(
        modpack_id,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or(crate::ErrorKind::InputError(
        "shared instance modpack version was not found".to_string(),
    ))?;
    let project = CachedEntry::get_project(
        &modpack_version.project_id,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(Some(crate::install::SharedInstanceInstallModpack {
        project_id: modpack_version.project_id,
        version_id: modpack_version.id,
        dependency_count: modpack_version.dependencies.len(),
        title: project
            .as_ref()
            .map(|project| project.title.clone())
            .unwrap_or(modpack_version.name),
        icon_url: project.and_then(|project| project.raw_icon_url),
    }))
}

pub(super) fn shared_instance_loader_version(
    loader_version: String,
) -> Option<String> {
    (!loader_version.is_empty()).then_some(loader_version)
}

pub(super) async fn current_shared_content(
    metadata: &crate::state::InstanceMetadata,
    include_linked_modpack_content: bool,
    state: &State,
) -> crate::Result<(Vec<String>, BTreeSet<ExternalFileKey>)> {
    let entries =
        crate::state::instances::adapters::sqlite::content_rows::get_content_entries(
            &metadata.applied_content_set.id,
            &state.pool,
        )
        .await?;
    let files = crate::state::instances::adapters::sqlite::content_rows::get_instance_files(
        &metadata.instance.id,
        &state.pool,
    )
    .await?
    .into_iter()
    .map(|file| (file.id.clone(), file))
    .collect::<HashMap<_, _>>();
    let mut version_ids = Vec::new();
    let mut external_files = BTreeSet::new();

    for entry in entries {
        let include_entry = entry.source_kind
            == crate::state::ContentSourceKind::SharedInstance
            || (include_linked_modpack_content
                && entry.source_kind
                    == crate::state::ContentSourceKind::ModrinthModpack);
        if !include_entry {
            continue;
        }

        if let Some(version_id) = entry.version_id {
            version_ids.push(version_id);
            continue;
        }

        let Some(file_id) = entry.file_id else {
            continue;
        };
        if let Some(file) = files.get(&file_id) {
            external_files.insert(ExternalFileKey {
                content_type: entry.project_type.into(),
                path: file.file_name.clone(),
            });
        }
    }
    if include_linked_modpack_content
        && let Some(modpack_id) = shared_modpack_id(&metadata.link)
    {
        extend_shared_modpack_dependencies(
            &mut version_ids,
            &modpack_id,
            state,
        )
        .await?;
    }
    dedupe_strings(&mut version_ids);

    Ok((version_ids, external_files))
}

pub(super) struct CurrentPublishSnapshot {
    pub(super) version_ids: Vec<String>,
    pub(super) external_files: Vec<ExternalFileCandidate>,
    pub(super) disabled_project_ids: HashSet<String>,
    pub(super) disabled_version_ids: Vec<String>,
    pub(super) disabled_external_files: BTreeSet<ExternalFileKey>,
    pub(super) shared_files: Vec<SharedInstancePublishFile>,
}

/// Cached metadata can outlive a deleted or hidden version. Only publish version
/// IDs that other members can resolve; retain installed files as uploads.
pub(super) async fn collect_publish_snapshot(
    metadata: &crate::state::InstanceMetadata,
    state: &State,
) -> crate::Result<CurrentPublishSnapshot> {
    let instance_path = state
        .directories
        .instances_dir()
        .join(&metadata.instance.path);
    let (items, shared_files) = if CONFIG_SYNC_ENABLED {
        tokio::try_join!(
            crate::state::list_content(
                &metadata.instance.id,
                None,
                None,
                state,
            ),
            collect_shareable_files(&instance_path),
        )?
    } else {
        (
            crate::state::list_content(
                &metadata.instance.id,
                None,
                None,
                state,
            )
            .await?,
            Vec::new(),
        )
    };
    let installed_version_ids = items
        .iter()
        .filter(|item| item.enabled)
        .filter_map(|item| {
            item.version.as_ref().map(|version| version.id.as_str())
        })
        .collect::<Vec<_>>();
    let available_version_ids = CachedEntry::get_version_many(
        &installed_version_ids,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .into_iter()
    .map(|version| version.id)
    .collect::<HashSet<_>>();
    let modpack_id = shared_modpack_id(&metadata.link);
    let mut version_ids = Vec::new();
    let mut seen_version_ids = HashSet::new();
    let mut external_files = Vec::new();
    let mut seen_external_files = HashSet::new();
    let mut disabled_project_ids = HashSet::new();
    let mut disabled_version_ids = Vec::new();
    let mut seen_disabled_version_ids = HashSet::new();
    let mut disabled_external_files = BTreeSet::new();

    for item in items {
        if item.enabled {
            if let Some(version) = item.version.as_ref()
                && available_version_ids.contains(&version.id)
            {
                if seen_version_ids.insert(version.id.clone()) {
                    version_ids.push(version.id.clone());
                }
                continue;
            }

            if item.file_path.is_empty() {
                continue;
            }

            let file_type = file_type(item.project_type);
            let external_key = format!("{}:{file_type}", item.file_path);
            if seen_external_files.insert(external_key) {
                external_files.push(ExternalFileCandidate {
                    file_name: item.file_name,
                    file_type,
                    source: ExternalFileSource::InstanceFile(item.file_path),
                });
            }
            continue;
        }

        let is_modpack = item.version.as_ref().is_some_and(|version| {
            modpack_id.as_deref() == Some(version.id.as_str())
        });
        if is_modpack {
            continue;
        }

        if item.version.is_none()
            && let Some(project) = item.project.as_ref()
        {
            disabled_project_ids.insert(project.id.clone());
        }

        if let Some(version) = item.version {
            if seen_disabled_version_ids.insert(version.id.clone()) {
                disabled_version_ids.push(version.id);
            }
            continue;
        }

        if item.file_path.is_empty() {
            continue;
        }

        disabled_external_files.insert(ExternalFileKey {
            content_type: item.project_type.into(),
            path: enabled_file_name(&item.file_name),
        });
    }

    Ok(CurrentPublishSnapshot {
        version_ids,
        external_files,
        disabled_project_ids,
        disabled_version_ids,
        disabled_external_files,
        shared_files,
    })
}

pub(super) async fn shared_versions_by_id(
    version_ids: &[String],
    allow_missing: bool,
    state: &State,
) -> crate::Result<HashMap<String, crate::state::Version>> {
    let version_id_refs =
        version_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &version_id_refs,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    let fetched_ids = versions
        .iter()
        .map(|version| version.id.as_str())
        .collect::<HashSet<_>>();
    if !allow_missing
        && let Some(missing) = version_ids
            .iter()
            .find(|id| !fetched_ids.contains(id.as_str()))
    {
        bail!(crate::ErrorKind::InputError(format!(
            "shared content version {missing} was not found"
        )));
    }
    Ok(versions
        .into_iter()
        .map(|version| (version.id.clone(), version))
        .collect())
}

pub(super) async fn shared_project_names(
    project_ids: &HashSet<String>,
    state: &State,
) -> crate::Result<HashMap<String, String>> {
    let project_id_refs =
        project_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let projects = CachedEntry::get_project_many(
        &project_id_refs,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(projects
        .into_iter()
        .map(|project| (project.id, project.title))
        .collect())
}

pub(super) fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

pub(super) fn enabled_file_name(file_name: &str) -> String {
    file_name
        .strip_suffix(".disabled")
        .unwrap_or(file_name)
        .to_string()
}

pub(super) fn shared_instance_name(name: String) -> String {
    match name.trim() {
        "" => "Shared instance".to_string(),
        name => name.to_string(),
    }
}

pub(super) async fn set_shared_instance_publish_status(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    latest_version: i32,
    has_changes: bool,
    state: &State,
) -> crate::Result<()> {
    let (status, applied_version) = if has_changes {
        (ContentSetSyncStatus::Stale, attachment.applied_version)
    } else {
        (ContentSetSyncStatus::UpToDate, Some(latest_version))
    };

    crate::state::set_shared_instance_sync_status(
        instance_id,
        status,
        applied_version,
        Some(latest_version),
        &state.pool,
    )
    .await
}

pub(super) async fn publish_shared_instance_inner(
    instance_id: &str,
    file_paths: &[String],
    on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    state: &State,
) -> crate::Result<()> {
    let mut report = PublishReport::new(instance_id, file_paths);
    let result = publish_shared_instance_reported(
        instance_id,
        file_paths,
        on_progress,
        &mut report,
        state,
    )
    .await;
    report.finish(&result);
    result
}

async fn publish_shared_instance_reported(
    instance_id: &str,
    file_paths: &[String],
    on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    report: &mut PublishReport,
    state: &State,
) -> crate::Result<()> {
    let attachment = shared_attachment(instance_id, state).await?.ok_or(
        crate::ErrorKind::InputError(
            "instance is not attached to a shared instance".to_string(),
        ),
    )?;
    report.shared_instance_id = Some(attachment.id.clone());
    report.applied_version = attachment.applied_version;
    report.latest_version = attachment.latest_version;
    ensure_owner(&attachment)?;
    tracing::info!(
        instance_id,
        shared_instance_id = %attachment.id,
        applied_version = attachment.applied_version,
        latest_version = attachment.latest_version,
        "Publishing shared instance content"
    );

    crate::state::set_shared_instance_sync_status(
        instance_id,
        ContentSetSyncStatus::Applying,
        attachment.applied_version,
        attachment.latest_version,
        &state.pool,
    )
    .await?;

    let result = publish_current_content(
        instance_id,
        &attachment.id,
        file_paths,
        on_progress,
        report,
        state,
    )
    .await;

    match result {
        Ok(version) => {
            tracing::info!(
                instance_id,
                shared_instance_id = %attachment.id,
                version,
                "Published shared instance content"
            );
            crate::state::set_shared_instance_sync_status(
                instance_id,
                ContentSetSyncStatus::UpToDate,
                Some(version),
                Some(version),
                &state.pool,
            )
            .await?;
            Ok(())
        }
        Err(error) => {
            tracing::warn!(
                instance_id,
                shared_instance_id = %attachment.id,
                error = %error,
                "Failed to publish shared instance content"
            );
            crate::state::set_shared_instance_sync_status(
                instance_id,
                ContentSetSyncStatus::Error,
                attachment.applied_version,
                attachment.latest_version,
                &state.pool,
            )
            .await?;
            Err(error)
        }
    }
}

pub(super) async fn publish_current_content(
    instance_id: &str,
    shared_instance_id: &str,
    file_paths: &[String],
    on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    report: &mut PublishReport,
    state: &State,
) -> crate::Result<i32> {
    let _store_lease = state.content_store.lease().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    report.instance_name = Some(metadata.instance.name.clone());
    report.game_version =
        Some(metadata.applied_content_set.game_version.clone());
    report.loader =
        Some(metadata.applied_content_set.loader.as_str().to_string());
    report.loader_version = metadata.applied_content_set.loader_version.clone();
    ensure_shareable_link(&metadata.link)?;
    update_remote_instance(
        shared_instance_id,
        shared_instance_name(metadata.instance.name.clone()),
        state,
    )
    .await?;
    report.step("Updated shared instance details");
    let modpack_id = shared_modpack_id(&metadata.link);
    report.modpack_id = modpack_id.clone();
    let snapshot = collect_publish_snapshot(&metadata, state).await?;
    report.step("Collected instance content");
    let shared_files = if CONFIG_SYNC_ENABLED {
        shared_file_candidates(
            &metadata.instance.path,
            &snapshot.shared_files,
            file_paths,
            state,
        )
        .await?
    } else {
        Vec::new()
    };
    let modrinth_ids = snapshot.version_ids;
    let mut external_files = snapshot.external_files;
    external_files.extend(shared_files);
    tracing::debug!(
        instance_id,
        shared_instance_id,
        modpack_id = modpack_id.as_deref().unwrap_or("none"),
        modrinth_id_count = modrinth_ids.len(),
        external_file_count = external_files.len(),
        "Creating shared instance version"
    );
    report.modrinth_ids = modrinth_ids.clone();
    let hashed_files =
        hash_external_files(&metadata.instance.path, &external_files, state)
            .await?;
    report.files = hashed_files
        .iter()
        .map(|file| PublishReportFile {
            file_name: file.file_name.clone(),
            file_type: file.file_type.clone(),
            size: file.size,
            sha512: file.sha512.clone(),
            executable: file.executable,
        })
        .collect();
    report.step(format!("Hashed {} files", hashed_files.len()));
    if let Some(file) = hashed_files.iter().find(|file| file.executable) {
        bail!(crate::ErrorKind::SharedInstanceFileNotAllowed(
            file.file_name.clone()
        ));
    }
    let total_size = hashed_files
        .iter()
        .fold(0_u64, |total, file| total.saturating_add(file.size));
    if total_size > MAX_SHARED_FILES_TOTAL_SIZE {
        bail!(crate::ErrorKind::InputError(format!(
            "files exceed the {} limit for a shared instance version",
            crate::util::io::format_bytes(MAX_SHARED_FILES_TOTAL_SIZE)
        )));
    }
    let external_file_data = hashed_files
        .iter()
        .map(|file| ExternalFileData {
            file_name: file.file_name.clone(),
            file_type: file.file_type.clone(),
        })
        .collect::<Vec<_>>();
    let response = request_json_optional_unavailable::<InstanceVersionResponse>(
        "create_instance_version",
        Method::POST,
        &format!("/instances/{shared_instance_id}/versions"),
        Some(json!({
            "modrinth_ids": modrinth_ids,
            "external_files": external_file_data,
            "modpack_id": modpack_id,
            "game_version": metadata.applied_content_set.game_version.clone(),
            "loader": metadata.applied_content_set.loader.as_str(),
            "loader_version": metadata
                .applied_content_set
                .loader_version
                .clone()
                .unwrap_or_default(),
        })),
        state,
        SharedInstancesRequestAuth::ModrinthSession,
    )
    .await?;
    let response = match response {
        SharedInstanceRemoteResponse::Available(response) => response,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            if let Some(attachment) = metadata.shared_instance.as_ref() {
                handle_unavailable_shared_instance_if_current_user(
                    instance_id,
                    attachment,
                    reason,
                    state,
                )
                .await?;
            }

            return Err(shared_instance_unavailable_error(reason));
        }
    };
    report.created_version = Some(response.version);
    report.requested_files = Some(
        response
            .external_files
            .iter()
            .map(|file| file.file_name.clone())
            .collect(),
    );
    report.step(format!(
        "Created version {} ({} files requested for upload)",
        response.version,
        response.external_files.len()
    ));

    if !response.external_files.is_empty() {
        upload_external_files(
            shared_instance_id,
            response.version,
            &hashed_files,
            &response.external_files,
            on_progress,
            state,
        )
        .await?;
        report.step("Uploaded requested files");
    } else if !response.ready {
        tracing::debug!(
            "Shared instance version {} was not ready but had no external files",
            response.version
        );
    }

    Ok(response.version)
}

pub(super) async fn collect_shareable_files(
    instance_path: &std::path::Path,
) -> crate::Result<Vec<SharedInstancePublishFile>> {
    let mut files = Vec::new();
    let root = instance_path.to_path_buf();
    let mut walker = WalkDir::new(instance_path).filter(move |entry| {
        let root = root.clone();
        async move {
            let is_dir = entry
                .file_type()
                .await
                .is_ok_and(|file_type| file_type.is_dir());
            let entry_path = entry.path();
            let skip_dir = is_dir
                && entry_path.strip_prefix(&root).is_ok_and(|relative| {
                    !is_shareable_directory(
                        &relative.to_string_lossy().replace('\\', "/"),
                    )
                });
            if skip_dir {
                Filtering::IgnoreDir
            } else {
                Filtering::Continue
            }
        }
    });

    while let Some(entry) = walker.next().await {
        let entry = entry.context("failed to read instance directory")?;
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let entry_path = entry.path();
        let relative_path = entry_path.strip_prefix(instance_path)?;
        let path = relative_path.to_string_lossy().replace('\\', "/");
        if !is_shareable_file_path(&path) {
            continue;
        }
        let size = entry.metadata().await?.len();
        let blocked =
            is_executable_content(&read_file_head(&entry_path).await?);
        files.push(SharedInstancePublishFile {
            path,
            size,
            blocked,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

async fn read_file_head(path: &std::path::Path) -> crate::Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;

    let mut head = Vec::with_capacity(FILE_SCAN_HEAD_SIZE);
    tokio::fs::File::open(path)
        .await?
        .take(FILE_SCAN_HEAD_SIZE as u64)
        .read_to_end(&mut head)
        .await?;
    Ok(head)
}

/// Each selected file is uploaded on its own under its instance-relative
/// path, matching how linked servers publish files.
async fn shared_file_candidates(
    instance_path: &str,
    local_files: &[SharedInstancePublishFile],
    selected_paths: &[String],
    state: &State,
) -> crate::Result<Vec<ExternalFileCandidate>> {
    let local_files = local_files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<HashMap<_, _>>();
    let selected_paths = selected_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for path in &selected_paths {
        match local_files.get(path) {
            None => bail!(crate::ErrorKind::InputError(format!(
                "file {path} is unavailable for sharing"
            ))),
            Some(file) if file.blocked => {
                bail!(crate::ErrorKind::SharedInstanceFileNotAllowed(
                    path.to_string()
                ))
            }
            Some(_) => {}
        }
    }
    ensure_shared_file_count(&selected_paths)?;

    let instance_path = state.directories.instances_dir().join(instance_path);
    let mut total_size = 0_u64;
    let mut candidates = Vec::with_capacity(selected_paths.len());
    for path in selected_paths {
        let source = instance_path.join(path);
        let size = crate::util::io::metadata(&source).await?.len();
        total_size = total_size.saturating_add(size);
        if total_size > MAX_SHARED_FILES_TOTAL_SIZE {
            bail!(crate::ErrorKind::InputError(format!(
                "selected files exceed the {} limit for a shared instance version",
                crate::util::io::format_bytes(MAX_SHARED_FILES_TOTAL_SIZE)
            )));
        }
        candidates.push(ExternalFileCandidate {
            file_name: path.to_string(),
            file_type: CONFIG_FILE_TYPE.to_string(),
            source: ExternalFileSource::SharedFile(source),
        });
    }

    Ok(candidates)
}

fn ensure_shared_file_count(paths: &BTreeSet<&str>) -> crate::Result<()> {
    if paths.len() <= MAX_SHARED_FILES {
        return Ok(());
    }

    let mut folder_entry_counts = HashMap::new();
    for path in paths {
        if let Some((folder, _)) = path.split_once('/') {
            *folder_entry_counts.entry(folder).or_insert(0_usize) += 1;
        }
    }

    if let Some((folder, count)) = folder_entry_counts
        .into_iter()
        .filter(|(_, count)| *count > MAX_SHARED_FILES)
        .max_by_key(|(_, count)| *count)
    {
        bail!(crate::ErrorKind::InputError(format!(
            "the \"{folder}\" folder has too many files to share ({count}; maximum {MAX_SHARED_FILES}); select fewer files from this folder"
        )));
    }

    bail!(crate::ErrorKind::InputError(format!(
        "too many files were selected to share ({}; maximum {MAX_SHARED_FILES}); select fewer files",
        paths.len()
    )))
}

pub(super) fn shared_modpack_id(link: &InstanceLink) -> Option<String> {
    match link {
        InstanceLink::ModrinthModpack { version_id, .. } => {
            Some(version_id.clone())
        }
        InstanceLink::ServerProjectModpack {
            content_version_id, ..
        } => Some(content_version_id.clone()),
        InstanceLink::SharedInstance {
            modpack_version_id: Some(version_id),
            ..
        } => Some(version_id.clone()),
        _ => None,
    }
}

pub(super) fn ensure_shareable_link(link: &InstanceLink) -> crate::Result<()> {
    if matches!(link, InstanceLink::ImportedModpack { .. }) {
        bail!(crate::ErrorKind::InputError(
            "you must unlink this modpack to share your instance".to_string(),
        ));
    }

    Ok(())
}

/// A publish candidate with its content, size and SHA-512.
pub(super) struct HashedExternalFile {
    file_name: String,
    file_type: String,
    content: crate::state::content_store::ReadableContent,
    size: u64,
    sha512: String,
    executable: bool,
}

async fn hash_external_files(
    instance_path: &str,
    candidates: &[ExternalFileCandidate],
    state: &State,
) -> crate::Result<Vec<HashedExternalFile>> {
    let hashes = candidates
        .iter()
        .map(|candidate| hash_external_file(instance_path, candidate, state))
        .collect::<Vec<_>>();
    futures::stream::iter(hashes)
        .buffered(UPLOAD_CONCURRENCY)
        .try_collect()
        .await
}

async fn hash_external_file(
    instance_path: &str,
    candidate: &ExternalFileCandidate,
    state: &State,
) -> crate::Result<HashedExternalFile> {
    use tokio::io::AsyncReadExt;

    let content = external_file_source(instance_path, candidate, state).await?;
    let mut file = tokio::fs::File::open(content.path()).await?;
    let mut hasher = sha2::Sha512::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut head = Vec::with_capacity(FILE_SCAN_HEAD_SIZE);
    let mut size = 0_u64;
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        let remaining = FILE_SCAN_HEAD_SIZE - head.len();
        head.extend_from_slice(&buffer[..read.min(remaining)]);
        size += read as u64;
    }

    Ok(HashedExternalFile {
        file_name: candidate.file_name.clone(),
        file_type: candidate.file_type.clone(),
        content,
        size,
        sha512: crate::util::content_hash::encode_hex(
            hasher.finalize().as_slice(),
        ),
        executable: is_executable_content(&head),
    })
}

pub(super) async fn upload_external_files(
    shared_instance_id: &str,
    version: i32,
    files: &[HashedExternalFile],
    uploads: &[ExternalFileResponse],
    on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    state: &State,
) -> crate::Result<()> {
    let mut sources = Vec::with_capacity(uploads.len());
    for upload in uploads {
        let file = files
            .iter()
            .find(|file| {
                file.file_name == upload.file_name
                    && file.file_type == upload.file_type
            })
            .ok_or(crate::ErrorKind::InputError(format!(
                "shared instance service requested unknown external file {}",
                upload.file_name
            )))?;
        sources.push((upload, file));
    }
    sources.sort_by_key(|(_, file)| file.size);
    let total = sources
        .iter()
        .fold(0_u64, |total, (_, file)| total.saturating_add(file.size));

    let (single, batched): (Vec<_>, Vec<_>) = sources
        .into_iter()
        .partition(|(_, file)| file.size > UPLOAD_BATCH_MAX_BYTES);
    let batch_bytes = (total / UPLOAD_CONCURRENCY as u64)
        .clamp(UPLOAD_BATCH_MIN_BYTES, UPLOAD_BATCH_MAX_BYTES);
    let mut batches = Vec::<Vec<&HashedExternalFile>>::new();
    let mut current_bytes = 0_u64;
    for (_, file) in batched {
        match batches.last_mut() {
            Some(batch)
                if batch.len() < UPLOAD_BATCH_MAX_FILES
                    && current_bytes + file.size <= batch_bytes =>
            {
                current_bytes += file.size;
                batch.push(file);
            }
            _ => {
                current_bytes = file.size;
                batches.push(vec![file]);
            }
        }
    }

    let batch_path =
        format!("/instances/{shared_instance_id}/versions/{version}/files");
    let uploaded = Arc::new(AtomicU64::new(0));
    on_progress(0, total);
    let uploads = batches
        .into_iter()
        .map(|batch| {
            upload_external_file_batch(
                &batch_path,
                batch,
                uploaded.clone(),
                state,
            )
            .boxed()
        })
        .chain(single.into_iter().map(|(upload, file)| {
            upload_external_file(upload, file, uploaded.clone(), state).boxed()
        }))
        .collect::<Vec<_>>();
    let uploads = futures::stream::iter(uploads)
        .buffer_unordered(UPLOAD_CONCURRENCY)
        .try_collect::<()>();
    tokio::pin!(uploads);
    let mut ticker = tokio::time::interval(UPLOAD_PROGRESS_INTERVAL);
    loop {
        tokio::select! {
            result = &mut uploads => break result?,
            _ = ticker.tick() => on_progress(
                uploaded.load(Ordering::Relaxed).min(total),
                total,
            ),
        }
    }
    on_progress(total, total);

    Ok(())
}

async fn upload_external_file_batch(
    path: &str,
    files: Vec<&HashedExternalFile>,
    uploaded: Arc<AtomicU64>,
    state: &State,
) -> crate::Result<()> {
    let url = service_url(service_base_url(), path);
    let mut attempt = 1;
    loop {
        let sent = Arc::new(AtomicU64::new(0));
        let result = send_external_file_batch(
            path,
            &url,
            &files,
            uploaded.clone(),
            sent.clone(),
            state,
        )
        .await;
        let retry = match &result {
            Ok(response) => response.status().is_server_error(),
            Err(_) => true,
        };
        if !retry || attempt == UPLOAD_BATCH_ATTEMPTS {
            let response = result?;
            return handle_external_file_batch_response(path, &files, response)
                .await;
        }
        uploaded.fetch_sub(sent.load(Ordering::Relaxed), Ordering::Relaxed);
        tracing::warn!(
            path,
            attempt,
            file_count = files.len(),
            "Retrying shared instance external file batch upload"
        );
        attempt += 1;
    }
}

async fn send_external_file_batch(
    path: &str,
    url: &str,
    files: &[&HashedExternalFile],
    uploaded: Arc<AtomicU64>,
    sent: Arc<AtomicU64>,
    state: &State,
) -> crate::Result<reqwest::Response> {
    use futures::stream::BoxStream;
    use reqwest::header::{CONTENT_ENCODING, CONTENT_TYPE, HeaderValue};

    let boundary = format!("modrinth-{}", uuid::Uuid::new_v4().simple());
    let manifest = serde_json::to_vec(&json!({
        "files": files
            .iter()
            .map(|file| json!({
                "file_name": file.file_name,
                "file_type": file.file_type,
                "file_size": file.size,
                "sha512": file.sha512,
            }))
            .collect::<Vec<_>>(),
    }))?;

    let chunk =
        |data: Vec<u8>| -> BoxStream<'static, std::io::Result<bytes::Bytes>> {
            futures::stream::once(futures::future::ready(Ok(data.into())))
                .boxed()
        };
    let part_header = |name: &str, content_type: &str| {
        chunk(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\nContent-Type: {content_type}\r\n\r\n"
            )
            .into_bytes(),
        )
    };

    let mut parts = vec![
        part_header("manifest", "application/json"),
        chunk(manifest),
        chunk(b"\r\n".to_vec()),
    ];
    for file in files {
        let file_path = file.content.path().to_path_buf();
        let uploaded = uploaded.clone();
        let sent = sent.clone();
        parts.push(part_header("file", "application/octet-stream"));
        parts.push(
            futures::stream::once(tokio::fs::File::open(file_path))
                .map_ok(|reader| {
                    tokio_util::io::ReaderStream::with_capacity(
                        reader,
                        64 * 1024,
                    )
                })
                .try_flatten()
                .inspect_ok(move |read| {
                    let len = read.len() as u64;
                    uploaded.fetch_add(len, Ordering::Relaxed);
                    sent.fetch_add(len, Ordering::Relaxed);
                })
                .boxed(),
        );
        parts.push(chunk(b"\r\n".to_vec()));
    }
    parts.push(chunk(format!("--{boundary}--\r\n").into_bytes()));

    let encoder = async_compression::tokio::bufread::ZstdEncoder::new(
        tokio_util::io::StreamReader::new(
            futures::stream::iter(parts).flatten(),
        ),
    );
    let body = reqwest::Body::wrap_stream(
        tokio_util::io::ReaderStream::with_capacity(encoder, 64 * 1024),
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&format!(
            "multipart/form-data; boundary={boundary}"
        ))
        .context("invalid multipart boundary")?,
    );
    headers.insert(CONTENT_ENCODING, HeaderValue::from_static("zstd"));

    let size = files.iter().map(|file| file.size).sum::<u64>();
    send_body_request_to_url(
        "upload_external_files",
        Method::POST,
        path,
        url,
        body,
        None,
        headers,
        state,
    )
    .await
    .context(format!(
        "uploading {} external files ({size} bytes)",
        files.len()
    ))
}

async fn handle_external_file_batch_response(
    path: &str,
    files: &[&HashedExternalFile],
    response: reqwest::Response,
) -> crate::Result<()> {
    if response.status() == reqwest::StatusCode::BAD_REQUEST
        && response.headers().contains_key(FILE_NOT_ALLOWED_HEADER)
    {
        let message = response.text().await.unwrap_or_default();
        let file_name = files
            .iter()
            .map(|file| file.file_name.as_str())
            .filter(|file_name| message.ends_with(file_name))
            .max_by_key(|file_name| file_name.len())
            .map(str::to_string)
            .unwrap_or(message);
        bail!(crate::ErrorKind::SharedInstanceFileNotAllowed(file_name));
    }
    if !response.status().is_success() {
        return shared_instances_request_error(
            "upload_external_files",
            Method::POST,
            path,
            response,
        )
        .await;
    }

    Ok(())
}

async fn upload_external_file(
    upload: &ExternalFileResponse,
    file: &HashedExternalFile,
    uploaded: Arc<AtomicU64>,
    state: &State,
) -> crate::Result<()> {
    let HashedExternalFile {
        content,
        size,
        sha512: file_sha512,
        ..
    } = file;
    let upload_url = url::Url::parse(&upload.url)
        .context("invalid shared instance external file upload URL")?;
    let reader = tokio::fs::File::open(content.path()).await?;
    let body = reqwest::Body::wrap_stream(
        tokio_util::io::ReaderStream::with_capacity(reader, 64 * 1024)
            .inspect_ok(move |chunk| {
                uploaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
            }),
    );
    let response = send_body_request_to_url(
        "upload_external_file",
        Method::PUT,
        upload_url.path(),
        &upload.url,
        body,
        Some(*size),
        file_sha512_headers(Some(file_sha512))?,
        state,
    )
    .await
    .context(format!(
        "uploading external file {} ({size} bytes, sha512 {file_sha512})",
        upload.file_name
    ))?;
    if response.status() == reqwest::StatusCode::BAD_REQUEST
        && response.headers().contains_key(FILE_NOT_ALLOWED_HEADER)
    {
        bail!(crate::ErrorKind::SharedInstanceFileNotAllowed(
            upload.file_name.clone(),
        ));
    }
    if !response.status().is_success() {
        return shared_instances_request_error(
            "upload_external_file",
            Method::PUT,
            upload_url.path(),
            response,
        )
        .await;
    }

    Ok(())
}

async fn external_file_source(
    instance_path: &str,
    candidate: &ExternalFileCandidate,
    state: &State,
) -> crate::Result<crate::state::content_store::ReadableContent> {
    Ok(match &candidate.source {
        ExternalFileSource::InstanceFile(file_path) => {
            let instance = crate::state::instances::adapters::sqlite::instance_rows::get_instance_by_path(instance_path, &state.pool).await?
				.ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
            let file = crate::state::instances::adapters::sqlite::content_rows::get_instance_file_by_relative_path(&instance.id, file_path, &state.pool).await?
				.ok_or(crate::ErrorKind::InputError("shared content file is not registered".to_string()))?;
            state.content_store.read_path(&file, instance_path).await?
        }
        ExternalFileSource::SharedFile(path) => {
            crate::state::content_store::ReadableContent::Local(path.clone())
        }
    })
}

pub(super) async fn shared_attachment(
    instance_id: &str,
    state: &State,
) -> crate::Result<Option<SharedInstanceAttachment>> {
    Ok(crate::state::get_instance(instance_id, &state.pool)
        .await?
        .and_then(|metadata| metadata.shared_instance))
}

pub(crate) async fn sync_shared_instance_icon(
    instance_id: &str,
    icon_path: Option<&str>,
    state: &State,
) -> crate::Result<()> {
    let Some(attachment) = shared_attachment(instance_id, state).await? else {
        return Ok(());
    };
    if attachment.role != SharedInstanceRole::Owner {
        return Ok(());
    }

    update_remote_instance_icon(&attachment.id, icon_path, state).await
}

pub(super) async fn shared_instance_for_invites(
    instance_id: &str,
    user_count: usize,
    state: &State,
) -> crate::Result<(crate::state::InstanceMetadata, SharedInstanceAttachment)> {
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let attachment = match metadata.shared_instance.clone() {
        Some(attachment) => {
            tracing::debug!(
                instance_id,
                shared_instance_id = %attachment.id,
                role = attachment.role.as_str(),
                user_count,
                "Using existing shared instance attachment for invite"
            );
            attachment
        }
        None => {
            ensure_shareable_link(&metadata.link)?;
            tracing::info!(
                instance_id,
                user_count,
                "Creating shared instance before first invite"
            );
            let remote = create_remote_instance(
                shared_instance_name(metadata.instance.name.clone()),
                state,
            )
            .await?;
            let linked_user_id = linked_modrinth_user_id(state).await?;
            tracing::info!(
                instance_id,
                shared_instance_id = %remote.id,
                "Created remote shared instance"
            );
            crate::state::attach_shared_instance(
                instance_id,
                crate::state::SharedInstanceAttachmentInput {
                    id: remote.id.clone(),
                    role: SharedInstanceRole::Owner,
                    manager_id: None,
                    server_manager_name: None,
                    server_manager_icon_url: None,
                    linked_user_id,
                    status: ContentSetSyncStatus::Unknown,
                    applied_version: None,
                    latest_version: None,
                },
                &state.pool,
            )
            .await?;
            tracing::debug!(
                instance_id,
                shared_instance_id = %remote.id,
                "Attached local instance as shared instance owner"
            );
            publish_shared_instance_inner(instance_id, &[], &|_, _| {}, state)
                .await?;
            shared_attachment(instance_id, state).await?.ok_or(
                crate::ErrorKind::InputError(
                    "shared instance attachment was not persisted".to_string(),
                ),
            )?
        }
    };

    ensure_owner(&attachment)?;
    update_remote_instance_icon(
        &attachment.id,
        metadata.instance.icon_path.as_deref(),
        state,
    )
    .await?;

    Ok((metadata, attachment))
}

pub(super) fn ensure_owner(
    attachment: &SharedInstanceAttachment,
) -> crate::Result<()> {
    if attachment.role == SharedInstanceRole::Owner {
        return Ok(());
    }

    bail!(crate::ErrorKind::InputError(
        "only the owner instance can manage shared instance users".to_string(),
    ))
}

pub(super) fn ensure_member(
    attachment: &SharedInstanceAttachment,
) -> crate::Result<()> {
    if attachment.role.is_member() {
        return Ok(());
    }

    bail!(crate::ErrorKind::InputError(
        "only shared instance members can unlink from shared instances"
            .to_string(),
    ))
}

pub(super) fn file_type(project_type: ProjectType) -> String {
    project_type.get_name().to_string()
}
