use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::projects::Version;

pub(super) fn validate(versions: &[Version]) -> Vec<ProjectNag> {
    if versions
        .iter()
        .any(|version| !version.files_missing_attribution.is_empty())
    {
        vec![ProjectNag::new(
            ProjectNagKind::ReviewPermissions,
            ProjectNagSeverity::Required,
        )]
    } else {
        Vec::new()
    }
}
