use std::sync::LazyLock;

use regex::Regex;

use super::text::{contains_profanity, has_non_standard_text};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::projects::Project;

static VERSION_NUMBER: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"\d+(?:\.\d+)+").unwrap());
static PORT_OR_FORK: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?i)\b(?:port|fork)\b").unwrap());

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
	let mut nags = Vec::new();
	let name = &project.name;

	if contains_profanity(name) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectNameProfanity,
			ProjectNagSeverity::Required,
		));
	}
	if has_non_standard_text(name) {
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectNameNonStandardText,
			ProjectNagSeverity::Required,
		));
	}

	let normalized = name.to_lowercase();
	if VERSION_NUMBER
		.find_iter(&normalized)
		.any(|version| !PORT_OR_FORK.is_match(&normalized[version.end()..]))
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::ProjectNameVersion,
			ProjectNagSeverity::Required,
		));
	}
	if normalized.contains("minecraft")
		&& normalized.split_whitespace().count() <= 3
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::MinecraftTitleClause,
			ProjectNagSeverity::Required,
		));
	}

	nags
}
