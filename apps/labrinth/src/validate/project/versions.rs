use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
    project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
    if project.versions.is_empty()
        && project.components.minecraft_server.is_none()
    {
        vec![ProjectNag::new(
            ProjectNagKind::UploadVersion,
            ProjectNagSeverity::Required,
        )]
    } else {
        Vec::new()
    }
}
