use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
	project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
	if project.icon_url.as_deref().is_none_or(str::is_empty) {
		vec![ProjectNag::new(
			ProjectNagKind::AddIcon,
			ProjectNagSeverity::Suggestion,
		)]
	} else {
		Vec::new()
	}
}
