use crate::models::projects::{Project, Version};

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

pub(super) fn validate(
    project: &Project,
    _versions: &[Version],
) -> Vec<ProjectNag> {
    let mut nags = Vec::new();

    if project.versions.is_empty()
        && project.components.minecraft_server.is_none()
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::UploadVersion,
            ProjectNagSeverity::Required,
        ));
    }

    let requires_environment = project
        .project_types
        .iter()
        .any(|project_type| matches!(project_type.as_str(), "mod" | "modpack"));
    let has_valid_environment =
        project
            .fields
            .get("environment")
            .is_some_and(|environments| {
                !environments.is_empty()
                    && environments.iter().all(|environment| {
                        environment.as_str().is_some_and(|environment| {
                            !environment.trim().is_empty()
                                && environment != "unknown"
                        })
                    })
            });
    if requires_environment && !has_valid_environment {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectEnvironment,
            ProjectNagSeverity::Required,
        ));
    }

    nags
}
