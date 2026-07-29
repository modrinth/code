use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use crate::model::{
    ContentType, Dependency, DependencyType, Error, ResolutionPreferences,
    ResolveContentPlan, ResolveContentRequest, ResolvedContent, SkippedContent,
    SkippedReason, Version,
};
use crate::provider::ContentMetadataProvider;

// Skip Fabric API if you're installing a fabric project onto a quilt instance.
const QUILT_FABRIC_API_EXCEPTION_PROJECT_ID: &str = "P7dR8mSH";

pub async fn resolve_content<P: ContentMetadataProvider>(
    mut provider: P,
    request: ResolveContentRequest,
) -> Result<ResolveContentPlan, Error> {
    let primary_version =
        resolve_primary_version(&mut provider, &request).await?;
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

async fn resolve_primary_version<P: ContentMetadataProvider>(
    provider: &mut P,
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
    provider: P,
    content_type: ContentType,
    selected: &'a ResolutionPreferences,
    target: &'a ResolutionPreferences,
    existing_project_ids: HashSet<String>,
    planned_project_versions: HashMap<String, String>,
    visited_versions: HashSet<String>,
    dependencies: Vec<ResolvedContent>,
    skipped: Vec<SkippedContent>,
}

impl<'a, P: ContentMetadataProvider> InstallResolver<'a, P> {
    fn new(provider: P, request: &'a ResolveContentRequest) -> Self {
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
                if !matches!(
                    dependency.dependency_type,
                    DependencyType::Required
                ) {
                    continue;
                }

                if should_skip_quilt_fabric_api(dependency, self.target) {
                    self.skipped.push(SkippedContent {
                        project_id: QUILT_FABRIC_API_EXCEPTION_PROJECT_ID
                            .to_string(),
                        version_id: dependency.version_id.clone(),
                        dependent_on_version_id: Some(version.id.clone()),
                        reason: SkippedReason::QuiltFabricApi,
                    });
                    continue;
                }

                let Some(dependency_version) =
                    self.resolve_dependency_version(dependency).await?
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
                    project_id: dependency
                        .project_id
                        .clone()
                        .unwrap_or_default(),
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
    versions.sort_by_key(|version| Reverse(version.date_published));
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
