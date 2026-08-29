use crate::models::{exp::minecraft::ServerContentQuery, projects::Project};

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_LANGUAGE_COUNT: usize = 10;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
	let mut nags = Vec::new();
	let server = project.components.minecraft_server.as_ref();

	if server.is_some_and(|server| server.region.is_none()) {
		nags.push(ProjectNag::new(
			ProjectNagKind::SelectCountry,
			ProjectNagSeverity::Required,
		));
	}

	if server.is_some()
		&& project
			.components
			.minecraft_java_server
			.as_ref()
			.is_none_or(|server| server.address.is_empty())
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::AddJavaAddress,
			ProjectNagSeverity::Required,
		));
	}

	if project
		.components
		.minecraft_java_server
		.as_ref()
		.is_some_and(|server| {
			matches!(
				&server.content,
				ServerContentQuery::Vanilla {
					recommended_game_version,
					..
				} if recommended_game_version
					.as_deref()
					.is_none_or(str::is_empty)
			)
		}) {
		nags.push(ProjectNag::new(
			ProjectNagKind::SelectCompatibility,
			ProjectNagSeverity::Required,
		));
	}

	if server.is_some_and(|server| server.languages.len() > MAX_LANGUAGE_COUNT)
	{
		nags.push(ProjectNag::new(
			ProjectNagKind::TooManyLanguages,
			ProjectNagSeverity::Warning,
		));
	}

	if server.is_some_and(|server| server.languages.is_empty()) {
		nags.push(ProjectNag::new(
			ProjectNagKind::SelectLanguage,
			ProjectNagSeverity::Suggestion,
		));
	}

	nags
}
