use super::sync_content_files::{
    project_type_for_file, sync_instance_content_files,
};
use crate::State;
use crate::pack::install_from::{PackFileHash, PackFormat};
use crate::state::instances::adapters::sqlite;
use crate::state::instances::{
    ContentEntry, ContentSet, Instance, InstanceLink,
};
use crate::state::{
    CacheBehaviour, CachedEntry, CachedFile, ContentFile, ContentItem,
    ContentItemOwner, ContentItemProject, ContentItemVersion, Dependency,
    LinkedModpackInfo, Organization, OwnerType, Project, ProjectType,
    ReleaseChannel, TeamMember, Version,
};
use crate::util::fetch::{
    DownloadMeta, DownloadReason, FetchSemaphore, fetch_mirrors, sha1_async,
};
use async_zip::base::read::seek::ZipFileReader;
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;

#[derive(Clone, Debug)]
struct ResolvedContentScope {
    instance: Instance,
    content_set: ContentSet,
}

#[derive(Clone, Copy, Debug)]
enum ContentFilter<'a> {
    All,
    ExcludeModpack(&'a ModpackIdentifiers),
    OnlyModpack(&'a ModpackIdentifiers),
}

pub(crate) async fn list_content_sets(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Vec<ContentSet>> {
    let instance = sqlite::instance_rows::get_instance_by_id(instance_id, pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;

    sqlite::content_rows::get_content_sets_for_instance(&instance.id, pool)
        .await
}

pub(crate) async fn get_content_projects(
    instance_id: &str,
    content_set_id: Option<&str>,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<DashMap<String, ContentFile>> {
    let resolved = resolve_content_scope_with_instance(
        instance_id,
        content_set_id,
        &state.pool,
    )
    .await?;

    content_projects_for_scope(
        &resolved,
        cache_behaviour,
        state,
        ContentFilter::All,
    )
    .await
}

pub(crate) async fn get_installed_project_ids_for_instance(
    instance_id: &str,
    content_set_id: Option<&str>,
    state: &State,
) -> crate::Result<Vec<String>> {
    let projects =
        get_content_projects(instance_id, content_set_id, None, state).await?;

    Ok(projects
        .into_iter()
        .filter_map(|(_, file)| {
            file.metadata.map(|metadata| metadata.project_id)
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect())
}

pub(crate) async fn list_content(
    instance_id: &str,
    content_set_id: Option<&str>,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Vec<ContentItem>> {
    let resolved = resolve_content_scope_with_instance(
        instance_id,
        content_set_id,
        &state.pool,
    )
    .await?;
    let modpack_ids = match linked_modpack_ids_for_instance(
        &resolved.instance.id,
        &state.pool,
    )
    .await?
    {
        Some((_, version_id)) => get_modpack_identifiers(
            &version_id,
            &resolved.content_set,
            &state.pool,
            &state.api_semaphore,
        )
        .await
        .ok(),
        None => None,
    };
    let filter = modpack_ids
        .as_ref()
        .map(ContentFilter::ExcludeModpack)
        .unwrap_or(ContentFilter::All);
    let files =
        content_projects_for_scope(&resolved, cache_behaviour, state, filter)
            .await?;
    let files = files.into_iter().collect::<Vec<_>>();

    content_files_to_content_items(
        &resolved.instance,
        &files,
        cache_behaviour,
        state,
    )
    .await
}

pub(crate) async fn list_linked_modpack_content(
    instance_id: &str,
    content_set_id: Option<&str>,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Vec<ContentItem>> {
    let resolved = resolve_content_scope_with_instance(
        instance_id,
        content_set_id,
        &state.pool,
    )
    .await?;
    let Some((_, version_id)) =
        linked_modpack_ids_for_instance(&resolved.instance.id, &state.pool)
            .await?
    else {
        return Ok(Vec::new());
    };
    let ids = match get_modpack_identifiers(
        &version_id,
        &resolved.content_set,
        &state.pool,
        &state.api_semaphore,
    )
    .await
    {
        Ok(ids) => ids,
        Err(err) => {
            tracing::warn!("Failed to fetch modpack identifiers: {}", err);
            return Ok(Vec::new());
        }
    };
    let files = content_projects_for_scope(
        &resolved,
        cache_behaviour,
        state,
        ContentFilter::OnlyModpack(&ids),
    )
    .await?;
    let files = files.into_iter().collect::<Vec<_>>();

    content_files_to_content_items(
        &resolved.instance,
        &files,
        cache_behaviour,
        state,
    )
    .await
}

pub(crate) async fn get_linked_modpack_info(
    instance_id: &str,
    content_set_id: Option<&str>,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Option<LinkedModpackInfo>> {
    let resolved = resolve_content_scope_with_instance(
        instance_id,
        content_set_id,
        &state.pool,
    )
    .await?;
    let Some((project_id, version_id)) =
        linked_modpack_ids_for_instance(&resolved.instance.id, &state.pool)
            .await?
    else {
        return Ok(None);
    };
    let (project, version, all_versions) = tokio::try_join!(
        CachedEntry::get_project(
            &project_id,
            cache_behaviour,
            &state.pool,
            &state.api_semaphore,
        ),
        CachedEntry::get_version(
            &version_id,
            cache_behaviour,
            &state.pool,
            &state.api_semaphore,
        ),
        CachedEntry::get_project_versions(
            &project_id,
            cache_behaviour,
            &state.pool,
            &state.api_semaphore,
        ),
    )?;
    let version = version.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Linked modpack version {version_id} not found"
        ))
    })?;
    let (project, all_versions) = if version.project_id != project_id {
        let (modpack_project, modpack_versions) = tokio::try_join!(
            CachedEntry::get_project(
                &version.project_id,
                cache_behaviour,
                &state.pool,
                &state.api_semaphore,
            ),
            CachedEntry::get_project_versions(
                &version.project_id,
                cache_behaviour,
                &state.pool,
                &state.api_semaphore,
            ),
        )?;
        (modpack_project.or(project), modpack_versions)
    } else {
        (project, all_versions)
    };
    let project = project.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Linked modpack project {project_id} not found"
        ))
    })?;
    let owner = if let Some(org_id) = &project.organization {
        let org = CachedEntry::get_organization(
            org_id,
            cache_behaviour,
            &state.pool,
            &state.api_semaphore,
        )
        .await?;
        org.map(|org| ContentItemOwner {
            id: org.id,
            name: org.name,
            avatar_url: org.icon_url,
            owner_type: OwnerType::Organization,
        })
    } else {
        let team = CachedEntry::get_team(
            &project.team,
            cache_behaviour,
            &state.pool,
            &state.api_semaphore,
        )
        .await?;
        team.and_then(|team| {
            team.into_iter()
                .find(|member| member.is_owner)
                .map(|member| ContentItemOwner {
                    id: member.user.id,
                    name: member.user.username,
                    avatar_url: member.user.avatar_url,
                    owner_type: OwnerType::User,
                })
        })
    };
    let (has_update, update_version_id, update_version) = check_modpack_update(
        &version_id,
        &version,
        all_versions,
        resolved.instance.update_channel,
    );

    Ok(Some(LinkedModpackInfo {
        project,
        version,
        owner,
        has_update,
        update_version_id,
        update_version,
    }))
}

pub(crate) async fn dependencies_to_content_items(
    dependencies: &[Dependency],
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let project_ids = dependencies
        .iter()
        .filter_map(|dependency| dependency.project_id.clone())
        .collect::<HashSet<_>>();
    if project_ids.is_empty() {
        return Ok(Vec::new());
    }
    let version_ids = dependencies
        .iter()
        .filter_map(|dependency| dependency.version_id.clone())
        .collect::<HashSet<_>>();
    let meta = resolve_metadata(
        &project_ids,
        &version_ids,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await?;
    let mut items = dependencies
        .iter()
        .filter_map(|dependency| {
            let project_id = dependency.project_id.as_ref()?;
            let project = meta
                .projects
                .iter()
                .find(|project| &project.id == project_id)?;
            let version =
                dependency.version_id.as_ref().and_then(|version_id| {
                    meta.versions
                        .iter()
                        .find(|version| &version.id == version_id)
                });
            let owner =
                resolve_owner(project, &meta.teams, &meta.organizations);
            let project_type =
                project_type_from_api_name(&project.project_type);

            Some(ContentItem {
                file_name: version
                    .and_then(|version| version.files.first())
                    .map(|file| file.filename.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{}.jar",
                            project.slug.as_deref().unwrap_or(&project.id)
                        )
                    }),
                file_path: String::new(),
                id: String::new(),
                size: version
                    .and_then(|version| version.files.first())
                    .map(|file| file.size as u64)
                    .unwrap_or(0),
                enabled: true,
                project_type,
                project: Some(ContentItemProject {
                    id: project.id.clone(),
                    slug: project.slug.clone(),
                    title: project.title.clone(),
                    icon_url: project.icon_url.clone(),
                }),
                version: version.map(|version| ContentItemVersion {
                    id: version.id.clone(),
                    version_number: version.version_number.clone(),
                    file_name: version
                        .files
                        .first()
                        .map(|file| file.filename.clone())
                        .unwrap_or_default(),
                    date_published: Some(version.date_published.to_rfc3339()),
                }),
                owner,
                has_update: false,
                update_version_id: None,
                date_added: None,
            })
        })
        .collect::<Vec<_>>();
    sort_content_items(&mut items);

    Ok(items)
}

async fn resolve_content_scope_with_instance(
    instance_id: &str,
    content_set_id: Option<&str>,
    pool: &SqlitePool,
) -> crate::Result<ResolvedContentScope> {
    let instance = sqlite::instance_rows::get_instance_by_id(instance_id, pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let content_set = match content_set_id {
        Some(content_set_id) => {
            let content_set =
                sqlite::content_rows::get_content_set(content_set_id, pool)
                    .await?
                    .ok_or_else(|| {
                        crate::ErrorKind::InputError(format!(
                            "Unknown content set {content_set_id}"
                        ))
                    })?;

            if content_set.instance_id != instance.id {
                return Err(crate::ErrorKind::InputError(format!(
					"Content set {content_set_id} does not belong to instance {}",
					instance.id
				))
				.into());
            }

            content_set
        }
        None => {
            sqlite::content_rows::get_applied_content_set(&instance.id, pool)
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError(format!(
                        "Instance {} has no applied content set",
                        instance.id
                    ))
                })?
        }
    };

    Ok(ResolvedContentScope {
        instance,
        content_set,
    })
}

async fn content_projects_for_scope(
    resolved: &ResolvedContentScope,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
    filter: ContentFilter<'_>,
) -> crate::Result<DashMap<String, ContentFile>> {
    let files = sync_instance_content_files(&resolved.instance, state).await?;
    let entries = sqlite::content_rows::get_content_entries(
        &resolved.content_set.id,
        &state.pool,
    )
    .await?;
    let entries_by_file_id = entries
        .iter()
        .filter_map(|entry| {
            entry.file_id.as_deref().map(|file_id| (file_id, entry))
        })
        .collect::<HashMap<_, _>>();
    let hashes = files
        .iter()
        .map(|file| file.sha1.as_str())
        .collect::<Vec<_>>();
    let file_info = CachedEntry::get_file_many(
        &hashes,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let file_info_by_hash = file_info
        .into_iter()
        .map(|file| (file.hash.clone(), file))
        .collect::<HashMap<_, _>>();
    let installed_channels = get_installed_update_channels(
        &file_info_by_hash,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let update_keys = files
        .iter()
        .filter(|file| file_info_by_hash.contains_key(&file.sha1))
        .filter_map(|file| {
            let project_type = project_type_for_file(file)?;
            let channel = resolved.instance.update_channel.least_stable(
                installed_channels
                    .get(&file.sha1)
                    .copied()
                    .unwrap_or(resolved.instance.update_channel),
            );
            Some(file_update_cache_key(
                &file.sha1,
                project_type,
                &resolved.content_set,
                channel,
            ))
        })
        .collect::<Vec<_>>();
    let update_key_refs =
        update_keys.iter().map(String::as_str).collect::<Vec<_>>();
    let file_updates = CachedEntry::get_file_update_many(
        &update_key_refs,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let mut updates_by_hash: HashMap<String, Vec<String>> = HashMap::new();
    for update in file_updates {
        updates_by_hash
            .entry(update.hash)
            .or_default()
            .push(update.update_version_id);
    }
    let output = DashMap::new();

    for file in files {
        if file.missing {
            continue;
        }

        let Some(project_type) = project_type_for_file(&file) else {
            continue;
        };
        let metadata = file_info_by_hash.get(&file.sha1).cloned();
        let entry = entries_by_file_id.get(file.id.as_str()).copied();

        match filter {
            ContentFilter::All => {}
            ContentFilter::ExcludeModpack(ids) => {
                if ids.is_modpack_file(
                    &file.sha1,
                    metadata.as_ref(),
                    entry.and_then(|entry| entry.project_id.as_deref()),
                ) {
                    continue;
                }
            }
            ContentFilter::OnlyModpack(ids) => {
                if !ids.is_modpack_file(
                    &file.sha1,
                    metadata.as_ref(),
                    entry.and_then(|entry| entry.project_id.as_deref()),
                ) {
                    continue;
                }
            }
        }

        let update_version_id = metadata.as_ref().and_then(|metadata| {
            let update_ids =
                updates_by_hash.remove(&file.sha1).unwrap_or_default();
            if !update_ids.contains(&metadata.version_id) {
                update_ids.into_iter().next()
            } else {
                None
            }
        });

        output.insert(
            file.relative_path.clone(),
            ContentFile {
                update_version_id,
                hash: file.sha1,
                file_name: file.file_name,
                size: file.size,
                metadata: file_metadata_from_entry_or_cache(entry, metadata),
                project_type,
            },
        );
    }

    Ok(output)
}

async fn get_installed_update_channels(
    file_info_by_hash: &HashMap<String, CachedFile>,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<HashMap<String, ReleaseChannel>> {
    let version_ids = file_info_by_hash
        .values()
        .map(|file| file.version_id.as_str())
        .collect::<HashSet<_>>();
    if version_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let version_id_refs = version_ids.iter().copied().collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &version_id_refs,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await?;
    let channels_by_version_id = versions
        .into_iter()
        .map(|version| {
            (
                version.id,
                ReleaseChannel::from_version_type(&version.version_type),
            )
        })
        .collect::<HashMap<_, _>>();

    Ok(file_info_by_hash
        .iter()
        .filter_map(|(hash, file)| {
            channels_by_version_id
                .get(&file.version_id)
                .copied()
                .map(|channel| (hash.clone(), channel))
        })
        .collect())
}

fn file_update_cache_key(
    hash: &str,
    project_type: ProjectType,
    content_set: &ContentSet,
    channel: ReleaseChannel,
) -> String {
    let loader_key = if project_type == ProjectType::Mod {
        content_set.loader.as_str().to_string()
    } else {
        project_type.get_loaders().join("+")
    };

    format!(
        "{}-{}-{}-{}",
        hash,
        loader_key,
        channel.key(),
        content_set.game_version
    )
}

async fn content_files_to_content_items(
    instance: &Instance,
    files: &[(String, ContentFile)],
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Vec<ContentItem>> {
    let project_ids = files
        .iter()
        .filter_map(|(_, file)| {
            file.metadata
                .as_ref()
                .map(|metadata| metadata.project_id.clone())
        })
        .collect::<HashSet<_>>();
    let version_ids = files
        .iter()
        .filter_map(|(_, file)| {
            file.metadata
                .as_ref()
                .map(|metadata| metadata.version_id.clone())
        })
        .collect::<HashSet<_>>();
    let meta = resolve_metadata(
        &project_ids,
        &version_ids,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let instance_path = state.directories.instances_dir().join(&instance.path);
    let paths = files
        .iter()
        .map(|(path, _)| instance_path.join(path))
        .collect::<Vec<_>>();
    let modification_times: Vec<Option<String>> =
        tokio::task::spawn_blocking(move || {
            paths
                .iter()
                .map(|path| {
                    std::fs::metadata(path)
                        .and_then(|metadata| metadata.modified())
                        .ok()
                        .map(|time| {
                            chrono::DateTime::<chrono::Utc>::from(time)
                                .to_rfc3339()
                        })
                })
                .collect()
        })
        .await?;
    let mut items = files
        .iter()
        .enumerate()
        .map(|(index, (path, file))| {
            let project = file.metadata.as_ref().and_then(|metadata| {
                meta.projects
                    .iter()
                    .find(|project| project.id == metadata.project_id)
            });
            let version = file.metadata.as_ref().and_then(|metadata| {
                meta.versions
                    .iter()
                    .find(|version| version.id == metadata.version_id)
            });
            let owner = project.and_then(|project| {
                resolve_owner(project, &meta.teams, &meta.organizations)
            });

            ContentItem {
                file_name: file.file_name.clone(),
                file_path: path.clone(),
                id: file.hash.clone(),
                size: file.size,
                enabled: !file.file_name.ends_with(".disabled"),
                project_type: file.project_type,
                project: project.map(|project| ContentItemProject {
                    id: project.id.clone(),
                    slug: project.slug.clone(),
                    title: project.title.clone(),
                    icon_url: project.icon_url.clone(),
                }),
                version: version.map(|version| ContentItemVersion {
                    id: version.id.clone(),
                    version_number: version.version_number.clone(),
                    file_name: file.file_name.clone(),
                    date_published: Some(version.date_published.to_rfc3339()),
                }),
                owner,
                has_update: file.update_version_id.is_some(),
                update_version_id: file.update_version_id.clone(),
                date_added: modification_times[index].clone(),
            }
        })
        .collect::<Vec<_>>();
    sort_content_items(&mut items);

    Ok(items)
}

struct ResolvedMetadata {
    projects: Vec<Project>,
    versions: Vec<Version>,
    teams: Vec<Vec<TeamMember>>,
    organizations: Vec<Organization>,
}

async fn resolve_metadata(
    project_ids: &HashSet<String>,
    version_ids: &HashSet<String>,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<ResolvedMetadata> {
    let project_id_refs =
        project_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let version_id_refs =
        version_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let (projects, versions) =
        if !project_ids.is_empty() || !version_ids.is_empty() {
            tokio::try_join!(
                async {
                    if project_ids.is_empty() {
                        Ok(Vec::new())
                    } else {
                        CachedEntry::get_project_many(
                            &project_id_refs,
                            cache_behaviour,
                            pool,
                            fetch_semaphore,
                        )
                        .await
                    }
                },
                async {
                    if version_ids.is_empty() {
                        Ok(Vec::new())
                    } else {
                        CachedEntry::get_version_many(
                            &version_id_refs,
                            cache_behaviour,
                            pool,
                            fetch_semaphore,
                        )
                        .await
                    }
                }
            )?
        } else {
            (Vec::new(), Vec::new())
        };
    let team_ids = projects
        .iter()
        .map(|project| project.team.clone())
        .collect::<HashSet<_>>();
    let org_ids = projects
        .iter()
        .filter_map(|project| project.organization.clone())
        .collect::<HashSet<_>>();
    let team_id_refs = team_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let org_id_refs = org_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let (teams, organizations) = if !team_ids.is_empty() || !org_ids.is_empty()
    {
        tokio::try_join!(
            async {
                if team_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    CachedEntry::get_team_many(
                        &team_id_refs,
                        cache_behaviour,
                        pool,
                        fetch_semaphore,
                    )
                    .await
                }
            },
            async {
                if org_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    CachedEntry::get_organization_many(
                        &org_id_refs,
                        cache_behaviour,
                        pool,
                        fetch_semaphore,
                    )
                    .await
                }
            }
        )?
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(ResolvedMetadata {
        projects,
        versions,
        teams,
        organizations,
    })
}

fn resolve_owner(
    project: &Project,
    teams: &[Vec<TeamMember>],
    organizations: &[Organization],
) -> Option<ContentItemOwner> {
    if let Some(org_id) = &project.organization {
        organizations
            .iter()
            .find(|organization| &organization.id == org_id)
            .map(|organization| ContentItemOwner {
                id: organization.id.clone(),
                name: organization.name.clone(),
                avatar_url: organization.icon_url.clone(),
                owner_type: OwnerType::Organization,
            })
    } else {
        teams
            .iter()
            .find(|team| {
                team.first()
                    .is_some_and(|member| member.team_id == project.team)
            })
            .and_then(|team| team.iter().find(|member| member.is_owner))
            .map(|member| ContentItemOwner {
                id: member.user.id.clone(),
                name: member.user.username.clone(),
                avatar_url: member.user.avatar_url.clone(),
                owner_type: OwnerType::User,
            })
    }
}

fn file_metadata_from_entry_or_cache(
    entry: Option<&ContentEntry>,
    cached: Option<CachedFile>,
) -> Option<crate::state::FileMetadata> {
    let project_id = entry
        .and_then(|entry| entry.project_id.clone())
        .or_else(|| cached.as_ref().map(|file| file.project_id.clone()))?;
    let version_id = entry
        .and_then(|entry| entry.version_id.clone())
        .or_else(|| cached.as_ref().map(|file| file.version_id.clone()))?;

    Some(crate::state::FileMetadata {
        project_id,
        version_id,
    })
}

async fn linked_modpack_ids_for_instance(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<(String, String)>> {
    let link =
        sqlite::instance_rows::get_instance_link(instance_id, pool).await?;
    Ok(linked_modpack_ids(&link))
}

fn linked_modpack_ids(link: &InstanceLink) -> Option<(String, String)> {
    match link {
        InstanceLink::ModrinthModpack {
            project_id,
            version_id,
        } => Some((project_id.clone(), version_id.clone())),
        InstanceLink::ServerProjectModpack {
            content_project_id,
            content_version_id,
            ..
        } => Some((content_project_id.clone(), content_version_id.clone())),
        InstanceLink::ImportedModpack {
            project_id: Some(project_id),
            version_id: Some(version_id),
        } => Some((project_id.clone(), version_id.clone())),
        _ => None,
    }
}

fn check_modpack_update(
    installed_version_id: &str,
    installed_version: &Version,
    all_versions: Option<Vec<Version>>,
    preferred_update_channel: ReleaseChannel,
) -> (bool, Option<String>, Option<Version>) {
    let Some(versions) = all_versions else {
        return (false, None, None);
    };
    let installed_channel =
        ReleaseChannel::from_version_type(&installed_version.version_type);
    let effective_channel =
        preferred_update_channel.least_stable(installed_channel);

    for version_types in effective_channel.version_type_fallbacks() {
        if !versions.iter().any(|version| {
            version_types.contains(&version.version_type.as_str())
        }) {
            continue;
        }

        let mut newer_versions = versions
            .iter()
            .filter(|version| {
                version.id != installed_version_id
                    && version.date_published > installed_version.date_published
                    && version_types.contains(&version.version_type.as_str())
            })
            .collect::<Vec<_>>();
        newer_versions
            .sort_by_key(|version| std::cmp::Reverse(version.date_published));

        if let Some(newest) = newer_versions.first() {
            return (true, Some(newest.id.clone()), Some((*newest).clone()));
        }

        return (false, None, None);
    }

    (false, None, None)
}

#[derive(Clone, Debug)]
struct ModpackIdentifiers {
    hashes: HashSet<String>,
    project_ids: HashSet<String>,
}

impl ModpackIdentifiers {
    fn is_modpack_file(
        &self,
        hash: &str,
        file: Option<&CachedFile>,
        entry_project_id: Option<&str>,
    ) -> bool {
        self.hashes.contains(hash)
            || entry_project_id
                .is_some_and(|project_id| self.project_ids.contains(project_id))
            || file
                .is_some_and(|file| self.project_ids.contains(&file.project_id))
    }
}

async fn get_modpack_identifiers(
    version_id: &str,
    content_set: &ContentSet,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<ModpackIdentifiers> {
    if let Some(cached) =
        CachedEntry::get_modpack_files(version_id, pool, fetch_semaphore)
            .await?
    {
        if !cached.project_ids.is_empty() {
            return Ok(ModpackIdentifiers {
                hashes: cached.file_hashes.into_iter().collect(),
                project_ids: cached.project_ids.into_iter().collect(),
            });
        }

        let hash_refs = cached
            .file_hashes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let files =
            CachedEntry::get_file_many(&hash_refs, None, pool, fetch_semaphore)
                .await?;
        let project_ids = files
            .iter()
            .map(|file| file.project_id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        CachedEntry::cache_modpack_files(
            version_id,
            cached.file_hashes.clone(),
            project_ids.clone(),
            pool,
        )
        .await?;

        return Ok(ModpackIdentifiers {
            hashes: cached.file_hashes.into_iter().collect(),
            project_ids: project_ids.into_iter().collect(),
        });
    }

    let version =
        CachedEntry::get_version(version_id, None, pool, fetch_semaphore)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Modpack version {version_id} not found"
                ))
            })?;
    let primary_file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "No files found for modpack version {version_id}"
            ))
        })?;
    let download_meta = DownloadMeta {
        reason: DownloadReason::Modpack,
        game_version: content_set.game_version.clone(),
        loader: content_set.loader.as_str().to_string(),
        dependent_on: Some(version_id.to_string()),
    };
    let mrpack_bytes = fetch_mirrors(
        &[&primary_file.url],
        primary_file.hashes.get("sha1").map(String::as_str),
        Some(&download_meta),
        fetch_semaphore,
        pool,
    )
    .await?;
    let reader = Cursor::new(&mrpack_bytes);
    let mut zip_reader =
        ZipFileReader::with_tokio(reader).await.map_err(|_| {
            crate::ErrorKind::InputError(
                "Failed to read modpack zip".to_string(),
            )
        })?;
    let manifest_idx = zip_reader
        .file()
        .entries()
        .iter()
        .position(|file| {
            matches!(file.filename().as_str(), Ok("modrinth.index.json"))
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No modrinth.index.json found in mrpack".to_string(),
            )
        })?;
    let mut manifest = String::new();
    let mut entry_reader = zip_reader.reader_with_entry(manifest_idx).await?;
    entry_reader.read_to_string_checked(&mut manifest).await?;
    let pack: PackFormat = serde_json::from_str(&manifest)?;
    let mut hashes = pack
        .files
        .iter()
        .filter_map(|file| file.hashes.get(&PackFileHash::Sha1).cloned())
        .collect::<Vec<_>>();
    let project_ids = pack
        .files
        .iter()
        .filter_map(|file| {
            file.downloads.iter().find_map(|url| {
                let parts = url.split('/').collect::<Vec<_>>();
                let data_idx = parts.iter().position(|part| *part == "data")?;
                parts.get(data_idx + 1).map(|part| part.to_string())
            })
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let override_entries = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let filename = entry.filename().as_str().ok()?;
            let is_override = (filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/")
                || filename.starts_with("server-overrides/"))
                && !filename.ends_with('/');
            is_override.then_some(index)
        })
        .collect::<Vec<_>>();

    for index in override_entries {
        let mut file_bytes = Vec::new();
        let mut entry_reader = zip_reader.reader_with_entry(index).await?;
        entry_reader.read_to_end_checked(&mut file_bytes).await?;
        hashes.push(sha1_async(bytes::Bytes::from(file_bytes)).await?);
    }

    CachedEntry::cache_modpack_files(
        version_id,
        hashes.clone(),
        project_ids.clone(),
        pool,
    )
    .await?;

    Ok(ModpackIdentifiers {
        hashes: hashes.into_iter().collect(),
        project_ids: project_ids.into_iter().collect(),
    })
}

fn project_type_from_api_name(project_type: &str) -> ProjectType {
    match project_type {
        "resourcepack" => ProjectType::ResourcePack,
        "shader" => ProjectType::ShaderPack,
        "datapack" => ProjectType::DataPack,
        _ => ProjectType::Mod,
    }
}

fn sort_content_items(items: &mut [ContentItem]) {
    items.sort_by(|left, right| {
        let left_name = left
            .project
            .as_ref()
            .map(|project| project.title.as_str())
            .unwrap_or(&left.file_name);
        let right_name = right
            .project
            .as_ref()
            .map(|project| project.title.as_str())
            .unwrap_or(&right.file_name);

        left_name
            .to_lowercase()
            .cmp(&right_name.to_lowercase())
            .then_with(|| left.file_name.cmp(&right.file_name))
    });
}
