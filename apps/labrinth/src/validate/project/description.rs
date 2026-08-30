use super::text::{
	contains_banned_description_link, contains_spam,
	extract_description_blocks, extract_description_text,
	has_image_without_alt_text, has_long_header, has_sufficient_english_blocks,
	non_standard_text_ratio, profanity_count,
};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::exp::minecraft::Language;
use crate::models::projects::Project;

const MIN_DESCRIPTION_CHARS: usize = 125;
const MAX_PROFANITY_COUNT: usize = 2;
const NON_STANDARD_TEXT_FAILURE_THRESHOLD: f64 = 0.05;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
	let mut nags = Vec::new();
	let description = project.description.trim();
	let text = extract_description_text(description);
	let blocks = extract_description_blocks(description);

	if profanity_count(description) > MAX_PROFANITY_COUNT {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectDescriptionProfanity,
			ProjectNagSeverity::Required,
		));
	}
	if non_standard_text_ratio(description)
		>= NON_STANDARD_TEXT_FAILURE_THRESHOLD
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectDescriptionNonStandardText,
			ProjectNagSeverity::Required,
		));
	}
	if project_requires_english(project)
		&& text.chars().count() >= MIN_DESCRIPTION_CHARS
		&& !contains_spam(&text)
		&& !has_sufficient_english_blocks(&blocks)
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectDescriptionNonEnglish,
			ProjectNagSeverity::Warning,
		));
	}
	if description.is_empty() {
		nags.push(ProjectNag::new(
			ProjectNagKind::AddDescription,
			ProjectNagSeverity::Required,
		));
	} else if text.chars().count() < MIN_DESCRIPTION_CHARS {
		nags.push(ProjectNag::new(
			ProjectNagKind::DescriptionTooShort,
			ProjectNagSeverity::Required,
		));
	}
	if contains_spam(&text) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectDescriptionSpam,
			ProjectNagSeverity::Required,
		));
	}
	if contains_banned_description_link(description) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectDescriptionBannedLink,
			ProjectNagSeverity::Required,
		));
	}
	if has_long_header(description) {
		nags.push(ProjectNag::new(
			ProjectNagKind::LongHeaders,
			ProjectNagSeverity::Required,
		));
	}
	if has_image_without_alt_text(description) {
		nags.push(ProjectNag::new(
			ProjectNagKind::MissingAltText,
			ProjectNagSeverity::Warning,
		));
	}

	nags
}

fn project_requires_english(project: &Project) -> bool {
	project.components.minecraft_java_server.is_none()
		|| project
			.components
			.minecraft_server
			.as_ref()
			.is_some_and(|server| server.languages.contains(&Language::En))
}
