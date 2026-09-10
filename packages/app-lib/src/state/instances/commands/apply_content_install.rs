use crate::state::instances::{
    ContentRequirement, ContentSourceKind, Instance, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CacheBehaviour, CachedEntry, Dependency, DependencyType, KnownModrinthFile,
    ModLoader, ProjectType, State, Version, cache_file_hash_metadata,
};
use crate::util::fetch::{self, DownloadMeta, DownloadReason};
use crate::util::io;
use async_trait::async_trait;
use bytes::Bytes;
use modrinth_content_management::{
    ContentMetadataProvider, ContentType, Error as ResolveError,
    ResolutionPreferences, ResolveContentPlan, ResolveContentRequest,
    ResolvedContent,
};
use std::path::{Path, PathBuf};

pub(crate) struct ContentScope {
    pub instance: Instance,
    pub content_set_id: String,
}

pub(crate) struct InstalledContentFile {
    pub relative_path: String,
    pub project_id: Option<String>,
    pub enabled: bool,
}

pub(crate) struct DownloadedProjectVersion {
    pub file_name: String,
    pub file: fetch::DownloadedFile,
    pub project_type: ProjectType,
    pub project_id: String,
    pub version_id: String,
}

pub(crate) struct InstanceInstallProjectRequest {
    pub project_id: String,
    pub version_id: Option<String>,
    pub content_type: ContentType,
    pub selected: ResolutionPreferences,
}

struct CachedEntryContentProvider<'a> {
    state: &'a State,
    cache_behaviour: Option<CacheBehaviour>,
}

#[async_trait]
impl ContentMetadataProvider for CachedEntryContentProvider<'_> {
    async fn get_version(
        &mut self,
        version_id: &str,
    ) -> Result<Option<modrinth_content_management::Version>, ResolveError>
    {
        let version = CachedEntry::get_version(
            version_id,
            self.cache_behaviour,
            &self.state.pool,
            &self.state.api_semaphore,
        )
        .await
        .map_err(resolve_provider_error)?;

        Ok(version.map(version_to_resolver))
    }

    async fn get_project_versions(
        &mut self,
        project_id: &str,
    ) -> Result<Vec<modrinth_content_management::Version>, ResolveError> {
        let versions = CachedEntry::get_project_versions(
            project_id,
            self.cache_behaviour,
            &self.state.pool,
            &self.state.api_semaphore,
        )
        .await
        .map_err(resolve_provider_error)?;

        Ok(versions
            .unwrap_or_default()
            .into_iter()
            .map(version_to_resolver)
            .collect())
    }
}

fn resolve_provider_error(error: crate::Error) -> ResolveError {
    ResolveError::Provider(error.to_string())
}

fn resolver_error(error: ResolveError) -> crate::Error {
    crate::ErrorKind::InputError(error.to_string()).into()
}

fn version_to_resolver(
    version: Version,
) -> modrinth_content_management::Version {
    modrinth_content_management::Version {
        id: version.id,
        project_id: version.project_id,
        date_published: version.date_published,
        dependencies: version
            .dependencies
            .into_iter()
            .map(dependency_to_resolver)
            .collect(),
        game_versions: version.game_versions,
        loaders: version.loaders,
    }
}

fn dependency_to_resolver(
    dependency: Dependency,
) -> modrinth_content_management::Dependency {
    modrinth_content_management::Dependency {
        version_id: dependency.version_id,
        project_id: dependency.project_id,
        file_name: dependency.file_name,
        dependency_type: match dependency.dependency_type {
            DependencyType::Required => {
                modrinth_content_management::DependencyType::Required
            }
            DependencyType::Optional => {
                modrinth_content_management::DependencyType::Optional
            }
            DependencyType::Incompatible => {
                modrinth_content_management::DependencyType::Incompatible
            }
            DependencyType::Embedded => {
                modrinth_content_management::DependencyType::Embedded
            }
        },
    }
}

fn target_preferences(
    game_version: String,
    loader: ModLoader,
    content_type: ContentType,
) -> ResolutionPreferences {
    let loader = match content_type {
        ContentType::DataPack => "datapack".to_string(),
        ContentType::ResourcePack => "minecraft".to_string(),
        ContentType::Shader => "iris".to_string(),
        _ => loader.as_str().to_string(),
    };

    ResolutionPreferences {
        game_versions: vec![game_version],
        loaders: vec![loader],
    }
}

pub(crate) async fn resolve_install_plan(
    instance_id: &str,
    request: InstanceInstallProjectRequest,
    state: &State,
) -> crate::Result<ResolveContentPlan> {
    let content_set =
        content_rows::get_applied_content_set(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Instance {instance_id} has no applied content set"
                ))
            })?;
    let existing_project_ids =
        crate::state::get_installed_project_ids_for_instance(
            instance_id,
            None,
            state,
        )
        .await?;
    let provider = CachedEntryContentProvider {
        state,
        cache_behaviour: Some(CacheBehaviour::MustRevalidate),
    };
    let content_type = request.content_type;
    let request = ResolveContentRequest {
        project_id: request.project_id,
        version_id: request.version_id,
        content_type,
        selected: request.selected,
        target: target_preferences(
            content_set.game_version,
            content_set.loader,
            content_type,
        ),
        existing_project_ids,
    };

    modrinth_content_management::resolve_content(provider, request)
        .await
        .map_err(resolver_error)
}

pub(crate) async fn install_resolved_content_plan(
    instance_id: &str,
    plan: &ResolveContentPlan,
    state: &State,
) -> crate::Result<Vec<String>> {
    let mut paths = vec![
        add_resolved_content(
            instance_id,
            &plan.primary,
            DownloadReason::Standalone,
            state,
        )
        .await?,
    ];
    for dependency in &plan.dependencies {
        paths.push(
            add_resolved_content(
                instance_id,
                dependency,
                DownloadReason::Dependency,
                state,
            )
            .await?,
        );
    }

    Ok(paths)
}

pub(crate) async fn switch_project_version_with_dependencies(
    instance_id: &str,
    project_path: &str,
    version_id: &str,
    state: &State,
) -> crate::Result<String> {
    let version = CachedEntry::get_version(
        version_id,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unable to install version id {version_id}. Not found."
        ))
    })?;
    let content_type = ProjectType::get_from_loaders(version.loaders.clone())
        .map(ContentType::from)
        .unwrap_or(ContentType::Mod);
    let plan = resolve_install_plan(
        instance_id,
        InstanceInstallProjectRequest {
            project_id: version.project_id,
            version_id: Some(version_id.to_string()),
            content_type,
            selected: ResolutionPreferences::default(),
        },
        state,
    )
    .await?;

    let was_disabled = project_path.ends_with(".disabled");
    let mut new_path = add_project_from_version(
        instance_id,
        &plan.primary.version_id,
        DownloadReason::Update,
        None,
        ContentSourceKind::Local,
        state,
    )
    .await?;

    if was_disabled {
        new_path =
            toggle_disable_project(instance_id, &new_path, Some(false), state)
                .await?;
    }

    for dependency in &plan.dependencies {
        add_resolved_content(
            instance_id,
            dependency,
            DownloadReason::Dependency,
            state,
        )
        .await?;
    }

    if new_path != project_path {
        rename_project_companion_file(
            instance_id,
            project_path,
            &new_path,
            state,
        )
        .await?;
        remove_project(instance_id, project_path, state).await?;
    }

    Ok(new_path)
}

async fn add_resolved_content(
    instance_id: &str,
    content: &ResolvedContent,
    reason: DownloadReason,
    state: &State,
) -> crate::Result<String> {
    add_project_from_version(
        instance_id,
        &content.version_id,
        reason,
        content.dependent_on_version_id.clone(),
        ContentSourceKind::Local,
        state,
    )
    .await
}

pub(crate) async fn resolve_content_scope(
    instance_id: &str,
    content_set_id: Option<&str>,
    state: &State,
) -> crate::Result<ContentScope> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let content_set_id = match content_set_id {
        Some(id) => id.to_string(),
        None => instance.applied_content_set_id.clone().ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Instance {} has no applied content set",
                instance.id
            ))
        })?,
    };

    Ok(ContentScope {
        instance,
        content_set_id,
    })
}

pub(crate) async fn add_project_from_version(
    instance_id: &str,
    version_id: &str,
    reason: DownloadReason,
    dependent_on_version_id: Option<String>,
    source_kind: ContentSourceKind,
    state: &State,
) -> crate::Result<String> {
    let downloaded = download_project_version(
        instance_id,
        version_id,
        reason,
        dependent_on_version_id,
        state,
    )
    .await?;

    add_downloaded_project_version(instance_id, downloaded, source_kind, state)
        .await
}

pub(crate) async fn download_project_version(
    instance_id: &str,
    version_id: &str,
    reason: DownloadReason,
    dependent_on_version_id: Option<String>,
    state: &State,
) -> crate::Result<DownloadedProjectVersion> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let content_set =
        content_rows::get_content_set(&scope.content_set_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Unknown content set {}",
                    scope.content_set_id
                ))
            })?;
    let version = CachedEntry::get_version(
        version_id,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unable to install version id {version_id}. Not found."
        ))
    })?;
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No files for input version present!".to_string(),
            )
        })?;
    let download_meta = DownloadMeta {
        reason,
        game_version: content_set.game_version,
        loader: content_set.loader.as_str().to_string(),
        dependent_on: dependent_on_version_id,
    };
    let downloaded_file = fetch::fetch_content_file(
        state,
        &[&file.url],
        file.hashes.get("sha512").map(String::as_str),
        file.hashes.get("sha1").map(|hash| hash.as_str()),
        Some(u64::from(file.size)),
        Some(&download_meta),
        None,
    )
    .await?;
    let project_type = ProjectType::get_from_loaders(version.loaders.clone())
        .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unable to infer project type for version {version_id}"
        ))
    })?;
    let project_id = version.project_id.clone();
    let version_id = version.id.clone();

    Ok(DownloadedProjectVersion {
        file_name: file.filename.clone(),
        file: downloaded_file,
        project_type,
        project_id,
        version_id,
    })
}

pub(crate) async fn add_downloaded_project_version(
    instance_id: &str,
    downloaded: DownloadedProjectVersion,
    source_kind: ContentSourceKind,
    state: &State,
) -> crate::Result<String> {
    add_downloaded_project_version_with_enabled(
        instance_id,
        downloaded,
        source_kind,
        None,
        None,
        state,
    )
    .await
}

pub(crate) async fn add_downloaded_project_version_with_enabled(
    instance_id: &str,
    downloaded: DownloadedProjectVersion,
    source_kind: ContentSourceKind,
    enabled: Option<bool>,
    replace_path: Option<&str>,
    state: &State,
) -> crate::Result<String> {
    if !path_util::is_safe_file_name(&downloaded.file_name) {
        return Err(crate::state::content_store::input(
            "Invalid project filename",
        ));
    }
    let blob = downloaded.file.store_blob(state).await?;
    install_content_blob_inner(
        instance_id,
        &format!(
            "{}/{}",
            downloaded.project_type.get_folder(),
            downloaded.file_name
        ),
        &blob,
        downloaded.project_type,
        source_kind,
        Some(&downloaded.project_id),
        Some(&downloaded.version_id),
        enabled,
        replace_path,
        state,
    )
    .await
}

pub(crate) async fn add_project_from_path(
    instance_id: &str,
    path: &Path,
    project_type: Option<ProjectType>,
    state: &State,
) -> crate::Result<String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            crate::state::content_store::input("Invalid project filename")
        })?;
    if !path_util::is_safe_file_name(file_name) {
        return Err(crate::state::content_store::input(
            "Invalid project filename",
        ));
    }
    let project_type = match project_type {
        Some(project_type) => project_type,
        None => {
            super::embedded_content_metadata::infer_project_type_path(path)
                .await?
        }
    };
    let blob = state.content_store.ingest_local_file(path, state).await?;
    install_content_blob(
        instance_id,
        &format!("{}/{}", project_type.get_folder(), file_name),
        &blob,
        project_type,
        ContentSourceKind::Local,
        None,
        None,
        None,
        state,
    )
    .await
}

pub(crate) async fn add_project_bytes(
    instance_id: &str,
    file_name: &str,
    bytes: Bytes,
    hash: Option<&str>,
    project_type: Option<ProjectType>,
    source_kind: ContentSourceKind,
    project_id: Option<&str>,
    version_id: Option<&str>,
    state: &State,
) -> crate::Result<String> {
    if !path_util::is_safe_file_name(file_name) {
        return Err(crate::state::content_store::input(
            "Invalid project filename",
        ));
    }
    let project_type = match project_type {
        Some(project_type) => project_type,
        None => {
            super::embedded_content_metadata::infer_project_type_bytes(&bytes)?
        }
    };
    if let Some(expected) = hash {
        crate::state::content_store::validate_digest(expected, 40)?;
        if sha1_smol::Sha1::from(&bytes[..]).hexdigest() != expected {
            return Err(crate::state::content_store::input(
                "Content bytes do not match the expected hash",
            ));
        }
    }
    let temporary = state.content_store.temporary().await?;
    tokio::fs::write(&temporary, &bytes).await?;
    let blob = state
        .content_store
        .ingest_local_file(&temporary, state)
        .await?;
    install_content_blob(
        instance_id,
        &format!("{}/{}", project_type.get_folder(), file_name),
        &blob,
        project_type,
        source_kind,
        project_id,
        version_id,
        None,
        state,
    )
    .await
}

pub(crate) async fn install_content_blob(
    instance_id: &str,
    requested_path: &str,
    blob: &crate::state::content_store::BlobLease,
    project_type: ProjectType,
    source_kind: ContentSourceKind,
    project_id: Option<&str>,
    version_id: Option<&str>,
    enabled_override: Option<bool>,
    state: &State,
) -> crate::Result<String> {
    install_content_blob_inner(
        instance_id,
        requested_path,
        blob,
        project_type,
        source_kind,
        project_id,
        version_id,
        enabled_override,
        None,
        state,
    )
    .await
}

async fn install_content_blob_inner(
    instance_id: &str,
    requested_path: &str,
    blob: &crate::state::content_store::BlobLease,
    project_type: ProjectType,
    source_kind: ContentSourceKind,
    project_id: Option<&str>,
    version_id: Option<&str>,
    enabled_override: Option<bool>,
    previous_path: Option<&str>,
    state: &State,
) -> crate::Result<String> {
    let _content_lock = state.lock_instance_content(instance_id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    let scope = resolve_content_scope(instance_id, None, state).await?;
    require_stopped_for_content(instance_id, project_type, state).await?;
    let relative_path = requested_path.trim_end_matches(".disabled");
    if !crate::state::content_store::eligible(relative_path) {
        return Err(crate::state::content_store::input(
            "Unsupported content destination",
        ));
    }
    let existing = content_rows::get_instance_file_by_relative_path(
        instance_id,
        previous_path.unwrap_or(relative_path),
        &state.pool,
    )
    .await?;
    if let Some(previous) = previous_path {
        if existing.is_none() {
            return Err(crate::state::content_store::input(
                "Content changed while its update was downloading; refresh and try again",
            ));
        }
        if previous != relative_path
            && content_rows::get_instance_file_by_relative_path(
                instance_id,
                relative_path,
                &state.pool,
            )
            .await?
            .is_some()
        {
            return Err(crate::state::content_store::input(
                "The updated filename belongs to another content item",
            ));
        }
    }
    let enabled = if previous_path.is_some() {
        existing.as_ref().is_some_and(|file| file.enabled)
    } else {
        enabled_override.unwrap_or_else(|| {
            !requested_path.ends_with(".disabled")
                && existing.as_ref().is_none_or(|file| file.enabled)
        })
    };
    let source_path =
        instance_full_path(state, &scope.instance).join(requested_path);
    let legacy_path = previous_path
        .filter(|path| *path != relative_path)
        .or_else(|| {
            (requested_path != relative_path && source_path.exists())
                .then_some(requested_path)
        });
    let mut operation = state
        .content_store
        .prepare(
            &scope.instance,
            relative_path,
            Some(blob),
            enabled,
            legacy_path,
        )
        .await?;
    operation.apply(&state.content_store).await?;
    let result = async {
        let file_name = Path::new(relative_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                crate::state::content_store::input("Invalid content filename")
            })?;
        let mut tx = state.pool.begin().await?;
        if let Some(legacy_path) = legacy_path {
            content_rows::rename_instance_file(
                instance_id,
                legacy_path,
                relative_path,
                file_name,
                enabled,
                &mut tx,
            )
            .await?;
        }
        let file = content_rows::upsert_instance_file_from_parts(
            content_rows::UpsertInstanceFile {
                instance_id,
                relative_path,
                file_name,
                enabled,
                sha1: &blob.blob.sha1,
                size: blob.blob.size as u64,
                missing: false,
            },
            &mut tx,
        )
        .await?;
        upsert_entry_for_file(
            &scope,
            &file,
            project_type,
            project_id,
            version_id,
            source_kind,
            &mut tx,
        )
        .await?;
        operation.commit(&mut tx, Some(&file.id)).await?;
        tx.commit().await?;
        Ok::<(), crate::Error>(())
    }
    .await;
    if result.is_err() {
        operation.rollback(&state.content_store).await?;
    }
    result?;
    cache_file_hash_metadata(
        instance_id,
        relative_path,
        blob.blob.size as u64,
        blob.blob.modified_at_ns as u64,
        blob.blob.sha1.clone(),
        Some(project_type),
        project_id.zip(version_id).map(|(project_id, version_id)| {
            KnownModrinthFile {
                project_id,
                version_id,
            }
        }),
        &state.pool,
    )
    .await?;
    super::mark_shared_instance_stale(instance_id, &state.pool).await?;
    Ok(relative_path.to_string())
}

pub(crate) async fn record_project_file(
    instance_id: &str,
    relative_path: &str,
    sha1: &str,
    size: u64,
    project_type: ProjectType,
    source_kind: ContentSourceKind,
    project_id: Option<&str>,
    version_id: Option<&str>,
    state: &State,
) -> crate::Result<()> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let path = state
        .content_store
        .instance_path(&scope.instance.path, relative_path)
        .await?;
    let blob = state.content_store.ingest_file(&path).await?;
    if blob.blob.sha1 != sha1 || blob.blob.size as u64 != size {
        return Err(crate::state::content_store::input(
            "Installed content does not match its expected hash or size",
        ));
    }
    install_content_blob(
        instance_id,
        relative_path,
        &blob,
        project_type,
        source_kind,
        project_id,
        version_id,
        None,
        state,
    )
    .await?;
    Ok(())
}

pub(crate) async fn require_stopped_for_content(
    instance_id: &str,
    project_type: ProjectType,
    state: &State,
) -> crate::Result<()> {
    if project_type == ProjectType::Mod
        && crate::state::instance_has_running_process(instance_id, state)
            .await?
    {
        return Err(crate::state::content_store::input(
            "Stop this instance before changing its mods",
        ));
    }
    Ok(())
}

pub(crate) async fn toggle_disable_project(
    instance_id: &str,
    project_path: &str,
    desired_enabled: Option<bool>,
    state: &State,
) -> crate::Result<String> {
    let _content_lock = state.lock_instance_content(instance_id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let canonical_path = project_path.trim_end_matches(".disabled");
    let file = content_rows::get_instance_file_by_relative_path(
        instance_id,
        canonical_path,
        &state.pool,
    )
    .await?
    .ok_or_else(|| {
        crate::state::content_store::input(
            "Content file is not registered; refresh the Content tab first",
        )
    })?;
    let project_type = super::sync_content_files::project_type_for_file(&file)
        .ok_or_else(|| {
            crate::state::content_store::input("Unsupported content type")
        })?;
    require_stopped_for_content(instance_id, project_type, state).await?;
    let enabled = desired_enabled.unwrap_or(!file.enabled);
    let blob = match state.content_store.file_blob(&file).await? {
        Some(blob) => blob,
        None => {
            if let Some(binding) =
                crate::state::content_store::catalog::binding(
                    &state.pool,
                    &file.id,
                )
                .await?
            {
                if !enabled {
                    state
                        .content_store
                        .catalog_blob(&binding.blob_sha512)
                        .await?
                        .ok_or_else(|| {
                            crate::state::content_store::input(
                                "Missing content catalog record",
                            )
                        })?
                } else {
                    return Err(crate::state::content_store::input(
                        "Content needs repair or re-import before it can be enabled",
                    ));
                }
            } else {
                let path = state
                    .content_store
                    .instance_path(&scope.instance.path, canonical_path)
                    .await?;
                if tokio::fs::symlink_metadata(&path)
                    .await?
                    .file_type()
                    .is_symlink()
                {
                    return Err(crate::state::content_store::input(
                        "Cannot adopt an external symlink",
                    ));
                }
                state.content_store.ingest_file(&path).await?
            }
        }
    };
    let mut operation = state
        .content_store
        .prepare(&scope.instance, canonical_path, Some(&blob), enabled, None)
        .await?;
    operation.apply(&state.content_store).await?;
    let result = async {
        let mut updated = file.clone();
        updated.enabled = enabled;
        updated.missing = false;
        updated.modified_at = chrono::Utc::now();
        let mut tx = state.pool.begin().await?;
        content_rows::upsert_instance_file(&updated, &mut tx).await?;
        content_rows::set_content_entry_enabled_for_file(
            &scope.content_set_id,
            &file.id,
            enabled,
            &mut tx,
        )
        .await?;
        operation.commit(&mut tx, Some(&file.id)).await?;
        tx.commit().await?;
        Ok::<(), crate::Error>(())
    }
    .await;
    if result.is_err() {
        operation.rollback(&state.content_store).await?;
    }
    result?;
    super::mark_shared_instance_stale(instance_id, &state.pool).await?;
    Ok(canonical_path.to_string())
}

pub(crate) async fn remove_project(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<()> {
    let _content_lock = state.lock_instance_content(instance_id).await;
    let _store_lock = state.content_store.files_lock.lock().await;
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let relative_path = project_path.trim_end_matches(".disabled");
    let project_type = ProjectType::get_from_parent_folder(relative_path)
        .ok_or_else(|| {
            crate::state::content_store::input("Unsupported content type")
        })?;
    require_stopped_for_content(instance_id, project_type, state).await?;
    let file = content_rows::get_instance_file_by_relative_path(
        instance_id,
        relative_path,
        &state.pool,
    )
    .await?;
    let mut operation = state
        .content_store
        .prepare(&scope.instance, relative_path, None, false, None)
        .await?;
    operation.apply(&state.content_store).await?;
    let result = async {
        let mut tx = state.pool.begin().await?;
        if let Some(file) = &file {
            content_rows::remove_content_entries_for_file(
                &scope.content_set_id,
                &file.id,
                &mut tx,
            )
            .await?;
            content_rows::remove_instance_file_by_relative_path(
                instance_id,
                relative_path,
                &mut tx,
            )
            .await?;
        }
        operation.commit(&mut tx, None).await?;
        tx.commit().await?;
        Ok::<(), crate::Error>(())
    }
    .await;
    if result.is_err() {
        operation.rollback(&state.content_store).await?;
    }
    result?;
    super::mark_shared_instance_stale(instance_id, &state.pool).await?;
    Ok(())
}

pub(crate) async fn content_source_kind_for_project_path(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<Option<ContentSourceKind>> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let Some(file) = content_rows::get_instance_file_by_relative_path(
        &scope.instance.id,
        project_path,
        &state.pool,
    )
    .await?
    else {
        return Ok(None);
    };
    let entries =
        content_rows::get_content_entries(&scope.content_set_id, &state.pool)
            .await?;

    Ok(entries.into_iter().find_map(|entry| {
        (entry.file_id.as_deref() == Some(file.id.as_str()))
            .then_some(entry.source_kind)
    }))
}

pub(crate) async fn is_project_locked(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<bool> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    content_rows::is_instance_file_locked(
        &scope.instance.id,
        project_path,
        &state.pool,
    )
    .await
}

pub(crate) async fn set_project_locked(
    instance_id: &str,
    project_path: &str,
    locked: bool,
    state: &State,
) -> crate::Result<()> {
    let _content_lock = state.lock_instance_content(instance_id).await;
    let scope = resolve_content_scope(instance_id, None, state).await?;
    content_rows::set_instance_file_locked(
        &scope.instance.id,
        project_path,
        locked,
        &state.pool,
    )
    .await
}

pub(crate) async fn rename_project_companion_file(
    instance_id: &str,
    old_project_path: &str,
    new_project_path: &str,
    state: &State,
) -> crate::Result<()> {
    let project_type = ProjectType::get_from_parent_folder(new_project_path);
    if project_type == Some(ProjectType::ShaderPack) {
        let scope = resolve_content_scope(instance_id, None, state).await?;
        let base = instance_full_path(state, &scope.instance);

        let old_txt_path = base.join(format!(
            "{}.txt",
            old_project_path.trim_end_matches(".disabled")
        ));
        let new_txt_path = base.join(format!(
            "{}.txt",
            new_project_path.trim_end_matches(".disabled")
        ));

        if old_txt_path.exists() {
            if new_txt_path.exists()
                && io::canonicalize(&old_txt_path)?
                    == io::canonicalize(&new_txt_path)?
            {
                return Ok(());
            }

            io::copy(&old_txt_path, &new_txt_path).await?;
            io::remove_file(&old_txt_path).await?;
        }
    }

    Ok(())
}

pub(crate) async fn list_project_files(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<InstalledContentFile>> {
    let scope = resolve_content_scope(instance_id, None, state).await?;
    let entries =
        content_rows::get_content_entries(&scope.content_set_id, &state.pool)
            .await?;
    let files =
        content_rows::get_instance_files(&scope.instance.id, &state.pool)
            .await?
            .into_iter()
            .map(|file| (file.id.clone(), file))
            .collect::<std::collections::HashMap<_, _>>();

    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let file = files.get(entry.file_id.as_ref()?)?;
            Some(InstalledContentFile {
                relative_path: file.relative_path.clone(),
                project_id: entry.project_id,
                enabled: entry.enabled && file.enabled,
            })
        })
        .collect())
}

pub(crate) fn instance_full_path(
    state: &State,
    instance: &Instance,
) -> PathBuf {
    state.directories.instances_dir().join(&instance.path)
}

async fn index_existing_file(
    scope: &ContentScope,
    relative_path: &str,
    state: &State,
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> crate::Result<InstanceFile> {
    let full_path =
        instance_full_path(state, &scope.instance).join(relative_path);
    let (size, sha1) = fetch::sha1_file_async(&full_path).await?;
    let file_name = Path::new(relative_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let project_type = ProjectType::get_from_parent_folder(relative_path)
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Unable to infer project type from {relative_path}"
            ))
        })?;

    let file = content_rows::upsert_instance_file_from_parts(
        content_rows::UpsertInstanceFile {
            instance_id: &scope.instance.id,
            relative_path,
            file_name: &file_name,
            enabled: !relative_path.ends_with(".disabled"),
            sha1: &sha1,
            size,
            missing: false,
        },
        tx,
    )
    .await?;
    upsert_entry_for_file(
        scope,
        &file,
        project_type,
        None,
        None,
        ContentSourceKind::Local,
        tx,
    )
    .await?;

    Ok(file)
}

async fn upsert_entry_for_file(
    scope: &ContentScope,
    file: &InstanceFile,
    project_type: ProjectType,
    project_id: Option<&str>,
    version_id: Option<&str>,
    source_kind: ContentSourceKind,
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> crate::Result<()> {
    content_rows::upsert_content_entry_from_parts(
        content_rows::UpsertContentEntry {
            instance_id: &scope.instance.id,
            content_set_id: &scope.content_set_id,
            file_id: Some(&file.id),
            project_type,
            project_id,
            version_id,
            source_kind,
            server_requirement: ContentRequirement::Required,
            client_requirement: ContentRequirement::Required,
            enabled: file.enabled,
        },
        tx,
    )
    .await?;

    Ok(())
}
