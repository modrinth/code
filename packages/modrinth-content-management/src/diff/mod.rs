//! Compares prepared content sets and composes caller-specific changes.

use std::collections::BTreeSet;

pub use configuration::{
    ConfigurationDiff, ContentSetConfiguration, LoaderReference,
    diff_configuration,
};
pub use model::{
    Change, CommonExternalFilePolicy, ContentSetDiff, ContentSetDiffEntry,
    ContentSetDiffKind, ContentSetDiffOptions, ContentSetSnapshot,
    ExternalFileKey,
};

/// Computes changes from `before` to `after`, ordered by project and file identity.
///
/// Callers select the publishing or installation scope before comparison.
/// External files are compared by identity, using the supplied policy for common files.
pub fn diff_content_sets(
    before: &ContentSetSnapshot,
    after: &ContentSetSnapshot,
    options: &ContentSetDiffOptions,
) -> ContentSetDiff {
    let mut content = Vec::new();
    let project_ids = before
        .projects
        .keys()
        .chain(after.projects.keys())
        .collect::<BTreeSet<_>>();
    for project_id in project_ids {
        if let Some(change) = Change::between(
            before.projects.get(project_id),
            after.projects.get(project_id),
        ) {
            content.push(ContentSetDiffEntry::Project {
                project_id: project_id.clone(),
                change,
            });
        }
    }
    for file in before.external_files.union(&after.external_files) {
        let kind = match (
            before.external_files.contains(file),
            after.external_files.contains(file),
        ) {
            (false, true) => ContentSetDiffKind::Added,
            (true, false) => ContentSetDiffKind::Removed,
            (true, true)
                if options.common_external_files
                    == CommonExternalFilePolicy::AssumeUpdated =>
            {
                ContentSetDiffKind::Updated
            }
            _ => continue,
        };
        content.push(ContentSetDiffEntry::ExternalFile {
            file: file.clone(),
            kind,
        });
    }
    ContentSetDiff {
        content,
        additional: Vec::new(),
    }
}

mod configuration;
mod model;
