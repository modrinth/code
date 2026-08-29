use crate::models::projects::Project;

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_TAG_COUNT: usize = 8;
const MAX_TAG_COUNT_SERVER: usize = 18;
const RESOLUTION_TAGS: [&str; 8] =
	["8x-", "16x", "32x", "48x", "64x", "128x", "256x", "512x+"];

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
	let mut nags = Vec::new();
	let tag_count =
		project.categories.len() + project.additional_categories.len();
	let is_minecraft_server = project.components.minecraft_server.is_some();

	if !project.versions.is_empty() && project.categories.is_empty() {
		nags.push(ProjectNag::new(
			ProjectNagKind::SelectTags,
			ProjectNagSeverity::Suggestion,
		));
	}

	if !is_minecraft_server && tag_count > MAX_TAG_COUNT {
		nags.push(ProjectNag::new(
			ProjectNagKind::TooManyTags,
			ProjectNagSeverity::Warning,
		));
	}

	if is_minecraft_server && tag_count > MAX_TAG_COUNT_SERVER {
		nags.push(ProjectNag::new(
			ProjectNagKind::TooManyTagsServer,
			ProjectNagSeverity::Required,
		));
	}

	if project
		.project_types
		.iter()
		.any(|project_type| project_type == "resourcepack")
		&& project
			.categories
			.iter()
			.chain(&project.additional_categories)
			.filter(|tag| RESOLUTION_TAGS.contains(&tag.as_str()))
			.count() > 1
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::MultipleResolutionTags,
			ProjectNagSeverity::Warning,
		));
	}

	nags
}
