use crate::install::{
    ContentUpdateSelection, InstallErrorContext, InstallPhaseDetails,
    InstallPhaseId, InstallProgress, InstallProgressReporter,
    InstallProgressSecondary,
};
use crate::state::instances::{
    ContentEntry, ContentSet, ContentSourceKind, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CacheBehaviour, CachedEntry, Dependency, DependencyType, State, Version,
};
use crate::util::fetch::DownloadReason;
use futures::stream::{self, StreamExt};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::apply_content_install::{
    DownloadedProjectVersion, add_downloaded_project_version,
    add_downloaded_project_version_with_enabled, download_project_version,
    download_project_version_with_progress, rename_project_companion_file,
};
use super::check_content_updates::{ContentUpdate, check_content_updates};

#[derive(Clone, Debug)]
struct BulkUpdatePlan {
    project_updates: Vec<PlannedProjectUpdate>,
    dependency_additions: Vec<PlannedDependencyInstall>,
}

#[derive(Clone, Debug)]
struct PlannedProjectUpdate {
    project_id: String,
    relative_path: String,
    current_version_id: String,
    update_version_id: String,
    file_size: u64,
}

#[derive(Clone, Debug)]
struct PlannedDependencyInstall {
    version_id: String,
    parent_version_id: String,
    file_size: u64,
}

#[derive(Clone, Debug)]
enum PlannedDownload {
    ProjectUpdate(PlannedProjectUpdate),
    DependencyAddition(PlannedDependencyInstall),
}

impl PlannedDownload {
    fn file_size(&self) -> u64 {
        match self {
            Self::ProjectUpdate(update) => update.file_size,
            Self::DependencyAddition(dependency) => dependency.file_size,
        }
    }
}

enum DownloadedBulkProject {
    ProjectUpdate(PlannedProjectUpdate, DownloadedProjectVersion),
    DependencyAddition(DownloadedProjectVersion),
}

#[derive(Clone, Debug)]
struct InstalledProject {
    relative_path: String,
    project_id: Option<String>,
    version_id: Option<String>,
    source_kind: ContentSourceKind,
    enabled: bool,
}

#[derive(Clone, Debug)]
struct ResolvedDependency {
    project_id: String,
    version_id: String,
    parent_version_id: String,
    file_size: u64,
}

pub(crate) async fn update_project(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<String> {
    let updates = check_content_updates(
        instance_id,
        Some(CacheBehaviour::MustRevalidate),
        state,
    )
    .await?;
    let update = updates
        .into_iter()
        .find(|update| update.relative_path == project_path)
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "This project cannot be updated!".to_string(),
            )
        })?;

    apply_content_update(instance_id, project_path, &update, state).await
}

async fn apply_content_update(
    instance_id: &str,
    project_path: &str,
    update: &ContentUpdate,
    state: &State,
) -> crate::Result<String> {
    let enabled = content_rows::get_instance_file_by_relative_path(
        instance_id,
        project_path,
        &state.pool,
    )
    .await?
    .is_none_or(|file| file.enabled);
    let downloaded = download_project_version(
        instance_id,
        &update.update_version_id,
        DownloadReason::Update,
        Some(update.current_version_id.clone()),
        state,
    )
    .await?;

    validate_update_project(&downloaded, &update.project_id)?;

    let new_path = add_downloaded_project_version_with_enabled(
        instance_id,
        downloaded,
        ContentSourceKind::Local,
        Some(enabled),
        Some(project_path),
        state,
    )
    .await?;

    if new_path != project_path {
        rename_project_companion_file(
            instance_id,
            project_path,
            &new_path,
            state,
        )
        .await?;
    }

    Ok(new_path)
}

pub(crate) async fn update_selected_projects(
    instance_id: &str,
    updates: &[ContentUpdateSelection],
    reporter: InstallProgressReporter,
    state: &State,
) -> crate::Result<()> {
    reporter
        .update(
            InstallPhaseId::ResolvingPack,
            None,
            InstallPhaseDetails::Empty,
        )
        .await?;
    let plan = plan_bulk_update(instance_id, updates, state).await?;
    apply_bulk_update(instance_id, plan, reporter, state).await
}

async fn apply_bulk_update(
    instance_id: &str,
    plan: BulkUpdatePlan,
    reporter: InstallProgressReporter,
    state: &State,
) -> crate::Result<()> {
    let download_total =
        plan.project_updates.len() + plan.dependency_additions.len();
    let downloads =
        download_planned_projects(instance_id, &plan, &reporter, state).await?;

    reporter
        .update(InstallPhaseId::Finalizing, None, InstallPhaseDetails::Empty)
        .await?;
    for (index, download) in downloads.into_iter().enumerate() {
        reporter
            .update(
                InstallPhaseId::Finalizing,
                Some(InstallProgress {
                    current: index as u64,
                    total: download_total as u64,
                    secondary: None,
                }),
                InstallPhaseDetails::Empty,
            )
            .await?;
        match download {
            DownloadedBulkProject::ProjectUpdate(update, downloaded) => {
                let enabled = content_rows::get_instance_file_by_relative_path(
                    instance_id,
                    &update.relative_path,
                    &state.pool,
                )
                .await?
                .is_none_or(|file| file.enabled);
                let new_path = add_downloaded_project_version_with_enabled(
                    instance_id,
                    downloaded,
                    ContentSourceKind::Local,
                    Some(enabled),
                    Some(&update.relative_path),
                    state,
                )
                .await?;

                if new_path != update.relative_path {
                    rename_project_companion_file(
                        instance_id,
                        &update.relative_path,
                        &new_path,
                        state,
                    )
                    .await?;
                }
            }
            DownloadedBulkProject::DependencyAddition(downloaded) => {
                add_downloaded_project_version(
                    instance_id,
                    downloaded,
                    ContentSourceKind::Local,
                    state,
                )
                .await?;
            }
        }
    }

    reporter.clear_context().await?;
    reporter.persist().await?;
    Ok(())
}

const BULK_DOWNLOAD_CONCURRENCY: usize = 4;

struct BulkDownloadProgress {
    bytes: Vec<u64>,
    completed: u64,
    total_bytes: u64,
}

impl BulkDownloadProgress {
    async fn report(
        &self,
        reporter: &InstallProgressReporter,
    ) -> crate::Result<()> {
        reporter
            .update(
                InstallPhaseId::DownloadingContent,
                Some(InstallProgress {
                    current: self.completed,
                    total: self.bytes.len() as u64,
                    secondary: Some(InstallProgressSecondary {
                        current: self.bytes.iter().sum(),
                        total: self.total_bytes,
                    }),
                }),
                InstallPhaseDetails::Empty,
            )
            .await
    }
}

async fn download_planned_projects(
    instance_id: &str,
    plan: &BulkUpdatePlan,
    reporter: &InstallProgressReporter,
    state: &State,
) -> crate::Result<Vec<DownloadedBulkProject>> {
    let planned = plan
        .project_updates
        .iter()
        .cloned()
        .map(PlannedDownload::ProjectUpdate)
        .chain(
            plan.dependency_additions
                .iter()
                .cloned()
                .map(PlannedDownload::DependencyAddition),
        )
        .collect::<Vec<_>>();
    let progress = Arc::new(Mutex::new(BulkDownloadProgress {
        bytes: vec![0; planned.len()],
        completed: 0,
        total_bytes: planned.iter().map(PlannedDownload::file_size).sum(),
    }));
    progress.lock().await.report(reporter).await?;
    let mut downloads = stream::iter(planned.into_iter().enumerate())
        .map(|(index, download)| {
            let progress = progress.clone();
            let reporter = reporter.clone();
            async move {
                let size = download.file_size();
                let (version_id, reason, dependent_on) = match &download {
                    PlannedDownload::ProjectUpdate(update) => (
                        &update.update_version_id,
                        DownloadReason::Update,
                        update.current_version_id.clone(),
                    ),
                    PlannedDownload::DependencyAddition(dependency) => (
                        &dependency.version_id,
                        DownloadReason::Dependency,
                        dependency.parent_version_id.clone(),
                    ),
                };
                let context =
                    InstallErrorContext::new("download content update")
                        .version_id(version_id.clone())
                        .build();
                let progress_callback = progress.clone();
                let reporter_callback = reporter.clone();
                let mut on_progress = move |current: u64,
                                            _total: u64|
                      -> Pin<
                    Box<dyn Future<Output = crate::Result<()>> + Send>,
                > {
                    let progress = progress_callback.clone();
                    let reporter = reporter_callback.clone();
                    Box::pin(async move {
                        let mut progress = progress.lock().await;
                        progress.bytes[index] =
                            progress.bytes[index].max(current.min(size));
                        progress.report(&reporter).await
                    })
                };
                let result = download_project_version_with_progress(
                    instance_id,
                    version_id,
                    reason,
                    Some(dependent_on),
                    state,
                    Some(&mut on_progress),
                )
                .await;
                let downloaded =
                    reporter.preserve_failure_context(context, result).await?;
                let downloaded = match download {
                    PlannedDownload::ProjectUpdate(update) => {
                        validate_update_project(
                            &downloaded,
                            &update.project_id,
                        )?;
                        DownloadedBulkProject::ProjectUpdate(update, downloaded)
                    }
                    PlannedDownload::DependencyAddition(_) => {
                        DownloadedBulkProject::DependencyAddition(downloaded)
                    }
                };
                let mut progress = progress.lock().await;
                progress.bytes[index] = size;
                progress.completed += 1;
                progress.report(&reporter).await?;
                Ok::<_, crate::Error>(downloaded)
            }
        })
        .buffer_unordered(BULK_DOWNLOAD_CONCURRENCY);
    let mut output = Vec::with_capacity(
        plan.project_updates.len() + plan.dependency_additions.len(),
    );
    while let Some(download) = downloads.next().await {
        output.push(download?);
    }
    Ok(output)
}

async fn plan_bulk_update(
    instance_id: &str,
    selections: &[ContentUpdateSelection],
    state: &State,
) -> crate::Result<BulkUpdatePlan> {
    let shared_instance_member =
        is_shared_instance_member(instance_id, state).await?;
    let updateable_paths = bulk_updateable_project_paths(
        instance_id,
        shared_instance_member,
        state,
    )
    .await?;
    let content_set =
        content_rows::get_applied_content_set(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Instance {instance_id} has no applied content set"
                ))
            })?;
    let installed =
        installed_projects(instance_id, &content_set, state).await?;
    let updateable_paths = if shared_instance_member {
        let managed_paths = installed
            .iter()
            .filter(|project| project.source_kind.is_shared_instance_managed())
            .map(|project| project.relative_path.clone())
            .collect::<HashSet<_>>();

        updateable_paths
            .into_iter()
            .filter(|path| !managed_paths.contains(path))
            .collect::<HashSet<_>>()
    } else {
        updateable_paths
    };

    let mut paths = HashSet::new();
    let mut updates = Vec::with_capacity(selections.len());
    for selection in selections {
        if !updateable_paths.contains(&selection.project_path)
            || !paths.insert(&selection.project_path)
        {
            return Err(crate::state::content_store::input(
                "Selected content cannot be updated",
            ));
        }
        let project = installed
            .iter()
            .find(|project| project.relative_path == selection.project_path)
            .ok_or_else(|| {
                crate::state::content_store::input(
                    "Selected content is no longer installed",
                )
            })?;
        let project_id = project.project_id.clone().ok_or_else(|| {
            crate::state::content_store::input(
                "Selected content has no Modrinth project",
            )
        })?;
        let current_version_id =
            project.version_id.clone().ok_or_else(|| {
                crate::state::content_store::input(
                    "Selected content has no Modrinth version",
                )
            })?;
        updates.push(ContentUpdate {
            project_id,
            relative_path: selection.project_path.clone(),
            current_version_id,
            update_version_id: selection.version_id.clone(),
        });
    }
    if updates.is_empty() {
        return Ok(BulkUpdatePlan {
            project_updates: Vec::new(),
            dependency_additions: Vec::new(),
        });
    }

    let installed_by_project = installed
        .iter()
        .filter_map(|project| {
            project
                .project_id
                .as_ref()
                .map(|project_id| (project_id.clone(), project.clone()))
        })
        .collect::<HashMap<_, _>>();
    let updates_by_path = updates
        .iter()
        .map(|update| {
            (
                update.relative_path.clone(),
                update.update_version_id.clone(),
            )
        })
        .collect::<HashMap<_, _>>();
    let version_ids = installed
        .iter()
        .filter(|project| updates_by_path.contains_key(&project.relative_path))
        .filter_map(|project| project.version_id.clone())
        .chain(
            updates
                .iter()
                .map(|update| update.update_version_id.clone()),
        )
        .collect::<HashSet<_>>();
    let version_id_refs =
        version_ids.iter().map(|id| id.as_str()).collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &version_id_refs,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let versions_by_id = versions
        .into_iter()
        .map(|version| (version.id.clone(), version))
        .collect::<HashMap<_, _>>();
    for update in &updates {
        let version = versions_by_id
            .get(&update.update_version_id)
            .ok_or_else(|| {
                crate::state::content_store::input(
                    "Update version no longer exists",
                )
            })?;
        if version.project_id != update.project_id {
            return Err(crate::state::content_store::input(
                "Cannot update content to a different Modrinth project",
            ));
        }
    }
    let planned_versions = installed
        .iter()
        .filter(|project| project.enabled)
        .filter(|project| updates_by_path.contains_key(&project.relative_path))
        .filter_map(|project| {
            let target_version_id = updates_by_path
                .get(&project.relative_path)
                .or(project.version_id.as_ref())?;

            versions_by_id.get(target_version_id).cloned()
        })
        .collect::<Vec<_>>();
    let planned_dependencies =
        dependency_closure(planned_versions, &content_set, state).await?;
    let dependency_additions = planned_dependencies
        .values()
        .filter(|dependency| {
            !installed_by_project.contains_key(&dependency.project_id)
        })
        .map(|dependency| PlannedDependencyInstall {
            version_id: dependency.version_id.clone(),
            parent_version_id: dependency.parent_version_id.clone(),
            file_size: dependency.file_size,
        })
        .collect::<Vec<_>>();
    let project_updates = updates
        .into_iter()
        .map(|update| {
            let version = versions_by_id
                .get(&update.update_version_id)
                .ok_or_else(|| {
                    crate::state::content_store::input(
                        "Update version no longer exists",
                    )
                })?;
            Ok(PlannedProjectUpdate {
                project_id: update.project_id,
                relative_path: update.relative_path,
                current_version_id: update.current_version_id,
                update_version_id: update.update_version_id,
                file_size: selected_file_size(version)?,
            })
        })
        .collect::<crate::Result<Vec<_>>>()?;

    Ok(BulkUpdatePlan {
        project_updates,
        dependency_additions,
    })
}

async fn bulk_updateable_project_paths(
    instance_id: &str,
    shared_instance_member: bool,
    state: &State,
) -> crate::Result<HashSet<String>> {
    let items = super::list_content::list_content(
        instance_id,
        None,
        Some(CacheBehaviour::MustRevalidate),
        state,
    )
    .await?;

    Ok(items
        .into_iter()
        .filter(|item| {
            !item.locked
                && (!shared_instance_member
                    || !item.source_kind.is_some_and(
                        ContentSourceKind::is_shared_instance_managed,
                    ))
        })
        .map(|item| item.file_path)
        .collect())
}

async fn installed_projects(
    instance_id: &str,
    content_set: &ContentSet,
    state: &State,
) -> crate::Result<Vec<InstalledProject>> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let entries =
        content_rows::get_content_entries(&content_set.id, &state.pool).await?;
    let entries_by_file_id = entries
        .iter()
        .filter_map(|entry| {
            entry.file_id.as_deref().map(|file_id| (file_id, entry))
        })
        .collect::<HashMap<_, _>>();
    let files =
        content_rows::get_instance_files(&instance.id, &state.pool).await?;

    Ok(files
        .into_iter()
        .filter_map(|file| {
            let entry = entries_by_file_id.get(file.id.as_str())?;
            installed_project_from_row(&file, entry)
        })
        .collect())
}

fn installed_project_from_row(
    file: &InstanceFile,
    entry: &ContentEntry,
) -> Option<InstalledProject> {
    if entry.project_id.is_none() && entry.version_id.is_none() {
        return None;
    }

    Some(InstalledProject {
        relative_path: file.relative_path.clone(),
        project_id: entry.project_id.clone(),
        version_id: entry.version_id.clone(),
        source_kind: entry.source_kind,
        enabled: entry.enabled && file.enabled,
    })
}

async fn is_shared_instance_member(
    instance_id: &str,
    state: &State,
) -> crate::Result<bool> {
    let Some(metadata) =
        instance_rows::get_instance_metadata_by_id(instance_id, &state.pool)
            .await?
    else {
        return Ok(false);
    };

    Ok(metadata
        .shared_instance
        .is_some_and(|attachment| attachment.role.is_member()))
}

async fn dependency_closure(
    root_versions: Vec<Version>,
    content_set: &ContentSet,
    state: &State,
) -> crate::Result<HashMap<String, ResolvedDependency>> {
    let mut output = HashMap::new();
    let mut stack = root_versions;
    let mut visited_versions = HashSet::new();
    let mut version_cache = HashMap::new();
    let mut project_versions_cache = HashMap::new();

    while let Some(version) = stack.pop() {
        if !visited_versions.insert(version.id.clone()) {
            continue;
        }

        for dependency in &version.dependencies {
            if !is_required_dependency(dependency, content_set) {
                continue;
            }

            let Some(dependency_version) = resolve_dependency_version(
                dependency,
                content_set,
                state,
                &mut version_cache,
                &mut project_versions_cache,
            )
            .await?
            else {
                continue;
            };
            let project_id = dependency
                .project_id
                .clone()
                .unwrap_or_else(|| dependency_version.project_id.clone());
            let file_size = selected_file_size(&dependency_version)?;

            output.entry(project_id.clone()).or_insert_with(|| {
                ResolvedDependency {
                    project_id,
                    version_id: dependency_version.id.clone(),
                    parent_version_id: version.id.clone(),
                    file_size,
                }
            });
            stack.push(dependency_version);
        }
    }

    Ok(output)
}

fn is_required_dependency(
    dependency: &Dependency,
    content_set: &ContentSet,
) -> bool {
    matches!(dependency.dependency_type, DependencyType::Required)
        && !(dependency.project_id.as_deref() == Some("P7dR8mSH")
            && content_set.loader.as_str() == "quilt")
}

async fn resolve_dependency_version(
    dependency: &Dependency,
    content_set: &ContentSet,
    state: &State,
    version_cache: &mut HashMap<String, Option<Version>>,
    project_versions_cache: &mut HashMap<String, Option<Vec<Version>>>,
) -> crate::Result<Option<Version>> {
    if let Some(version_id) = &dependency.version_id {
        return cached_version(version_id, version_cache, state).await;
    }

    let Some(project_id) = &dependency.project_id else {
        return Ok(None);
    };
    let Some(mut versions) =
        cached_project_versions(project_id, project_versions_cache, state)
            .await?
    else {
        return Ok(None);
    };

    versions.sort_by_key(|version| Reverse(version.date_published));

    Ok(find_preferred_dependency_version(&versions, content_set))
}

async fn cached_version(
    version_id: &str,
    version_cache: &mut HashMap<String, Option<Version>>,
    state: &State,
) -> crate::Result<Option<Version>> {
    if !version_cache.contains_key(version_id) {
        let version = CachedEntry::get_version(
            version_id,
            Some(CacheBehaviour::MustRevalidate),
            &state.pool,
            &state.api_semaphore,
        )
        .await?;
        version_cache.insert(version_id.to_string(), version);
    }

    Ok(version_cache.get(version_id).cloned().flatten())
}

async fn cached_project_versions(
    project_id: &str,
    project_versions_cache: &mut HashMap<String, Option<Vec<Version>>>,
    state: &State,
) -> crate::Result<Option<Vec<Version>>> {
    if !project_versions_cache.contains_key(project_id) {
        let versions = CachedEntry::get_project_versions(
            project_id,
            Some(CacheBehaviour::MustRevalidate),
            &state.pool,
            &state.api_semaphore,
        )
        .await?;
        project_versions_cache.insert(project_id.to_string(), versions);
    }

    Ok(project_versions_cache.get(project_id).cloned().flatten())
}

fn find_preferred_dependency_version(
    versions: &[Version],
    content_set: &ContentSet,
) -> Option<Version> {
    versions
        .iter()
        .find(|version| {
            version.game_versions.contains(&content_set.game_version)
                && version
                    .loaders
                    .iter()
                    .any(|loader| loader == content_set.loader.as_str())
        })
        .or_else(|| {
            versions.iter().find(|version| {
                is_dependency_version_compatible(version, content_set)
            })
        })
        .cloned()
}

fn is_dependency_version_compatible(
    version: &Version,
    content_set: &ContentSet,
) -> bool {
    version.game_versions.contains(&content_set.game_version)
        && (version
            .loaders
            .iter()
            .any(|loader| loader == content_set.loader.as_str())
            || version.loaders.iter().any(|loader| loader == "datapack"))
}

fn selected_file_size(version: &Version) -> crate::Result<u64> {
    version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .map(|file| u64::from(file.size))
        .ok_or_else(|| {
            crate::state::content_store::input("Update version has no files")
        })
}

fn validate_update_project(
    downloaded: &DownloadedProjectVersion,
    project_id: &str,
) -> crate::Result<()> {
    if downloaded.project_id != project_id {
        return Err(crate::ErrorKind::InputError(
            "Cannot update content to a different Modrinth project".to_string(),
        )
        .into());
    }
    Ok(())
}
