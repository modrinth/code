use async_trait::async_trait;
use modrinth_content_management::{
	ContentMetadataProvider, ContentType, DependencyType, Error,
	ResolutionPreferences, ResolveContentPlan, ResolveContentRequest, Version,
	resolve_content,
};
use serde::Deserialize;

struct ModrinthApiProvider {
	client: reqwest::Client,
}

#[derive(Deserialize)]
struct ApiVersion {
	version_number: String,
	#[serde(flatten)]
	version: Version,
}

impl ModrinthApiProvider {
	fn new() -> Self {
		Self {
			client: reqwest::Client::builder()
				.user_agent(
					"modrinth-content-management-test/0.0.0 (support@modrinth.com)",
				)
				.build()
				.unwrap(),
		}
	}

	async fn get_project_api_versions(
		&self,
		project_id: &str,
	) -> Result<Vec<ApiVersion>, Error> {
		println!(
			"GET /v2/project/{project_id}/version?include_changelog=false"
		);
		let response = self
			.client
			.get(format!(
				"https://api.modrinth.com/v2/project/{project_id}/version"
			))
			.query(&[("include_changelog", "false")])
			.send()
			.await
			.map_err(|err| Error::Provider(err.to_string()))?;
		let status = response.status();
		println!("GET /v2/project/{project_id}/version -> {status}");

		if response.status() == reqwest::StatusCode::NOT_FOUND {
			return Ok(Vec::new());
		}

		let response = response
			.error_for_status()
			.map_err(|err| Error::Provider(err.to_string()))?;

		let versions = response
			.json::<Vec<ApiVersion>>()
			.await
			.map_err(|err| Error::Provider(err.to_string()))?;
		println!(
			"project {project_id} returned {} versions",
			versions.len()
		);
		for version in versions.iter().take(5) {
			println!(
				"candidate version_number={} id={} published={} game_versions={:?} loaders={:?} dependencies={} ({})",
				version.version_number,
				version.version.id,
				version.version.date_published,
				version.version.game_versions,
				version.version.loaders,
				version.version.dependencies.len(),
				dependency_type_counts(&version.version)
			);
		}

		Ok(versions)
	}
}

fn dependency_type_counts(version: &Version) -> String {
	let mut required = 0;
	let mut optional = 0;
	let mut incompatible = 0;
	let mut embedded = 0;

	for dependency in &version.dependencies {
		match dependency.dependency_type {
			DependencyType::Required => required += 1,
			DependencyType::Optional => optional += 1,
			DependencyType::Incompatible => incompatible += 1,
			DependencyType::Embedded => embedded += 1,
		}
	}

	format!(
		"required={required}, optional={optional}, incompatible={incompatible}, embedded={embedded}"
	)
}

#[async_trait]
impl ContentMetadataProvider for ModrinthApiProvider {
	async fn get_version(
		&self,
		version_id: &str,
	) -> Result<Option<Version>, Error> {
		println!("GET /v2/version/{version_id}");
		let response = self
			.client
			.get(format!(
				"https://api.modrinth.com/v2/version/{version_id}"
			))
			.send()
			.await
			.map_err(|err| Error::Provider(err.to_string()))?;
		let status = response.status();
		println!("GET /v2/version/{version_id} -> {status}");

		if response.status() == reqwest::StatusCode::NOT_FOUND {
			return Ok(None);
		}

		let response = response
			.error_for_status()
			.map_err(|err| Error::Provider(err.to_string()))?;

		let version = response
			.json::<Version>()
			.await
			.map_err(|err| Error::Provider(err.to_string()))?;
		println!(
			"version {} project={} game_versions={:?} loaders={:?} dependencies={} ({})",
			version.id,
			version.project_id,
			version.game_versions,
			version.loaders,
			version.dependencies.len(),
			dependency_type_counts(&version)
		);

		Ok(Some(version))
	}

	async fn get_project_versions(
		&self,
		project_id: &str,
	) -> Result<Vec<Version>, Error> {
		self.get_project_api_versions(project_id)
			.await
			.map(|versions| {
				versions
					.into_iter()
					.map(|version| version.version)
					.collect()
			})
	}
}

fn print_plan(plan: &ResolveContentPlan) {
	println!("resolved primary: {:#?}", plan.primary);
	println!("resolved dependencies: {}", plan.dependencies.len());
	for dependency in &plan.dependencies {
		println!("{dependency:#?}");
	}
	println!("skipped entries: {}", plan.skipped.len());
	for skipped in &plan.skipped {
		println!("{skipped:#?}");
	}
}

#[tokio::test]
#[ignore = "calls api.modrinth.com"]
async fn resolves_bmc2_modpack_from_modrinth_api() {
	let provider = ModrinthApiProvider::new();
	let request = ResolveContentRequest {
		project_id: "better-mc-fabric-bmc2".to_string(),
		version_id: None,
		content_type: ContentType::ModPack,
		selected: ResolutionPreferences::default(),
		target: ResolutionPreferences::default(),
		existing_project_ids: Vec::new(),
	};

	println!("resolve_content request: {request:#?}");
	let plan = resolve_content(&provider, request).await.unwrap();
	print_plan(&plan);

	assert!(!plan.primary.project_id.is_empty());
	assert!(!plan.primary.version_id.is_empty());
	assert!(plan.primary.dependent_on_version_id.is_none());
}

#[tokio::test]
#[ignore = "calls api.modrinth.com"]
async fn resolves_create_aeronautics_encased_fluid_pipes_from_modrinth_api() {
	let provider = ModrinthApiProvider::new();
	let project_versions = provider
		.get_project_api_versions("create-aeronautics-encased-fluid-pipes")
		.await
		.unwrap();
	let version = project_versions
		.into_iter()
		.find(|version| version.version_number == "1.0.7")
		.expect("expected version 1.0.7 to exist");
	let target = ResolutionPreferences {
		game_versions: version.version.game_versions.clone(),
		loaders: version.version.loaders.clone(),
	};
	let request = ResolveContentRequest {
		project_id: version.version.project_id.clone(),
		version_id: Some(version.version.id.clone()),
		content_type: ContentType::Mod,
		selected: ResolutionPreferences::default(),
		target,
		existing_project_ids: Vec::new(),
	};

	println!("selected version_number=1.0.7 id={}", version.version.id);
	println!("resolve_content request: {request:#?}");
	let plan = resolve_content(&provider, request).await.unwrap();
	print_plan(&plan);

	assert_eq!(plan.primary.version_id, version.version.id);
	assert!(plan.primary.dependent_on_version_id.is_none());
	assert!(
		!plan.dependencies.is_empty(),
		"expected Create Aeronautics Encased Fluid Pipes 1.0.7 to resolve required dependencies"
	);
}
