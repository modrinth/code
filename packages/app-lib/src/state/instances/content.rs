//! # Content API
//!
//! ## Data Flow
//!
//! 1. Frontend calls `get_content_items(profile_path)`
//! 2. Backend fetches all installed files via `Profile::get_projects()`
//! 3. If profile is linked to a modpack:
//!    - Fetch modpack file hashes from cache (populated during installation)
//!    - Fallback: re-download .mrpack if cache miss (cleared/expired)
//!    - Filter out files that belong to the modpack
//! 4. For remaining files, fetch project/version/owner metadata in parallel
//! 5. Return sorted `ContentItem` list
//!
//! ## Caching
//!
//! Modpack file hashes are cached in `CacheValueType::ModpackFiles`
//! during modpack installation. The cache never expires (version_id is
//! immutable), so re-download is only needed if cache was cleared or
//! profile predates this caching mechanism.

use crate::pack::install_from::{PackFileHash, PackFormat};
use crate::state::profiles::{Profile, ProfileFile, ProjectType};
use crate::state::{CacheBehaviour, CachedEntry};
use crate::util::fetch::{FetchSemaphore, fetch_mirrors, sha1_async};
use async_zip::base::read::seek::ZipFileReader;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::io::Cursor;

/// Content item with rich metadata for frontend display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItem {
    /// Unique identifier (the file name)
    pub file_name: String,
    /// Relative path to the file within the profile
    pub file_path: String,
    /// SHA1 hash of the file
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is enabled (not .disabled)
    pub enabled: bool,
    /// Type of project (mod, resourcepack, etc.)
    pub project_type: ProjectType,
    /// Modrinth project info if recognized
    pub project: Option<ContentItemProject>,
    /// Version info if recognized
    pub version: Option<ContentItemVersion>,
    /// Owner info (organization or user)
    pub owner: Option<ContentItemOwner>,
    /// Whether an update is available
    pub has_update: bool,
    /// The recommended version ID to update to (if has_update is true)
    pub update_version_id: Option<String>,
    /// When the file was added to the instance (file modification time)
    pub date_added: Option<String>,
}

/// Project information for content item display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItemProject {
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub icon_url: Option<String>,
}

/// Version information for content item display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItemVersion {
    pub id: String,
    pub version_number: String,
    pub file_name: String,
    pub date_published: Option<String>,
}

/// Owner information for content item display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItemOwner {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    #[serde(rename = "type")]
    pub owner_type: OwnerType,
}

/// Type of content owner
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OwnerType {
    User,
    Organization,
}

use crate::state::cache::{Dependency, Organization, TeamMember};
use crate::state::{Project, Version};

/// Full linked modpack information including owner and update status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkedModpackInfo {
    pub project: Project,
    pub version: Version,
    pub owner: Option<ContentItemOwner>,
    /// Whether an update is available for this modpack
    pub has_update: bool,
    /// The version ID to update to (if has_update is true)
    pub update_version_id: Option<String>,
    /// The full version info for the update (if has_update is true)
    pub update_version: Option<Version>,
}

/// Get linked modpack info including project, version, owner, and update status.
/// Returns None if the profile is not linked to a modpack.
pub async fn get_linked_modpack_info(
    profile: &Profile,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Option<LinkedModpackInfo>> {
    let Some(linked_data) = &profile.linked_data else {
        return Ok(None);
    };

    // Fetch project, version, and all project versions in parallel
    let (project, version, all_versions) = tokio::try_join!(
        CachedEntry::get_project(
            &linked_data.project_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        ),
        CachedEntry::get_version(
            &linked_data.version_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        ),
        CachedEntry::get_project_versions(
            &linked_data.project_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        ),
    )?;

    let project = project.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Linked modpack project {} not found",
            linked_data.project_id
        ))
    })?;

    let version = version.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Linked modpack version {} not found",
            linked_data.version_id
        ))
    })?;

    // Resolve owner - prefer organization, fall back to team owner
    let owner = if let Some(org_id) = &project.organization {
        let org = CachedEntry::get_organization(
            org_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        )
        .await?;
        org.map(|o| ContentItemOwner {
            id: o.id,
            name: o.name,
            avatar_url: o.icon_url,
            owner_type: OwnerType::Organization,
        })
    } else {
        let team = CachedEntry::get_team(
            &project.team,
            cache_behaviour,
            pool,
            fetch_semaphore,
        )
        .await?;
        team.and_then(|t| {
            t.into_iter()
                .find(|m| m.is_owner)
                .map(|m| ContentItemOwner {
                    id: m.user.id,
                    name: m.user.username,
                    avatar_url: m.user.avatar_url,
                    owner_type: OwnerType::User,
                })
        })
    };

    // Check for updates
    let (has_update, update_version_id, update_version) = check_modpack_update(
        profile,
        &linked_data.version_id,
        &version,
        all_versions,
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

/// Check if a newer compatible version exists for the linked modpack.
/// Returns (has_update, update_version_id, update_version).
fn check_modpack_update(
    profile: &Profile,
    installed_version_id: &str,
    installed_version: &Version,
    all_versions: Option<Vec<Version>>,
) -> (bool, Option<String>, Option<Version>) {
    let Some(versions) = all_versions else {
        return (false, None, None);
    };

    // Get the loader as a string for comparison
    let loader_str = profile.loader.as_str().to_lowercase();
    let game_version = &profile.game_version;

    // Filter to compatible versions
    let mut compatible_versions: Vec<&Version> = versions
        .iter()
        .filter(|v| {
            // Must support the profile's game version
            let supports_game = v.game_versions.contains(game_version);

            // Must support the profile's loader
            // Modpacks list "mrpack" as a loader, but also list actual loaders
            let supports_loader = v.loaders.iter().any(|l| {
                let l_lower = l.to_lowercase();
                l_lower == loader_str || l_lower == "mrpack"
            });

            supports_game && supports_loader
        })
        .collect();

    // Sort by date_published descending (newest first)
    compatible_versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));

    // Find the newest compatible version
    if let Some(newest) = compatible_versions.first() {
        // Check if the newest version is different and newer than installed
        if newest.id != installed_version_id
            && newest.date_published > installed_version.date_published
        {
            return (true, Some(newest.id.clone()), Some((*newest).clone()));
        }
    }

    (false, None, None)
}

/// Get content items with rich metadata, filtered to exclude modpack content.
/// Returns only user-added content (not part of the linked modpack).
pub async fn get_content_items(
    profile: &Profile,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let all_files = profile
        .get_projects(cache_behaviour, pool, fetch_semaphore)
        .await?;

    let modpack_hashes: HashSet<String> = if let Some(ref linked_data) =
        profile.linked_data
    {
        match get_modpack_file_hashes(
            &linked_data.version_id,
            pool,
            fetch_semaphore,
        )
        .await
        {
            Ok(hashes) => hashes,
            Err(e) => {
                tracing::warn!("Failed to fetch modpack file hashes: {}", e);
                HashSet::new()
            }
        }
    } else {
        HashSet::new()
    };

    let user_files: Vec<(String, ProfileFile)> = all_files
        .into_iter()
        .filter(|(_, file)| !modpack_hashes.contains(&file.hash))
        .collect();

    profile_files_to_content_items(
        &profile.path,
        &user_files,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await
}

/// Pre-fetched metadata for projects, versions, teams, and organizations.
struct ResolvedMetadata {
    projects: Vec<Project>,
    versions: Vec<Version>,
    teams: Vec<Vec<TeamMember>>,
    organizations: Vec<Organization>,
}

/// Fetch project, version, team, and organization metadata in parallel batches.
async fn resolve_metadata(
    project_ids: &HashSet<String>,
    version_ids: &HashSet<String>,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<ResolvedMetadata> {
    let project_ids_vec: Vec<&str> =
        project_ids.iter().map(|s| s.as_str()).collect();
    let version_ids_vec: Vec<&str> =
        version_ids.iter().map(|s| s.as_str()).collect();

    let (projects, versions) =
        if !project_ids.is_empty() || !version_ids.is_empty() {
            tokio::try_join!(
                async {
                    if project_ids.is_empty() {
                        Ok(Vec::new())
                    } else {
                        CachedEntry::get_project_many(
                            &project_ids_vec,
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
                            &version_ids_vec,
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

    let team_ids: HashSet<String> =
        projects.iter().map(|p| p.team.clone()).collect();
    let org_ids: HashSet<String> = projects
        .iter()
        .filter_map(|p| p.organization.clone())
        .collect();

    let team_ids_vec: Vec<&str> = team_ids.iter().map(|s| s.as_str()).collect();
    let org_ids_vec: Vec<&str> = org_ids.iter().map(|s| s.as_str()).collect();

    let (teams, organizations) = if !team_ids.is_empty() || !org_ids.is_empty()
    {
        tokio::try_join!(
            async {
                if team_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    CachedEntry::get_team_many(
                        &team_ids_vec,
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
                        &org_ids_vec,
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

/// Shared helper: convert profile files to ContentItems with rich metadata.
/// Used by both `get_content_items` (user-added files) and
/// `get_linked_modpack_content` (modpack-bundled files).
async fn profile_files_to_content_items(
    profile_path: &str,
    files: &[(String, ProfileFile)],
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let project_ids: HashSet<String> = files
        .iter()
        .filter_map(|(_, f)| f.metadata.as_ref().map(|m| m.project_id.clone()))
        .collect();

    let version_ids: HashSet<String> = files
        .iter()
        .filter_map(|(_, f)| f.metadata.as_ref().map(|m| m.version_id.clone()))
        .collect();

    let meta = resolve_metadata(
        &project_ids,
        &version_ids,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await?;

    let profile_base_path =
        crate::api::profile::get_full_path(profile_path).await?;

    // Batch-read file modification times off the main async runtime
    let paths: Vec<std::path::PathBuf> = files
        .iter()
        .map(|(path, _)| profile_base_path.join(path))
        .collect();

    let modification_times: Vec<Option<String>> =
        tokio::task::spawn_blocking(move || {
            paths
                .iter()
                .map(|path| {
                    std::fs::metadata(path).and_then(|m| m.modified()).ok().map(
                        |t| {
                            chrono::DateTime::<chrono::Utc>::from(t)
                                .to_rfc3339()
                        },
                    )
                })
                .collect()
        })
        .await?;

    let mut items: Vec<ContentItem> = files
        .iter()
        .enumerate()
        .map(|(i, (path, file))| {
            let project = file.metadata.as_ref().and_then(|m| {
                meta.projects.iter().find(|p| p.id == m.project_id)
            });

            let version = file.metadata.as_ref().and_then(|m| {
                meta.versions.iter().find(|v| v.id == m.version_id)
            });

            let owner = project.and_then(|p| {
                resolve_owner(p, &meta.teams, &meta.organizations)
            });

            ContentItem {
                file_name: file.file_name.clone(),
                file_path: path.clone(),
                hash: file.hash.clone(),
                size: file.size,
                enabled: !file.file_name.ends_with(".disabled"),
                project_type: file.project_type,
                project: project.map(|p| ContentItemProject {
                    id: p.id.clone(),
                    slug: p.slug.clone(),
                    title: p.title.clone(),
                    icon_url: p.icon_url.clone(),
                }),
                version: version.map(|v| ContentItemVersion {
                    id: v.id.clone(),
                    version_number: v.version_number.clone(),
                    file_name: file.file_name.clone(),
                    date_published: Some(v.date_published.to_rfc3339()),
                }),
                owner,
                has_update: file.update_version_id.is_some(),
                update_version_id: file.update_version_id.clone(),
                date_added: modification_times[i].clone(),
            }
        })
        .collect();

    items.sort_by(|a, b| {
        let name_a = a
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&a.file_name);
        let name_b = b
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&b.file_name);
        name_a.to_lowercase().cmp(&name_b.to_lowercase())
    });

    Ok(items)
}

/// Resolve the owner of a project from pre-fetched teams and organizations.
fn resolve_owner(
    project: &Project,
    teams: &[Vec<TeamMember>],
    organizations: &[Organization],
) -> Option<ContentItemOwner> {
    if let Some(org_id) = &project.organization {
        organizations.iter().find(|o| &o.id == org_id).map(|o| {
            ContentItemOwner {
                id: o.id.clone(),
                name: o.name.clone(),
                avatar_url: o.icon_url.clone(),
                owner_type: OwnerType::Organization,
            }
        })
    } else {
        teams
            .iter()
            .find(|t| t.first().is_some_and(|m| m.team_id == project.team))
            .and_then(|t| t.iter().find(|m| m.is_owner))
            .map(|m| ContentItemOwner {
                id: m.user.id.clone(),
                name: m.user.username.clone(),
                avatar_url: m.user.avatar_url.clone(),
                owner_type: OwnerType::User,
            })
    }
}

/// Get content items that are part of the linked modpack (not user-added).
/// Returns modpack-bundled files with full on-disk metadata (file_path, enabled, etc).
/// Returns empty vec if the profile is not linked to a modpack.
pub async fn get_linked_modpack_content(
    profile: &Profile,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let Some(linked_data) = &profile.linked_data else {
        return Ok(Vec::new());
    };

    let all_files = profile
        .get_projects(cache_behaviour, pool, fetch_semaphore)
        .await?;

    let modpack_hashes: HashSet<String> = match get_modpack_file_hashes(
        &linked_data.version_id,
        pool,
        fetch_semaphore,
    )
    .await
    {
        Ok(hashes) => hashes,
        Err(e) => {
            tracing::warn!("Failed to fetch modpack file hashes: {}", e);
            return Ok(Vec::new());
        }
    };

    // Inverse of get_content_items: keep only modpack-bundled files
    let modpack_files: Vec<(String, ProfileFile)> = all_files
        .into_iter()
        .filter(|(_, file)| modpack_hashes.contains(&file.hash))
        .collect();

    profile_files_to_content_items(
        &profile.path,
        &modpack_files,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await
}

/// Convert a list of dependencies into ContentItems with rich metadata.
/// Fetches project, version, and owner info for each dependency.
pub async fn dependencies_to_content_items(
    dependencies: &[Dependency],
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let project_ids: HashSet<String> = dependencies
        .iter()
        .filter_map(|d| d.project_id.clone())
        .collect();

    if project_ids.is_empty() {
        return Ok(Vec::new());
    }

    let version_ids: HashSet<String> = dependencies
        .iter()
        .filter_map(|d| d.version_id.clone())
        .collect();

    let meta = resolve_metadata(
        &project_ids,
        &version_ids,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await?;

    let mut items: Vec<ContentItem> = dependencies
        .iter()
        .filter_map(|dep| {
            let project_id = dep.project_id.as_ref()?;
            let project = meta.projects.iter().find(|p| &p.id == project_id)?;

            let version = dep
                .version_id
                .as_ref()
                .and_then(|vid| meta.versions.iter().find(|v| &v.id == vid));

            let owner =
                resolve_owner(project, &meta.teams, &meta.organizations);

            let project_type = match project.project_type.as_str() {
                "mod" => ProjectType::Mod,
                "resourcepack" => ProjectType::ResourcePack,
                "shader" => ProjectType::ShaderPack,
                "datapack" => ProjectType::DataPack,
                _ => ProjectType::Mod,
            };

            Some(ContentItem {
                file_name: version
                    .and_then(|v| v.files.first())
                    .map(|f| f.filename.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{}.jar",
                            project.slug.as_deref().unwrap_or(&project.id)
                        )
                    }),
                file_path: String::new(),
                hash: String::new(),
                size: version
                    .and_then(|v| v.files.first())
                    .map(|f| f.size as u64)
                    .unwrap_or(0),
                enabled: true,
                project_type,
                project: Some(ContentItemProject {
                    id: project.id.clone(),
                    slug: project.slug.clone(),
                    title: project.title.clone(),
                    icon_url: project.icon_url.clone(),
                }),
                version: version.map(|v| ContentItemVersion {
                    id: v.id.clone(),
                    version_number: v.version_number.clone(),
                    file_name: v
                        .files
                        .first()
                        .map(|f| f.filename.clone())
                        .unwrap_or_default(),
                    date_published: Some(v.date_published.to_rfc3339()),
                }),
                owner,
                has_update: false,
                update_version_id: None,
                date_added: None,
            })
        })
        .collect();

    items.sort_by(|a, b| {
        let name_a = a
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&a.file_name);
        let name_b = b
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&b.file_name);
        name_a.to_lowercase().cmp(&name_b.to_lowercase())
    });

    Ok(items)
}

/// Gets SHA1 hashes of all files in a modpack version.
/// Checks cache first, falls back to downloading mrpack if not cached.
async fn get_modpack_file_hashes(
    version_id: &str,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<HashSet<String>> {
    if let Some(cached) =
        CachedEntry::get_modpack_files(version_id, pool).await?
    {
        return Ok(cached.file_hashes.into_iter().collect());
    }

    tracing::debug!(
        "Modpack files not cached, downloading mrpack for version {}",
        version_id
    );

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
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "No files found for modpack version {version_id}"
            ))
        })?;

    let mrpack_bytes = fetch_mirrors(
        &[&primary_file.url],
        primary_file.hashes.get("sha1").map(|s| s.as_str()),
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
        .position(|f| {
            matches!(f.filename().as_str(), Ok("modrinth.index.json"))
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

    let mut hashes: Vec<String> = pack
        .files
        .iter()
        .filter_map(|f| f.hashes.get(&PackFileHash::Sha1).cloned())
        .collect();

    // Also hash files from overrides folders (these aren't in modrinth.index.json)
    let override_entries: Vec<usize> = zip_reader
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
        .collect();

    for index in override_entries {
        let mut file_bytes = Vec::new();
        let mut entry_reader = zip_reader.reader_with_entry(index).await?;
        entry_reader.read_to_end_checked(&mut file_bytes).await?;

        let hash = sha1_async(bytes::Bytes::from(file_bytes)).await?;
        hashes.push(hash);
    }

    CachedEntry::cache_modpack_files(version_id, hashes.clone(), pool).await?;

    Ok(hashes.into_iter().collect())
}
