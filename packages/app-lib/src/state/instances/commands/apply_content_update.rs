use crate::state::instances::{
    BulkUpdatePreview, ContentDiffItem, ContentEntry, ContentSet,
    ContentSourceKind, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CacheBehaviour, CachedEntry, Dependency, DependencyType, Project, State,
    Version,
};
use crate::util::fetch::DownloadReason;
use futures::future::try_join_all;
use std::collections::{HashMap, HashSet};

use super::apply_content_install::{
    add_downloaded_project_version, add_project_from_version,
    download_project_version, remove_project, toggle_disable_project,
};
use super::check_content_updates::{ContentUpdate, check_content_updates};

#[derive(Clone, Debug)]
struct BulkUpdatePlan {
    project_updates: Vec<PlannedProjectUpdate>,
    dependency_additions: Vec<PlannedDependencyInstall>,
    disable_candidates: Vec<PlannedDisableCandidate>,
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
struct PlannedDisableCandidate {
    relative_path: String,
    file_name: String,
    project_id: Option<String>,
    version_id: Option<String>,
}

#[derive(Clone, Debug)]
struct InstalledProject {
    relative_path: String,
    file_name: String,
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

pub(crate) async fn preview_update_all_projects(
    instance_id: &str,
    state: &State,
) -> crate::Result<BulkUpdatePreview> {
    let plan = plan_bulk_update(instance_id, state).await?;
    build_preview(&plan, state).await
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
            toggle_disable_project(instance_id, &new_path, state).await?;
    }

    if new_path != project_path {
        remove_project(instance_id, project_path, state).await?;
    }

    Ok(new_path)
}

pub(crate) async fn update_all_projects(
    instance_id: &str,
    state: &State,
) -> crate::Result<HashMap<String, String>> {
    let plan = plan_bulk_update(instance_id, state).await?;
    let update_downloads =
        try_join_all(plan.project_updates.iter().map(|update| async move {
            let downloaded = download_project_version(
                instance_id,
                &update.update_version_id,
                DownloadReason::Update,
                Some(update.current_version_id.clone()),
                state,
            )
            .await?;

            Ok::<_, crate::Error>((update.clone(), downloaded))
        }));
    let dependency_downloads =
        try_join_all(plan.dependency_additions.iter().map(
            |dependency| async move {
                download_project_version(
                    instance_id,
                    &dependency.version_id,
                    DownloadReason::Dependency,
                    Some(dependency.parent_version_id.clone()),
                    state,
                )
                .await
            },
        ));
    let (update_downloads, dependency_downloads) =
        tokio::try_join!(update_downloads, dependency_downloads)?;

    let mut changed = HashMap::new();
    for (update, downloaded) in update_downloads {
        let mut new_path = add_downloaded_project_version(
            instance_id,
            downloaded,
            ContentSourceKind::Local,
            state,
        )
        .await?;

        if update.relative_path.ends_with(".disabled") {
            new_path =
                toggle_disable_project(instance_id, &new_path, state).await?;
        }

        if new_path != update.relative_path {
            remove_project(instance_id, &update.relative_path, state).await?;
        }

        changed.insert(update.relative_path, new_path);
    }

    for downloaded in dependency_downloads {
        add_downloaded_project_version(
            instance_id,
            downloaded,
            ContentSourceKind::Local,
            state,
        )
        .await?;
    }

    for candidate in plan.disable_candidates {
        let path = changed
            .get(&candidate.relative_path)
            .cloned()
            .unwrap_or(candidate.relative_path);

        if !path.ends_with(".disabled") {
            let disabled_path =
                toggle_disable_project(instance_id, &path, state).await?;
            changed.insert(path, disabled_path);
        }
    }

    Ok(changed)
}

async fn plan_bulk_update(
    instance_id: &str,
    state: &State,
) -> crate::Result<BulkUpdatePlan> {
    let updates = check_content_updates(
        instance_id,
        Some(CacheBehaviour::MustRevalidate),
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
    let current_versions = installed
        .iter()
        .filter(|project| project.enabled)
        .filter_map(|project| {
            versions_by_id.get(project.version_id.as_ref()?).cloned()
        })
        .collect::<Vec<_>>();
    let planned_versions = installed
        .iter()
        .filter(|project| project.enabled)
        .filter_map(|project| {
            let target_version_id = updates_by_path
                .get(&project.relative_path)
                .map(|(_, update_version_id)| update_version_id)
                .or(project.version_id.as_ref())?;

            versions_by_id.get(target_version_id).cloned()
        })
        .collect::<Vec<_>>();
    let current_dependencies =
        dependency_closure(current_versions, &content_set, state).await?;
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
    let disable_candidates = current_dependencies
        .keys()
        .filter(|project_id| !planned_dependencies.contains_key(*project_id))
        .filter_map(|project_id| installed_by_project.get(project_id))
        .filter(|project| project.enabled)
        .map(|project| PlannedDisableCandidate {
            relative_path: project.relative_path.clone(),
            file_name: project.file_name.clone(),
            project_id: project.project_id.clone(),
            version_id: project.version_id.clone(),
        })
        .collect::<Vec<_>>();
    let disable_paths = disable_candidates
        .iter()
        .map(|candidate| candidate.relative_path.as_str())
        .collect::<HashSet<_>>();
    let project_updates = updates
        .into_iter()
        .filter(|update| !disable_paths.contains(update.relative_path.as_str()))
        .map(|update| PlannedProjectUpdate {
            relative_path: update.relative_path,
            current_version_id: update.current_version_id,
            update_version_id: update.update_version_id,
        })
        .collect::<Vec<_>>();

    Ok(BulkUpdatePlan {
        project_updates,
        dependency_additions,
        disable_candidates,
    })
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
        file_name: file.file_name.clone(),
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

    versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));

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

async fn build_preview(
    plan: &BulkUpdatePlan,
    state: &State,
) -> crate::Result<BulkUpdatePreview> {
    let project_ids = plan
        .disable_candidates
        .iter()
        .filter_map(|candidate| candidate.project_id.as_deref())
        .collect::<HashSet<_>>();
    let version_ids = plan
        .disable_candidates
        .iter()
        .filter_map(|candidate| candidate.version_id.as_deref())
        .collect::<HashSet<_>>();
    let project_id_refs = project_ids.iter().copied().collect::<Vec<_>>();
    let version_id_refs = version_ids.iter().copied().collect::<Vec<_>>();
    let projects = CachedEntry::get_project_many(
        &project_id_refs,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await
    .unwrap_or_default();
    let versions = CachedEntry::get_version_many(
        &version_id_refs,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await
    .unwrap_or_default();
    let project_names = projects
        .into_iter()
        .map(|project: Project| (project.id, project.title))
        .collect::<HashMap<_, _>>();
    let version_names = versions
        .into_iter()
        .map(|version| (version.id, version.version_number))
        .collect::<HashMap<_, _>>();
    let disable_candidates =
        plan.disable_candidates
            .iter()
            .map(|candidate| ContentDiffItem {
                type_: "removed".to_string(),
                project_name: candidate.project_id.as_ref().and_then(
                    |project_id| project_names.get(project_id).cloned(),
                ),
                file_name: Some(candidate.file_name.clone()),
                current_version_name: candidate.version_id.as_ref().and_then(
                    |version_id| version_names.get(version_id).cloned(),
                ),
                new_version_name: None,
            })
            .collect::<Vec<_>>();
    let disable_paths = plan
        .disable_candidates
        .iter()
        .map(|candidate| candidate.relative_path.clone())
        .collect::<Vec<_>>();

    Ok(BulkUpdatePreview {
        requires_confirmation: !disable_paths.is_empty(),
        disable_candidates,
        disable_paths,
    })
}
