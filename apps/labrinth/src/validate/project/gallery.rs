use super::text::{contains_profanity, has_non_standard_text};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

fn validate_text(text: Option<&str>) -> Vec<ProjectNag> {
	let text = text.unwrap_or_default();
	let mut nags = Vec::new();

	if contains_profanity(text) {
		nags.push(ProjectNag::new(
			ProjectNagKind::GalleryTextProfanity,
			ProjectNagSeverity::Required,
		));
	}

	if has_non_standard_text(text) {
		nags.push(ProjectNag::new(
			ProjectNagKind::GalleryTextNonStandard,
			ProjectNagSeverity::Required,
		));
	}

	nags
}

pub(super) fn validate(
	project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
	let mut nags = Vec::new();
	let gallery_is_empty = project.gallery.is_empty();
	let is_shader = project.project_types.iter().any(|ty| ty == "shader");
	let is_resource_pack =
		project.project_types.iter().any(|ty| ty == "resourcepack");
	let has_gallery_exemption = project
		.categories
		.iter()
		.chain(&project.additional_categories)
		.any(|category| category == "audio" || category == "locale");

	if (is_shader && project.gallery.len() < 3)
		|| (is_resource_pack && gallery_is_empty && !has_gallery_exemption)
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::UploadGalleryImage,
			ProjectNagSeverity::Required,
		));
	}

	let is_minecraft_server = project.components.minecraft_server.is_some();
	if !is_minecraft_server && !project.gallery.iter().any(|item| item.featured)
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::FeatureGalleryImage,
			ProjectNagSeverity::Suggestion,
		));
	}

	for item in &project.gallery {
		nags.extend(validate_text(item.name.as_deref()));
		nags.extend(validate_text(item.description.as_deref()));
	}

	nags
}
