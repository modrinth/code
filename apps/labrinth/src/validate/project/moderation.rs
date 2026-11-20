use crate::models::projects::ProjectReviewOutcome;

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
    project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
    if matches!(
        project.review_outcome,
        ProjectReviewOutcome::Rejected | ProjectReviewOutcome::Withheld
    ) {
        vec![ProjectNag::new(
            ProjectNagKind::ModeratorFeedback,
            ProjectNagSeverity::Warning,
        )]
    } else {
        Vec::new()
    }
}
