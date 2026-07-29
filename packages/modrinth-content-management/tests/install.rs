#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use modrinth_content_management::{
        ContentMetadataProvider, ContentType, Dependency, DependencyType,
        Error, ResolutionPreferences, ResolveContentRequest, SkippedReason,
        Version, resolve_content,
    };

    const QUILT_FABRIC_API_EXCEPTION_PROJECT_ID: &str = "P7dR8mSH";

    #[derive(Default)]
    struct MemoryProvider {
        versions: HashMap<String, Version>,
        project_versions: HashMap<String, Vec<String>>,
    }

    #[async_trait]
    impl ContentMetadataProvider for MemoryProvider {
        async fn get_version(
            &mut self,
            version_id: &str,
        ) -> Result<Option<Version>, Error> {
            Ok(self.versions.get(version_id).cloned())
        }

        async fn get_project_versions(
            &mut self,
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

    impl MemoryProvider {
        fn with_versions(mut self, versions: Vec<Version>) -> Self {
            for version in versions {
                self.project_versions
                    .entry(version.project_id.clone())
                    .or_default()
                    .push(version.id.clone());
                self.versions.insert(version.id.clone(), version);
            }

            self
        }

        fn with_project_versions(
            mut self,
            project_id: &str,
            version_ids: &[&str],
        ) -> Self {
            self.project_versions.insert(
                project_id.to_string(),
                version_ids.iter().map(|id| id.to_string()).collect(),
            );
            self
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
            game_versions: game_versions
                .iter()
                .map(|v| v.to_string())
                .collect(),
            loaders: loaders.iter().map(|v| v.to_string()).collect(),
        }
    }

    fn dependency(
        project_id: Option<&str>,
        version_id: Option<&str>,
        dependency_type: DependencyType,
    ) -> Dependency {
        Dependency {
            version_id: version_id.map(str::to_string),
            project_id: project_id.map(str::to_string),
            file_name: None,
            dependency_type,
        }
    }

    fn required_project_dependency(project_id: &str) -> Dependency {
        dependency(Some(project_id), None, DependencyType::Required)
    }

    fn required_version_dependency(version_id: &str) -> Dependency {
        dependency(None, Some(version_id), DependencyType::Required)
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
    async fn explicit_primary_version_is_used() {
        let provider = MemoryProvider::default().with_versions(vec![version(
            "v1",
            "p1",
            "2024-01-01T00:00:00Z",
            &["1.20.1"],
            &["fabric"],
            vec![],
        )]);
        let mut request = request("p1");
        request.version_id = Some("v1".to_string());

        let plan = resolve_content(provider, request).await.unwrap();

        assert_eq!(plan.primary.version_id, "v1");
    }

    #[tokio::test]
    async fn newest_matching_primary_version_is_selected() {
        let provider = MemoryProvider::default().with_versions(vec![
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

        let plan = resolve_content(provider, request("p1")).await.unwrap();

        assert_eq!(plan.primary.version_id, "new");
    }

    #[tokio::test]
    async fn project_only_dependency_selects_matching_version() {
        let provider = MemoryProvider::default().with_versions(vec![
            version(
                "p1v1",
                "p1",
                "2024-01-01T00:00:00Z",
                &["1.20.1"],
                &["fabric"],
                vec![required_project_dependency("dep")],
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

        let plan = resolve_content(provider, request("p1")).await.unwrap();

        assert_eq!(plan.dependencies[0].project_id, "dep");
        assert_eq!(
            plan.dependencies[0].dependent_on_version_id.as_deref(),
            Some("p1v1")
        );
    }

    #[tokio::test]
    async fn exact_version_dependency_is_used_even_when_target_mismatches() {
        let provider = MemoryProvider::default().with_versions(vec![
            version(
                "p1v1",
                "p1",
                "2024-01-01T00:00:00Z",
                &["1.20.1"],
                &["fabric"],
                vec![required_version_dependency("depv1")],
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

        let plan = resolve_content(provider, request("p1")).await.unwrap();

        assert_eq!(plan.dependencies[0].version_id, "depv1");
    }

    #[tokio::test]
    async fn already_installed_dependencies_are_skipped() {
        let provider = MemoryProvider::default().with_versions(vec![
            version(
                "p1v1",
                "p1",
                "2024-01-01T00:00:00Z",
                &["1.20.1"],
                &["fabric"],
                vec![required_project_dependency("dep")],
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

        let plan = resolve_content(provider, request).await.unwrap();

        assert!(plan.dependencies.is_empty());
        assert_eq!(plan.skipped[0].reason, SkippedReason::AlreadyInstalled);
    }

    #[tokio::test]
    async fn duplicate_dependency_projects_are_skipped() {
        let provider = MemoryProvider::default().with_versions(vec![
            version(
                "p1v1",
                "p1",
                "2024-01-01T00:00:00Z",
                &["1.20.1"],
                &["fabric"],
                vec![
                    required_project_dependency("dep"),
                    required_project_dependency("dep"),
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
        ]);

        let plan = resolve_content(provider, request("p1")).await.unwrap();

        assert_eq!(plan.dependencies.len(), 1);
        assert_eq!(plan.skipped[0].reason, SkippedReason::DuplicateProject);
    }

    #[tokio::test]
    async fn conflicting_dependency_versions_are_skipped() {
        let provider = MemoryProvider::default().with_versions(vec![
            version(
                "p1v1",
                "p1",
                "2024-01-01T00:00:00Z",
                &["1.20.1"],
                &["fabric"],
                vec![
                    dependency(
                        Some("dep"),
                        Some("depv1"),
                        DependencyType::Required,
                    ),
                    dependency(
                        Some("dep"),
                        Some("depv2"),
                        DependencyType::Required,
                    ),
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

        let plan = resolve_content(provider, request("p1")).await.unwrap();

        assert_eq!(plan.dependencies.len(), 1);
        assert_eq!(
            plan.skipped[0].reason,
            SkippedReason::ConflictingDependency
        );
    }

    #[tokio::test]
    async fn quilt_instances_skip_fabric_api_dependency() {
        let provider = MemoryProvider::default().with_versions(vec![version(
            "p1v1",
            "p1",
            "2024-01-01T00:00:00Z",
            &["1.20.1"],
            &["quilt"],
            vec![required_project_dependency(
                QUILT_FABRIC_API_EXCEPTION_PROJECT_ID,
            )],
        )]);
        let mut request = request("p1");
        request.target.loaders = vec!["quilt".to_string()];

        let plan = resolve_content(provider, request).await.unwrap();

        assert!(plan.dependencies.is_empty());
        assert_eq!(plan.skipped[0].reason, SkippedReason::QuiltFabricApi);
    }

    #[tokio::test]
    async fn mods_can_fall_back_to_datapack_versions() {
        let provider = MemoryProvider::default().with_versions(vec![version(
            "p1v1",
            "p1",
            "2024-01-01T00:00:00Z",
            &["1.20.1"],
            &["datapack"],
            vec![],
        )]);

        let plan = resolve_content(provider, request("p1")).await.unwrap();

        assert_eq!(plan.primary.version_id, "p1v1");
    }

    #[tokio::test]
    async fn neoforge_matches_neo_loader_alias() {
        let provider = MemoryProvider::default().with_versions(vec![version(
            "p1v1",
            "p1",
            "2024-01-01T00:00:00Z",
            &["1.20.1"],
            &["neo"],
            vec![],
        )]);
        let mut request = request("p1");
        request.target.loaders = vec!["neoforge".to_string()];

        let plan = resolve_content(provider, request).await.unwrap();

        assert_eq!(plan.primary.version_id, "p1v1");
    }

    #[tokio::test]
    async fn paper_matches_bukkit_loader_alias() {
        let provider = MemoryProvider::default().with_versions(vec![version(
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

        let plan = resolve_content(provider, request).await.unwrap();

        assert_eq!(plan.primary.version_id, "p1v1");
    }

    #[tokio::test]
    async fn modpack_embedded_dependencies_are_not_installed() {
        let provider = MemoryProvider::default()
            .with_versions(vec![version(
                "Ur9uoHH5",
                "shFhR8Vx",
                "2025-01-19T06:39:41.349487Z",
                &["1.20.1"],
                &["fabric"],
                vec![
                    dependency(
                        Some("embedded-a"),
                        None,
                        DependencyType::Embedded,
                    ),
                    dependency(
                        Some("embedded-b"),
                        None,
                        DependencyType::Embedded,
                    ),
                    dependency(
                        Some("embedded-c"),
                        None,
                        DependencyType::Embedded,
                    ),
                ],
            )])
            .with_project_versions("better-mc-fabric-bmc2", &["Ur9uoHH5"]);

        let plan = resolve_content(
            provider,
            ResolveContentRequest {
                project_id: "better-mc-fabric-bmc2".to_string(),
                version_id: None,
                content_type: ContentType::ModPack,
                selected: ResolutionPreferences::default(),
                target: ResolutionPreferences::default(),
                existing_project_ids: Vec::new(),
            },
        )
        .await
        .unwrap();

        assert_eq!(plan.primary.project_id, "shFhR8Vx");
        assert_eq!(plan.primary.version_id, "Ur9uoHH5");
        assert!(plan.dependencies.is_empty());
        assert!(plan.skipped.is_empty());
    }

    #[tokio::test]
    async fn required_dependencies_resolve_transitively() {
        let provider = MemoryProvider::default().with_versions(vec![
            version(
                "pKFEfjEB",
                "DdAlVT8M",
                "2026-06-07T21:21:24.772638Z",
                &["1.21.1", "1.21.2"],
                &["neoforge"],
                vec![
                    dependency(
                        Some("LNytGWDc"),
                        Some("UjX6dr61"),
                        DependencyType::Required,
                    ),
                    dependency(
                        Some("oWaK0Q19"),
                        Some("YhZLrAFC"),
                        DependencyType::Required,
                    ),
                ],
            ),
            version(
                "UjX6dr61",
                "LNytGWDc",
                "2026-04-21T22:20:03.579201Z",
                &["1.21.1"],
                &["neoforge"],
                vec![],
            ),
            version(
                "YhZLrAFC",
                "oWaK0Q19",
                "2026-05-21T22:20:03.579201Z",
                &["1.21.1"],
                &["neoforge"],
                vec![
                    dependency(
                        Some("T9PomCSv"),
                        None,
                        DependencyType::Required,
                    ),
                    dependency(
                        Some("LNytGWDc"),
                        Some("UjX6dr61"),
                        DependencyType::Required,
                    ),
                ],
            ),
            version(
                "hyQUls27",
                "T9PomCSv",
                "2026-06-17T04:32:50.469591Z",
                &["1.21.1"],
                &["fabric"],
                vec![],
            ),
            version(
                "1L6XJqnY",
                "T9PomCSv",
                "2026-06-17T04:32:49.279302Z",
                &["1.21.1"],
                &["neoforge"],
                vec![],
            ),
        ]);

        let plan = resolve_content(
            provider,
            ResolveContentRequest {
                project_id: "DdAlVT8M".to_string(),
                version_id: Some("pKFEfjEB".to_string()),
                content_type: ContentType::Mod,
                selected: ResolutionPreferences::default(),
                target: ResolutionPreferences {
                    game_versions: vec![
                        "1.21.1".to_string(),
                        "1.21.2".to_string(),
                    ],
                    loaders: vec!["neoforge".to_string()],
                },
                existing_project_ids: Vec::new(),
            },
        )
        .await
        .unwrap();

        assert_eq!(plan.primary.project_id, "DdAlVT8M");
        assert_eq!(plan.primary.version_id, "pKFEfjEB");
        assert_eq!(plan.dependencies.len(), 3);
        assert!(plan.dependencies.iter().any(|dependency| {
            dependency.project_id == "LNytGWDc"
                && dependency.version_id == "UjX6dr61"
                && dependency.dependent_on_version_id.as_deref()
                    == Some("pKFEfjEB")
        }));
        assert!(plan.dependencies.iter().any(|dependency| {
            dependency.project_id == "oWaK0Q19"
                && dependency.version_id == "YhZLrAFC"
                && dependency.dependent_on_version_id.as_deref()
                    == Some("pKFEfjEB")
        }));
        assert!(plan.dependencies.iter().any(|dependency| {
            dependency.project_id == "T9PomCSv"
                && dependency.version_id == "1L6XJqnY"
                && dependency.dependent_on_version_id.as_deref()
                    == Some("YhZLrAFC")
        }));
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(
            plan.skipped[0].reason,
            modrinth_content_management::SkippedReason::DuplicateProject
        );
        assert_eq!(plan.skipped[0].project_id, "LNytGWDc");
    }
}
