use std::sync::LazyLock;

use regex::Regex;
use serde_json::json;

use super::text::{
    ProfanityKind, has_non_standard_text, normalize_project_field_text,
    profanity_matches,
};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::projects::Project;

static VERSION_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d+(?:\.\d+)+").unwrap());

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let name = &project.name;
    let profanity = profanity_matches(name);

    if let Some(matched) = profanity
        .iter()
        .find(|matched| matched.kind == ProfanityKind::Slur)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectNameSlur,
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
                ProjectNagKind::ProjectNameProfanity,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if has_non_standard_text(name) {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectNameNonStandardText,
            ProjectNagSeverity::Required,
        ));
    }

    let normalized = normalize_project_field_text(name).to_lowercase();
    let is_port_or_fork =
        normalized.contains("port") || normalized.contains("fork");
    if VERSION_NUMBER.is_match(&normalized) && !is_port_or_fork {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectNameVersion,
            ProjectNagSeverity::Required,
        ));
    }
    if normalized.contains("minecraft")
        && normalized.split_whitespace().count() <= 3
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::MinecraftTitleClause,
            ProjectNagSeverity::Required,
        ));
    }

    nags
}
