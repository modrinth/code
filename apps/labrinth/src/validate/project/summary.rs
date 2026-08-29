use super::text::{
	contains_link_or_ip, contains_profanity, contains_spam,
	has_non_standard_text, has_summary_formatting, is_likely_english,
};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::exp::minecraft::Language;
use crate::models::projects::Project;

const MIN_SUMMARY_CHARS: usize = 25;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
	let mut nags = Vec::new();
	let summary = project.summary.trim();
	let contains_link = contains_link_or_ip(summary);

	if contains_profanity(summary) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectSummaryProfanity,
			ProjectNagSeverity::Required,
		));
	}
	if has_non_standard_text(summary) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectSummaryNonStandardText,
			ProjectNagSeverity::Required,
		));
	}
	if project_requires_english(project)
		&& summary.chars().count() >= MIN_SUMMARY_CHARS
		&& !contains_link
		&& !contains_spam(summary)
		&& !is_likely_english(summary)
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectSummaryNonEnglish,
			ProjectNagSeverity::Warning,
		));
	}
	if !summary.is_empty()
		&& !contains_link
		&& normalized_without_whitespace(summary)
			== normalized_without_whitespace(&project.name)
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectSummaryMatchesTitle,
			ProjectNagSeverity::Required,
		));
	}
	if !summary.is_empty()
		&& !contains_link
		&& summary.chars().count() < MIN_SUMMARY_CHARS
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::SummaryTooShort,
			ProjectNagSeverity::Required,
		));
	}
	if contains_spam(summary) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectSummarySpam,
			ProjectNagSeverity::Required,
		));
	}
	if !summary.is_empty() && has_summary_formatting(summary) {
		nags.push(ProjectNag::new(
			ProjectNagKind::SummarySpecialFormatting,
			ProjectNagSeverity::Required,
		));
	}
	if !summary.is_empty() && contains_link {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectSummaryLinks,
			ProjectNagSeverity::Required,
		));
	}

	nags
}

fn normalized_without_whitespace(text: &str) -> String {
	text.trim()
		.to_lowercase()
		.chars()
		.filter(|character| !character.is_whitespace())
		.collect()
}

fn project_requires_english(project: &Project) -> bool {
	project.components.minecraft_java_server.is_none()
		|| project
			.components
			.minecraft_server
			.as_ref()
			.is_some_and(|server| server.languages.contains(&Language::En))
}
