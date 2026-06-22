use std::collections::{HashMap, HashSet};

use crate::model::{
	ContentType, Dependency, DependencyType, Error, ResolutionPreferences,
	ResolveContentPlan, ResolveContentRequest, ResolvedContent, SkippedContent,
	SkippedReason, Version,
};
use crate::provider::ContentMetadataProvider;

// Skip Fabric API if you're installing a fabric project onto a quilt instance.
const QUILT_FABRIC_API_EXCEPTION_PROJECT_ID: &str = "P7dR8mSH";

pub async fn resolve_content<P: ContentMetadataProvider + Sync>(
	provider: &P,
	request: ResolveContentRequest,
) -> Result<ResolveContentPlan, Error> {
	let primary_version =
		resolve_primary_version(provider, &request).await?;
	let primary = ResolvedContent {
		project_id: primary_version.project_id.clone(),
		version_id: primary_version.id.clone(),
		dependent_on_version_id: None,
	};
	let mut resolver = InstallResolver::new(provider, &request);
	resolver
		.resolve_dependencies_for_version(primary_version)
		.await?;

	Ok(ResolveContentPlan {
		primary,
		dependencies: resolver.dependencies,
		skipped: resolver.skipped,
	})
}

async fn resolve_primary_version<P: ContentMetadataProvider + Sync>(
	provider: &P,
	request: &ResolveContentRequest,
) -> Result<Version, Error> {
	if let Some(version_id) = &request.version_id {
		let version = provider
			.get_version(version_id)
			.await?
			.ok_or_else(|| Error::VersionNotFound(version_id.clone()))?;

		if version.project_id != request.project_id {
			return Err(Error::VersionProjectMismatch {
				version_id: version.id,
				project_id: request.project_id.clone(),
			});
		}

		return Ok(version);
	}

	let versions = provider.get_project_versions(&request.project_id).await?;
	if versions.is_empty() {
		return Err(Error::ProjectNotFound(request.project_id.clone()));
	}

	select_newest_matching_version(
		versions,
		request.content_type,
		&request.selected,
		&request.target,
	)
	.ok_or_else(|| Error::NoCompatibleVersion(request.project_id.clone()))
}

struct InstallResolver<'a, P> {
	provider: &'a P,
	content_type: ContentType,
	selected: &'a ResolutionPreferences,
	target: &'a ResolutionPreferences,
	existing_project_ids: HashSet<String>,
	planned_project_versions: HashMap<String, String>,
	visited_versions: HashSet<String>,
	dependencies: Vec<ResolvedContent>,
	skipped: Vec<SkippedContent>,
}

impl<'a, P: ContentMetadataProvider + Sync> InstallResolver<'a, P> {
	fn new(
		provider: &'a P,
		request: &'a ResolveContentRequest,
	) -> Self {
		let mut planned_project_versions = HashMap::new();
		planned_project_versions.insert(
			request.project_id.clone(),
			request.version_id.clone().unwrap_or_default(),
		);

		Self {
			provider,
			content_type: request.content_type,
			selected: &request.selected,
			target: &request.target,
			existing_project_ids: request
				.existing_project_ids
				.iter()
				.cloned()
				.collect(),
			planned_project_versions,
			visited_versions: HashSet::new(),
			dependencies: Vec::new(),
			skipped: Vec::new(),
		}
	}

	async fn resolve_dependencies_for_version(
		&mut self,
		version: Version,
	) -> Result<(), Error> {
		let mut stack = vec![version];

		while let Some(version) = stack.pop() {
			if !self.visited_versions.insert(version.id.clone()) {
				continue;
			}

			for dependency in &version.dependencies {
				if !matches!(dependency.dependency_type, DependencyType::Required)
				{
					continue;
				}

				if should_skip_quilt_fabric_api(dependency, self.target) {
					self.skipped.push(SkippedContent {
						project_id: QUILT_FABRIC_API_EXCEPTION_PROJECT_ID.to_string(),
						version_id: dependency.version_id.clone(),
						dependent_on_version_id: Some(version.id.clone()),
						reason: SkippedReason::QuiltFabricApi,
					});
					continue;
				}

				let Some(dependency_version) = self
					.resolve_dependency_version(dependency)
					.await?
				else {
					continue;
				};

				let project_id = dependency
					.project_id
					.clone()
					.unwrap_or_else(|| dependency_version.project_id.clone());

				if self.existing_project_ids.contains(&project_id) {
					self.skipped.push(SkippedContent {
						project_id,
						version_id: Some(dependency_version.id),
						dependent_on_version_id: Some(version.id.clone()),
						reason: SkippedReason::AlreadyInstalled,
					});
					continue;
				}

				if let Some(planned_version_id) =
					self.planned_project_versions.get(&project_id)
				{
					let reason = if planned_version_id.is_empty()
						|| planned_version_id == &dependency_version.id
					{
						SkippedReason::DuplicateProject
					} else {
						SkippedReason::ConflictingDependency
					};

					self.skipped.push(SkippedContent {
						project_id,
						version_id: Some(dependency_version.id),
						dependent_on_version_id: Some(version.id.clone()),
						reason,
					});
					continue;
				}

				self.planned_project_versions
					.insert(project_id.clone(), dependency_version.id.clone());
				self.dependencies.push(ResolvedContent {
					project_id,
					version_id: dependency_version.id.clone(),
					dependent_on_version_id: Some(version.id.clone()),
				});
				stack.push(dependency_version);
			}
		}

		Ok(())
	}

	async fn resolve_dependency_version(
		&mut self,
		dependency: &Dependency,
	) -> Result<Option<Version>, Error> {
		if let Some(version_id) = &dependency.version_id {
			let version = self.provider.get_version(version_id).await?;
			if version.is_none() {
				self.skipped.push(SkippedContent {
					project_id: dependency.project_id.clone().unwrap_or_default(),
					version_id: Some(version_id.clone()),
					dependent_on_version_id: None,
					reason: SkippedReason::MissingVersion,
				});
			}
			return Ok(version);
		}

		let Some(project_id) = &dependency.project_id else {
			return Ok(None);
		};
		let versions = self.provider.get_project_versions(project_id).await?;
		let version = select_newest_matching_version(
			versions,
			self.content_type,
			self.selected,
			self.target,
		);

		if version.is_none() {
			self.skipped.push(SkippedContent {
				project_id: project_id.clone(),
				version_id: None,
				dependent_on_version_id: None,
				reason: SkippedReason::NoCompatibleVersion,
			});
		}

		Ok(version)
	}
}

fn select_newest_matching_version(
	mut versions: Vec<Version>,
	content_type: ContentType,
	selected: &ResolutionPreferences,
	target: &ResolutionPreferences,
) -> Option<Version> {
	versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));
	let merged = selected.merge(target);

	versions
		.iter()
		.find(|version| version_matches(version, content_type, &merged))
		.or_else(|| {
			versions
				.iter()
				.find(|version| version_matches(version, content_type, target))
		})
		.cloned()
}

trait MergePreferences {
	fn merge(&self, target: &Self) -> Self;
}

impl MergePreferences for ResolutionPreferences {
	fn merge(&self, target: &Self) -> Self {
		Self {
			game_versions: if self.game_versions.is_empty() {
				target.game_versions.clone()
			} else {
				self.game_versions.clone()
			},
			loaders: if self.loaders.is_empty() {
				target.loaders.clone()
			} else {
				self.loaders.clone()
			},
		}
	}
}

fn version_matches(
	version: &Version,
	content_type: ContentType,
	preferences: &ResolutionPreferences,
) -> bool {
	matches_game_versions(version, preferences)
		&& matches_loaders(version, content_type, preferences)
}

fn matches_game_versions(
	version: &Version,
	preferences: &ResolutionPreferences,
) -> bool {
	preferences.game_versions.is_empty()
		|| preferences.game_versions.iter().any(|game_version| {
			version
				.game_versions
				.iter()
				.any(|candidate| candidate == game_version)
		})
}

fn matches_loaders(
	version: &Version,
	content_type: ContentType,
	preferences: &ResolutionPreferences,
) -> bool {
	if preferences.loaders.is_empty() {
		return true;
	}

	let direct_match = preferences.loaders.iter().any(|loader| {
		version
			.loaders
			.iter()
			.any(|candidate| loaders_match(loader, candidate))
	});

	if direct_match {
		return true;
	}

	content_type == ContentType::Mod
		&& version.loaders.iter().any(|loader| loader == "datapack")
}

fn loaders_match(expected: &str, candidate: &str) -> bool {
	let expected = expected.to_lowercase();
	let candidate = candidate.to_lowercase();

	expected == candidate
		|| loader_aliases(&expected).contains(&candidate.as_str())
		|| loader_aliases(&candidate).contains(&expected.as_str())
}

fn loader_aliases(loader: &str) -> &'static [&'static str] {
	match loader {
		"neoforge" => &["neo"],
		"neo" => &["neoforge"],
		"paper" | "purpur" | "spigot" | "bukkit" => {
			&["paper", "purpur", "spigot", "bukkit"]
		}
		_ => &[],
	}
}

fn should_skip_quilt_fabric_api(
	dependency: &Dependency,
	target: &ResolutionPreferences,
) -> bool {
	dependency.project_id.as_deref()
		== Some(QUILT_FABRIC_API_EXCEPTION_PROJECT_ID)
		&& target
			.loaders
			.iter()
			.any(|loader| loaders_match(loader, "quilt"))
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use async_trait::async_trait;
	use chrono::{DateTime, Utc};

	use super::*;

	#[derive(Default)]
	struct MemoryProvider {
		versions: HashMap<String, Version>,
		project_versions: HashMap<String, Vec<String>>,
	}

	#[async_trait]
	impl ContentMetadataProvider for MemoryProvider {
		async fn get_version(
			&self,
			version_id: &str,
		) -> Result<Option<Version>, Error> {
			Ok(self.versions.get(version_id).cloned())
		}

		async fn get_project_versions(
			&self,
			project_id: &str,
		) -> Result<Vec<Version>, Error> {
			Ok(self
				.project_versions
				.get(project_id)
				.into_iter()
				.flatten()
				.filter_map(|id| self.versions.get(id))
				.cloned()
				.collect())
		}
	}

	fn version(
		id: &str,
		project_id: &str,
		date: &str,
		game_versions: &[&str],
		loaders: &[&str],
		dependencies: Vec<Dependency>,
	) -> Version {
		Version {
			id: id.to_string(),
			project_id: project_id.to_string(),
			date_published: DateTime::parse_from_rfc3339(date)
				.unwrap()
				.with_timezone(&Utc),
			dependencies,
			game_versions: game_versions.iter().map(|v| v.to_string()).collect(),
			loaders: loaders.iter().map(|v| v.to_string()).collect(),
		}
	}

	fn dependency(project_id: &str) -> Dependency {
		Dependency {
			version_id: None,
			project_id: Some(project_id.to_string()),
			file_name: None,
			dependency_type: DependencyType::Required,
		}
	}

	fn exact_dependency(version_id: &str) -> Dependency {
		Dependency {
			version_id: Some(version_id.to_string()),
			project_id: None,
			file_name: None,
			dependency_type: DependencyType::Required,
		}
	}

	fn provider(versions: Vec<Version>) -> MemoryProvider {
		let mut provider = MemoryProvider::default();
		for version in versions {
			provider
				.project_versions
				.entry(version.project_id.clone())
				.or_default()
				.push(version.id.clone());
			provider.versions.insert(version.id.clone(), version);
		}
		provider
	}

	fn request(project_id: &str) -> ResolveContentRequest {
		ResolveContentRequest {
			project_id: project_id.to_string(),
			version_id: None,
			content_type: ContentType::Mod,
			selected: ResolutionPreferences::default(),
			target: ResolutionPreferences {
				game_versions: vec!["1.20.1".to_string()],
				loaders: vec!["fabric".to_string()],
			},
			existing_project_ids: Vec::new(),
		}
	}

	#[tokio::test]
	async fn explicit_primary_version() {
		let provider = provider(vec![version(
			"v1",
			"p1",
			"2024-01-01T00:00:00Z",
			&["1.20.1"],
			&["fabric"],
			vec![],
		)]);
		let mut request = request("p1");
		request.version_id = Some("v1".to_string());

		let plan = resolve_content(&provider, request).await.unwrap();

		assert_eq!(plan.primary.version_id, "v1");
	}

	#[tokio::test]
	async fn target_selected_primary_version() {
		let provider = provider(vec![
			version(
				"old",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
			version(
				"new",
				"p1",
				"2024-02-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
		]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.primary.version_id, "new");
	}

	#[tokio::test]
	async fn project_only_dependency() {
		let provider = provider(vec![
			version(
				"p1v1",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![dependency("dep")],
			),
			version(
				"depv1",
				"dep",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
		]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.dependencies[0].project_id, "dep");
		assert_eq!(
			plan.dependencies[0].dependent_on_version_id.as_deref(),
			Some("p1v1")
		);
	}

	#[tokio::test]
	async fn exact_version_dependency() {
		let provider = provider(vec![
			version(
				"p1v1",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![exact_dependency("depv1")],
			),
			version(
				"depv1",
				"dep",
				"2024-01-01T00:00:00Z",
				&["1.19.4"],
				&["forge"],
				vec![],
			),
		]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.dependencies[0].version_id, "depv1");
	}

	#[tokio::test]
	async fn transitive_dependency_closure() {
		let provider = provider(vec![
			version(
				"p1v1",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![dependency("dep1")],
			),
			version(
				"dep1v1",
				"dep1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![dependency("dep2")],
			),
			version(
				"dep2v1",
				"dep2",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
		]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.dependencies.len(), 2);
		assert!(plan.dependencies.iter().any(|dep| dep.project_id == "dep2"));
	}

	#[tokio::test]
	async fn already_existing_dependency_skip() {
		let provider = provider(vec![
			version(
				"p1v1",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![dependency("dep")],
			),
			version(
				"depv1",
				"dep",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
		]);
		let mut request = request("p1");
		request.existing_project_ids = vec!["dep".to_string()];

		let plan = resolve_content(&provider, request).await.unwrap();

		assert!(plan.dependencies.is_empty());
		assert_eq!(plan.skipped[0].reason, SkippedReason::AlreadyInstalled);
	}

	#[tokio::test]
	async fn duplicate_dependency_project_skip() {
		let provider = provider(vec![
			version(
				"p1v1",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![dependency("dep"), dependency("dep")],
			),
			version(
				"depv1",
				"dep",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
		]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.dependencies.len(), 1);
		assert_eq!(plan.skipped[0].reason, SkippedReason::DuplicateProject);
	}

	#[tokio::test]
	async fn conflicting_dependency_version_behavior() {
		let provider = provider(vec![
			version(
				"p1v1",
				"p1",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![
					Dependency {
						project_id: Some("dep".to_string()),
						..exact_dependency("depv1")
					},
					Dependency {
						project_id: Some("dep".to_string()),
						..exact_dependency("depv2")
					},
				],
			),
			version(
				"depv1",
				"dep",
				"2024-01-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
			version(
				"depv2",
				"dep",
				"2024-02-01T00:00:00Z",
				&["1.20.1"],
				&["fabric"],
				vec![],
			),
		]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.dependencies.len(), 1);
		assert_eq!(plan.skipped[0].reason, SkippedReason::ConflictingDependency);
	}

	#[tokio::test]
	async fn quilt_fabric_api_exception() {
		let provider = provider(vec![version(
			"p1v1",
			"p1",
			"2024-01-01T00:00:00Z",
			&["1.20.1"],
			&["quilt"],
			vec![dependency(QUILT_FABRIC_API_EXCEPTION_PROJECT_ID)],
		)]);
		let mut request = request("p1");
		request.target.loaders = vec!["quilt".to_string()];

		let plan = resolve_content(&provider, request).await.unwrap();

		assert!(plan.dependencies.is_empty());
		assert_eq!(plan.skipped[0].reason, SkippedReason::QuiltFabricApi);
	}

	#[tokio::test]
	async fn datapack_fallback() {
		let provider = provider(vec![version(
			"p1v1",
			"p1",
			"2024-01-01T00:00:00Z",
			&["1.20.1"],
			&["datapack"],
			vec![],
		)]);

		let plan = resolve_content(&provider, request("p1")).await.unwrap();

		assert_eq!(plan.primary.version_id, "p1v1");
	}

	#[tokio::test]
	async fn neoforge_alias() {
		let provider = provider(vec![version(
			"p1v1",
			"p1",
			"2024-01-01T00:00:00Z",
			&["1.20.1"],
			&["neo"],
			vec![],
		)]);
		let mut request = request("p1");
		request.target.loaders = vec!["neoforge".to_string()];

		let plan = resolve_content(&provider, request).await.unwrap();

		assert_eq!(plan.primary.version_id, "p1v1");
	}

	#[tokio::test]
	async fn paper_purpur_spigot_bukkit_alias() {
		let provider = provider(vec![version(
			"p1v1",
			"p1",
			"2024-01-01T00:00:00Z",
			&["1.20.1"],
			&["bukkit"],
			vec![],
		)]);
		let mut request = request("p1");
		request.content_type = ContentType::Plugin;
		request.target.loaders = vec!["paper".to_string()];

		let plan = resolve_content(&provider, request).await.unwrap();

		assert_eq!(plan.primary.version_id, "p1v1");
	}

}
