use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
    project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
    let _ = project;

    vec![ProjectNag::new(
        ProjectNagKind::CheckDisclosures,
        ProjectNagSeverity::Suggestion,
    )]
}
