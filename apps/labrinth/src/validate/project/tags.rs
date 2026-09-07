use crate::{
    database::models::categories::Category,
    models::{projects::Project, v2::projects::LegacyProject},
};

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_TAG_COUNT: usize = 8;
const MAX_TAG_COUNT_SERVER: usize = 18;
const RESOLUTION_TAGS: [&str; 8] =
    ["8x-", "16x", "32x", "48x", "64x", "128x", "256x", "512x+"];

pub(super) fn validate(
    project: &Project,
    available_categories: Option<&[Category]>,
) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let tag_count =
        project.categories.len() + project.additional_categories.len();
    let is_minecraft_server = project.components.minecraft_server.is_some();
    let (project_type, actual_project_type) =
        LegacyProject::get_project_type(&project.project_types);

    if !project.versions.is_empty() && project.categories.is_empty() {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectTags,
            ProjectNagSeverity::Suggestion,
        ));
    }

    if !is_minecraft_server && tag_count > MAX_TAG_COUNT {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::TooManyTags,
                ProjectNagSeverity::Warning,
            )
            .with_details(serde_json::json!({
                "tag_count": tag_count,
                "max_tag_count": MAX_TAG_COUNT,
            })),
        );
    }

    if is_minecraft_server && tag_count > MAX_TAG_COUNT_SERVER {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::TooManyTagsServer,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({
                "tag_count": tag_count,
                "max_tag_count": MAX_TAG_COUNT_SERVER,
            })),
        );
    }

    let mut resolution_tags = project
        .categories
        .iter()
        .chain(&project.additional_categories)
        .filter(|tag| RESOLUTION_TAGS.contains(&tag.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    resolution_tags.sort_by_key(|tag| {
        RESOLUTION_TAGS
            .iter()
            .position(|resolution| resolution == tag)
            .unwrap_or(RESOLUTION_TAGS.len())
    });
    if project_type == "resourcepack" && resolution_tags.len() > 1 {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::MultipleResolutionTags,
                ProjectNagSeverity::Warning,
            )
            .with_details(serde_json::json!({
                "count": resolution_tags.len(),
                "tags": resolution_tags.join("|"),
            })),
        );
    }

    if let Some(available_categories) = available_categories {
        let total_available_tags = available_categories
            .iter()
            .filter(|category| category.project_type == actual_project_type)
            .count();
        if tag_count == total_available_tags && project_type != "project" {
            nags.push(
                ProjectNag::new(
                    ProjectNagKind::AllTagsSelected,
                    ProjectNagSeverity::Required,
                )
                .with_details(serde_json::json!({
                    "total_available_tags": total_available_tags,
                })),
            );
        }
    }

    nags
}
