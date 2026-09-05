use serde_json::json;

use super::text::{
    ProfanityKind, contains_spam, find_link_or_ip, has_non_standard_text,
    has_summary_formatting, is_likely_english_summary, js_string_length,
    normalize_project_field_text, profanity_matches, project_requires_english,
};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

use crate::models::projects::Project;

const MIN_SUMMARY_CHARS: usize = 25;
const MAX_SUMMARY_NAME_SIMILARITY: f64 = 0.8;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let summary = project.summary.as_str();
    let normalized_summary = normalize_project_field_text(summary);
    let summary_link = find_link_or_ip(summary);
    let contains_link = summary_link.is_some();
    let has_spam = contains_spam(&normalized_summary);
    let profanity = profanity_matches(summary);

    if let Some(matched) = profanity
        .iter()
        .find(|matched| matched.kind == ProfanityKind::Slur)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectSummarySlur,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if let Some(matched) = profanity
        .iter()
        .find(|matched| matched.kind == ProfanityKind::Profanity)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectSummaryProfanity,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if has_non_standard_text(summary) {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectSummaryNonStandardText,
            ProjectNagSeverity::Required,
        ));
    }
    if requires_language_nag(
        project,
        &normalized_summary,
        contains_link,
        has_spam,
    ) {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectSummaryNonEnglish,
            ProjectNagSeverity::Required,
        ));
    }
    if !summary.is_empty()
        && !contains_link
        && !project.name.is_empty()
        && summary_name_similarity(summary, &project.name)
            >= MAX_SUMMARY_NAME_SIMILARITY
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectSummaryMatchesTitle,
            ProjectNagSeverity::Required,
        ));
    }
    let summary_length = js_string_length(&normalized_summary);
    if !summary.is_empty()
        && !contains_link
        && summary_length < MIN_SUMMARY_CHARS
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::SummaryTooShort,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({
                "length": summary_length,
                "min_chars": MIN_SUMMARY_CHARS,
            })),
        );
    }
    if has_spam {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectSummarySpam,
            ProjectNagSeverity::Required,
        ));
    }
    if !summary.is_empty() && has_summary_formatting(summary) {
        nags.push(ProjectNag::new(
            ProjectNagKind::SummarySpecialFormatting,
            ProjectNagSeverity::Required,
        ));
    }
    if let Some(value) = summary_link {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectSummaryLinks,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": value })),
        );
    }

    nags
}

pub(super) fn is_non_english(project: &Project) -> bool {
    let normalized_summary = normalize_project_field_text(&project.summary);
    is_non_english_text(project, &normalized_summary)
}

fn is_non_english_text(project: &Project, normalized_summary: &str) -> bool {
    project_requires_english(project)
        && js_string_length(normalized_summary) >= MIN_SUMMARY_CHARS
        && !is_likely_english_summary(normalized_summary)
}

fn requires_language_nag(
    project: &Project,
    normalized_summary: &str,
    contains_link: bool,
    has_spam: bool,
) -> bool {
    is_non_english_text(project, normalized_summary)
        && !contains_link
        && !has_spam
}

fn summary_name_similarity(summary: &str, name: &str) -> f64 {
    let summary = normalized_for_similarity(summary);
    let name = normalized_for_similarity(name);
    let longest_length = summary.len().max(name.len());
    if longest_length == 0 {
        return 0.0;
    }

    1.0 - levenshtein_distance(&summary, &name) as f64 / longest_length as f64
}

fn normalized_for_similarity(text: &str) -> Vec<char> {
    normalize_project_field_text(text)
        .to_lowercase()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn levenshtein_distance(left: &[char], right: &[char]) -> usize {
    if left.len() > right.len() {
        return levenshtein_distance(right, left);
    }

    let mut previous_row = (0..=left.len()).collect::<Vec<_>>();
    for (right_index, right_character) in right.iter().enumerate() {
        let mut current_row = Vec::with_capacity(left.len() + 1);
        current_row.push(right_index + 1);
        for (left_index, left_character) in left.iter().enumerate() {
            current_row.push(
                (current_row[left_index] + 1)
                    .min(previous_row[left_index + 1] + 1)
                    .min(
                        previous_row[left_index]
                            + usize::from(left_character != right_character),
                    ),
            );
        }
        previous_row = current_row;
    }
    previous_row[left.len()]
}
