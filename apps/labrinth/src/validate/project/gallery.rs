use serde_json::json;

use super::text::{ProfanityKind, has_non_standard_text, profanity_matches};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_GALLERY_DESCRIPTION_PROFANITY_COUNT: usize = 0;

fn validate_text(
    text: Option<&str>,
    gallery_index: usize,
    field: &'static str,
    max_profanity_count: usize,
) -> Vec<ProjectNag> {
    let text = text.unwrap_or_default();
    let mut nags = Vec::new();
    let profanity = profanity_matches(text);

    if let Some(matched) = profanity
        .iter()
        .find(|matched| matched.kind == ProfanityKind::Slur)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::GalleryTextSlur,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({
                "gallery_index": gallery_index,
                "field": field,
                "value": matched.raw_text,
            })),
        );
    }
    if let Some(matched) = profanity
        .iter()
        .filter(|matched| matched.kind == ProfanityKind::Profanity)
        .nth(max_profanity_count)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::GalleryTextProfanity,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({
                "gallery_index": gallery_index,
                "field": field,
                "value": matched.raw_text,
            })),
        );
    }
    if has_non_standard_text(text) {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::GalleryTextNonStandard,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({
                "gallery_index": gallery_index,
                "field": field,
            })),
        );
    }

    nags
}

pub(super) fn validate(
    project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
    let mut nags = Vec::new();
    let gallery_is_empty = project.gallery.is_empty();
    let is_shader = project.project_types.iter().any(|ty| ty == "shader");
    let is_resource_pack =
        project.project_types.iter().any(|ty| ty == "resourcepack");
    let has_gallery_exemption = project
        .categories
        .iter()
        .chain(&project.additional_categories)
        .any(|category| category == "audio" || category == "locale");

    if is_shader && project.gallery.len() < 3 {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::UploadGalleryImage,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "project_type": "shader" })),
        );
    } else if is_resource_pack && gallery_is_empty && !has_gallery_exemption {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::UploadGalleryImage,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "project_type": "resourcepack" })),
        );
    }

    let is_minecraft_server = project.components.minecraft_server.is_some();
    if !is_minecraft_server && !project.gallery.iter().any(|item| item.featured)
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::FeatureGalleryImage,
            ProjectNagSeverity::Suggestion,
        ));
    }

    for (index, item) in project.gallery.iter().enumerate() {
        nags.extend(validate_text(item.name.as_deref(), index, "name", 0));
        nags.extend(validate_text(
            item.description.as_deref(),
            index,
            "description",
            MAX_GALLERY_DESCRIPTION_PROFANITY_COUNT,
        ));
    }

    nags
}
