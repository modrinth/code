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
use crate::util::fetch::{fetch_mirrors, FetchSemaphore};
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

/// Get content items with rich metadata, filtered to exclude modpack content.
/// Returns only user-added content (not part of the linked modpack).
pub async fn get_content_items(
	profile: &Profile,
	cache_behaviour: Option<CacheBehaviour>,
	pool: &SqlitePool,
	fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
	let all_files =
		profile.get_projects(cache_behaviour, pool, fetch_semaphore).await?;

	let modpack_hashes: HashSet<String> =
		if let Some(ref linked_data) = profile.linked_data {
			match get_modpack_file_hashes(
				&linked_data.version_id,
				pool,
				fetch_semaphore,
			)
			.await
			{
				Ok(hashes) => hashes,
				Err(e) => {
					tracing::warn!(
						"Failed to fetch modpack file hashes: {}",
						e
					);
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

	let project_ids: HashSet<String> = user_files
		.iter()
		.filter_map(|(_, f)| f.metadata.as_ref().map(|m| m.project_id.clone()))
		.collect();

	let version_ids: HashSet<String> = user_files
		.iter()
		.filter_map(|(_, f)| f.metadata.as_ref().map(|m| m.version_id.clone()))
		.collect();

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

	let team_ids_vec: Vec<&str> =
		team_ids.iter().map(|s| s.as_str()).collect();
	let org_ids_vec: Vec<&str> =
		org_ids.iter().map(|s| s.as_str()).collect();

	let (teams, organizations) =
		if !team_ids.is_empty() || !org_ids.is_empty() {
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

	let mut items: Vec<ContentItem> = user_files
		.iter()
		.map(|(path, file)| {
			let project = file
				.metadata
				.as_ref()
				.and_then(|m| projects.iter().find(|p| p.id == m.project_id));

			let version = file
				.metadata
				.as_ref()
				.and_then(|m| versions.iter().find(|v| v.id == m.version_id));

			let owner = project.and_then(|p| {
				if let Some(org_id) = &p.organization {
					organizations
						.iter()
						.find(|o| &o.id == org_id)
						.map(|o| ContentItemOwner {
							id: o.id.clone(),
							name: o.name.clone(),
							avatar_url: o.icon_url.clone(),
							owner_type: OwnerType::Organization,
						})
				} else {
					teams
						.iter()
						.find(|t| {
							t.first().is_some_and(|m| m.team_id == p.team)
						})
						.and_then(|t| t.iter().find(|m| m.is_owner))
						.map(|m| ContentItemOwner {
							id: m.user.id.clone(),
							name: m.user.username.clone(),
							avatar_url: m.user.avatar_url.clone(),
							owner_type: OwnerType::User,
						})
				}
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
				}),
				owner,
				has_update: file.update_version_id.is_some(),
				update_version_id: file.update_version_id.clone(),
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

/// Gets SHA1 hashes of all files in a modpack version.
/// Checks cache first, falls back to downloading mrpack if not cached.
async fn get_modpack_file_hashes(
	version_id: &str,
	pool: &SqlitePool,
	fetch_semaphore: &FetchSemaphore,
) -> crate::Result<HashSet<String>> {
	if let Some(cached) = CachedEntry::get_modpack_files(version_id, pool).await?
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
		.position(|f| matches!(f.filename().as_str(), Ok("modrinth.index.json")))
		.ok_or_else(|| {
			crate::ErrorKind::InputError(
				"No modrinth.index.json found in mrpack".to_string(),
			)
		})?;

	let mut manifest = String::new();
	let mut entry_reader =
		zip_reader.reader_with_entry(manifest_idx).await?;
	entry_reader.read_to_string_checked(&mut manifest).await?;

	let pack: PackFormat = serde_json::from_str(&manifest)?;

	let hashes: Vec<String> = pack
		.files
		.iter()
		.filter_map(|f| f.hashes.get(&PackFileHash::Sha1).cloned())
		.collect();

	CachedEntry::cache_modpack_files(version_id, hashes.clone(), pool)
		.await?;

	Ok(hashes.into_iter().collect())
}
