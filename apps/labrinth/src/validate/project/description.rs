use serde_json::json;

use super::text::{
    ProfanityKind, contains_spam, description_ends_with_header,
    extract_description_blocks, extract_description_text,
    find_banned_description_link, has_adjacent_same_level_headers,
    has_image_without_alt_text, has_sufficient_english_blocks,
    js_string_length, long_header_count, non_standard_text_ratio,
    normalize_project_field_text, profanity_matches, project_requires_english,
};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

use crate::models::projects::Project;

const MIN_DESCRIPTION_CHARS: usize = 125;
const MAX_PROFANITY_COUNT: usize = 2;
const NON_STANDARD_TEXT_FAILURE_THRESHOLD: f64 = 0.05;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let description = project.description.as_str();
    let normalized_description = normalize_project_field_text(description);
    let text = extract_description_text(description);
    let normalized_text = extract_description_text(&normalized_description);
    let blocks = extract_description_blocks(description);
    let profanity = profanity_matches(description);

    if let Some(matched) = profanity
        .iter()
        .find(|matched| matched.kind == ProfanityKind::Slur)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectDescriptionSlur,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if let Some(matched) = profanity
        .iter()
        .filter(|matched| matched.kind == ProfanityKind::Profanity)
        .nth(MAX_PROFANITY_COUNT)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectDescriptionProfanity,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if non_standard_text_ratio(description)
        >= NON_STANDARD_TEXT_FAILURE_THRESHOLD
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionNonStandardText,
            ProjectNagSeverity::Required,
        ));
    }
    if project_requires_english(project)
        && js_string_length(&text) >= MIN_DESCRIPTION_CHARS
        && !contains_spam(&text)
        && !has_sufficient_english_blocks(&blocks)
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionNonEnglish,
            ProjectNagSeverity::Required,
        ));
    }
    if normalized_description.is_empty() {
        nags.push(ProjectNag::new(
            ProjectNagKind::AddDescription,
            ProjectNagSeverity::Required,
        ));
    } else {
        let description_length = js_string_length(&normalized_text);
        if description_length < MIN_DESCRIPTION_CHARS {
            nags.push(
                ProjectNag::new(
                    ProjectNagKind::DescriptionTooShort,
                    ProjectNagSeverity::Required,
                )
                .with_details(json!({
                    "length": description_length,
                    "min_chars": MIN_DESCRIPTION_CHARS,
                })),
            );
        }
    }
    if contains_spam(&text) {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionSpam,
            ProjectNagSeverity::Required,
        ));
    }
    if let Some(url) = find_banned_description_link(description) {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectDescriptionBannedLink,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "full_url": url })),
        );
    }
    let long_headers = long_header_count(description);
    if long_headers > 0 {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::LongHeaders,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "count": long_headers })),
        );
    }
    if description_ends_with_header(description) {
        nags.push(ProjectNag::new(
            ProjectNagKind::DescriptionEndsWithHeader,
            ProjectNagSeverity::Required,
        ));
    }
    if has_adjacent_same_level_headers(description) {
        nags.push(ProjectNag::new(
            ProjectNagKind::AdjacentHeaders,
            ProjectNagSeverity::Required,
        ));
    }
    if has_image_without_alt_text(description) {
        nags.push(ProjectNag::new(
            ProjectNagKind::MissingAltText,
            ProjectNagSeverity::Warning,
        ));
    }

    nags
}
