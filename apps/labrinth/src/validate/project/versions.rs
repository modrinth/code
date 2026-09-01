use crate::models::projects::{Project, Version};

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
	project: &Project,
	versions: &[Version],
) -> Vec<ProjectNag> {
	let mut nags = Vec::new();

	if project.versions.is_empty()
		&& project.components.minecraft_server.is_none()
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::UploadVersion,
			ProjectNagSeverity::Required,
		));
	}

	let requires_environment = project
		.project_types
		.iter()
		.any(|project_type| matches!(project_type.as_str(), "mod" | "modpack"));
	if requires_environment
		&& versions.iter().any(|version| {
			version
				.fields
				.get("environment")
				.and_then(serde_json::Value::as_str)
				.is_none_or(|environment| environment.trim().is_empty())
		}) {
		nags.push(ProjectNag::new(
			ProjectNagKind::SelectEnvironment,
			ProjectNagSeverity::Required,
		));
	}

	nags
}
