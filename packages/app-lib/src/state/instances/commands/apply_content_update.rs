use crate::state::instances::{
    ContentEntry, ContentSet, ContentSourceKind, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CacheBehaviour, CachedEntry, Dependency, DependencyType, State, Version,
};
use crate::util::fetch::DownloadReason;
use futures::stream::{FuturesUnordered, StreamExt};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use super::apply_content_install::{
    DownloadedProjectVersion, add_downloaded_project_version,
    add_project_from_version, download_project_version, remove_project,
    rename_project_companion_file, toggle_disable_project,
};
use super::check_content_updates::{ContentUpdate, check_content_updates};

#[derive(Clone, Debug)]
struct BulkUpdatePlan {
    project_updates: Vec<PlannedProjectUpdate>,
    dependency_additions: Vec<PlannedDependencyInstall>,
}

#[derive(Clone, Debug)]
struct PlannedProjectUpdate {
    relative_path: String,
    current_version_id: String,
    update_version_id: String,
}

#[derive(Clone, Debug)]
struct PlannedDependencyInstall {
    version_id: String,
    parent_version_id: String,
}

#[derive(Clone, Debug)]
enum PlannedDownload {
    ProjectUpdate(PlannedProjectUpdate),
    DependencyAddition(PlannedDependencyInstall),
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
    enabled: bool,
}

#[derive(Clone, Debug)]
struct ResolvedDependency {
    project_id: String,
    version_id: String,
    parent_version_id: String,
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
    let mut new_path = add_project_from_version(
        instance_id,
        &update.update_version_id,
        DownloadReason::Update,
        Some(update.current_version_id.clone()),
        ContentSourceKind::Local,
        state,
    )
    .await?;

    if project_path.ends_with(".disabled") {
        new_path =
            toggle_disable_project(instance_id, &new_path, Some(false), state)
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

pub(crate) async fn update_all_projects(
    instance_id: &str,
    state: &State,
) -> crate::Result<HashMap<String, String>> {
    emit_bulk_update_progress(
        instance_id,
        crate::event::InstanceBulkUpdateProgressStage::ResolvingVersions,
        0,
        0,
    )
    .await?;
    let plan = plan_bulk_update(instance_id, state).await?;
    let download_total =
        plan.project_updates.len() + plan.dependency_additions.len();
    let downloads =
        download_planned_projects(instance_id, &plan, download_total, state)
            .await?;

    let mut changed = HashMap::new();
    emit_bulk_update_progress(
        instance_id,
        crate::event::InstanceBulkUpdateProgressStage::Finishing,
        download_total,
        download_total,
    )
    .await?;
    for download in downloads {
        match download {
            DownloadedBulkProject::ProjectUpdate(update, downloaded) => {
                let mut new_path = add_downloaded_project_version(
                    instance_id,
                    downloaded,
                    ContentSourceKind::Local,
                    state,
                )
                .await?;

                if update.relative_path.ends_with(".disabled") {
                    new_path = toggle_disable_project(
                        instance_id,
                        &new_path,
                        Some(false),
                        state,
                    )
                    .await?;
                }

                if new_path != update.relative_path {
                    rename_project_companion_file(
                        instance_id,
                        &update.relative_path,
                        &new_path,
                        state,
                    )
                    .await?;
                    remove_project(instance_id, &update.relative_path, state)
                        .await?;
                }

                changed.insert(update.relative_path, new_path);
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

    Ok(changed)
}

async fn download_planned_projects(
    instance_id: &str,
    plan: &BulkUpdatePlan,
    total: usize,
    state: &State,
) -> crate::Result<Vec<DownloadedBulkProject>> {
    emit_bulk_update_progress(
        instance_id,
        crate::event::InstanceBulkUpdateProgressStage::Downloading,
        0,
        total,
    )
    .await?;

    let mut downloads = plan
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
        .map(|download| async move {
            match download {
                PlannedDownload::ProjectUpdate(update) => {
                    let downloaded = download_project_version(
                        instance_id,
                        &update.update_version_id,
                        DownloadReason::Update,
                        Some(update.current_version_id.clone()),
                        state,
                    )
                    .await?;

                    Ok::<_, crate::Error>(DownloadedBulkProject::ProjectUpdate(
                        update, downloaded,
                    ))
                }
                PlannedDownload::DependencyAddition(dependency) => {
                    let downloaded = download_project_version(
                        instance_id,
                        &dependency.version_id,
                        DownloadReason::Dependency,
                        Some(dependency.parent_version_id.clone()),
                        state,
                    )
                    .await?;

                    Ok::<_, crate::Error>(
                        DownloadedBulkProject::DependencyAddition(downloaded),
                    )
                }
            }
        })
        .collect::<FuturesUnordered<_>>();
    let mut completed = 0;
    let mut output = Vec::with_capacity(total);

    while let Some(download) = downloads.next().await {
        let download = download?;
        completed += 1;
        emit_bulk_update_progress(
            instance_id,
            crate::event::InstanceBulkUpdateProgressStage::Downloading,
            completed,
            total,
        )
        .await?;
        output.push(download);
    }

    Ok(output)
}

async fn emit_bulk_update_progress(
    instance_id: &str,
    stage: crate::event::InstanceBulkUpdateProgressStage,
    current: usize,
    total: usize,
) -> crate::Result<()> {
    crate::event::emit::emit_instance_bulk_update_progress(
        crate::event::InstanceBulkUpdateProgressPayload {
            instance_id: instance_id.to_string(),
            stage,
            current,
            total,
        },
    )
    .await
}

async fn plan_bulk_update(
    instance_id: &str,
    state: &State,
) -> crate::Result<BulkUpdatePlan> {
    let updateable_paths =
        bulk_updateable_project_paths(instance_id, state).await?;
    if updateable_paths.is_empty() {
        return Ok(BulkUpdatePlan {
            project_updates: Vec::new(),
            dependency_additions: Vec::new(),
        });
    }

    let updates = check_content_updates(
        instance_id,
        Some(CacheBehaviour::MustRevalidate),
        state,
    )
    .await?
    .into_iter()
    .filter(|update| updateable_paths.contains(&update.relative_path))
    .collect::<Vec<_>>();
    if updates.is_empty() {
        return Ok(BulkUpdatePlan {
            project_updates: Vec::new(),
            dependency_additions: Vec::new(),
        });
    }

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
                (
                    update.current_version_id.clone(),
                    update.update_version_id.clone(),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let version_ids = installed
        .iter()
        .filter(|project| updateable_paths.contains(&project.relative_path))
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
    let planned_versions = installed
        .iter()
        .filter(|project| project.enabled)
        .filter(|project| updateable_paths.contains(&project.relative_path))
        .filter_map(|project| {
            let target_version_id = updates_by_path
                .get(&project.relative_path)
                .map(|(_, update_version_id)| update_version_id)
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
        })
        .collect::<Vec<_>>();
    let project_updates = updates
        .into_iter()
        .map(|update| PlannedProjectUpdate {
            relative_path: update.relative_path,
            current_version_id: update.current_version_id,
            update_version_id: update.update_version_id,
        })
        .collect::<Vec<_>>();

    Ok(BulkUpdatePlan {
        project_updates,
        dependency_additions,
    })
}

async fn bulk_updateable_project_paths(
    instance_id: &str,
    state: &State,
) -> crate::Result<HashSet<String>> {
    let items = super::list_content::list_content(
        instance_id,
        None,
        Some(CacheBehaviour::MustRevalidate),
        state,
    )
    .await?;

    Ok(items.into_iter().map(|item| item.file_path).collect())
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
        enabled: entry.enabled && file.enabled,
    })
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

            output.entry(project_id.clone()).or_insert_with(|| {
                ResolvedDependency {
                    project_id,
                    version_id: dependency_version.id.clone(),
                    parent_version_id: version.id.clone(),
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
