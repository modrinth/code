use crate::models::projects::ProjectStatus;

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
	project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
	if matches!(
		project.status,
		ProjectStatus::Rejected | ProjectStatus::Withheld
	) {
		vec![ProjectNag::new(
			ProjectNagKind::ModeratorFeedback,
			ProjectNagSeverity::Warning,
		)]
	} else {
		Vec::new()
	}
}
