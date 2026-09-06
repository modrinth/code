//! Finds added, removed, and updated content, with room for extra changes
//! such as linking a modpack.

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

/// Lists what would change when replacing `before` with `after`.
///
/// Projects are matched by project ID and updated when their version ID changes.
/// Files are matched by content type and path. For matching files, `options`
/// decides whether to show an update; file contents are not checked.
///
/// Results are sorted by project ID, followed by files sorted by type and path.
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
