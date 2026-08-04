use crate::state::Version;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContentSetDiffKind {
    Added,
    Removed,
    Updated,
}

#[derive(Clone, Debug)]
pub(crate) enum ContentSetDiffEntry {
    Project {
        kind: ContentSetDiffKind,
        project_id: String,
        current_version_name: Option<String>,
        new_version_name: Option<String>,
        disabled: bool,
    },
    ExternalFile {
        kind: ContentSetDiffKind,
        file_name: String,
        disabled: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContentSetDiffOptions {
    pub removed_disabled_project_ids: HashSet<String>,
    pub removed_disabled_external_files: HashSet<String>,
    pub common_external_files_are_updated: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContentSetSnapshot {
    pub versions: Vec<ContentSetSnapshotVersion>,
    pub external_files: HashSet<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ContentSetSnapshotVersion {
    pub project_id: String,
    pub version_id: String,
    pub version_name: String,
}

impl From<Version> for ContentSetSnapshotVersion {
    fn from(version: Version) -> Self {
        Self {
            project_id: version.project_id,
            version_id: version.id,
            version_name: version.version_number,
        }
    }
}

pub(crate) fn diff_content_sets(
    current: &ContentSetSnapshot,
    latest: &ContentSetSnapshot,
    options: &ContentSetDiffOptions,
) -> Vec<ContentSetDiffEntry> {
    let current_versions = versions_by_project(current);
    let latest_versions = versions_by_project(latest);
    let project_ids = current_versions
        .keys()
        .chain(latest_versions.keys())
        .copied()
        .collect::<HashSet<_>>();

    let mut diffs = Vec::new();
    for project_id in project_ids {
        let current = current_versions.get(project_id);
        let latest = latest_versions.get(project_id);

        match (current, latest) {
            (None, Some(latest)) => {
                diffs.push(ContentSetDiffEntry::Project {
                    kind: ContentSetDiffKind::Added,
                    project_id: project_id.to_string(),
                    current_version_name: None,
                    new_version_name: Some(latest.version_name.clone()),
                    disabled: false,
                });
            }
            (Some(current), None) => {
                diffs.push(ContentSetDiffEntry::Project {
                    kind: ContentSetDiffKind::Removed,
                    project_id: project_id.to_string(),
                    current_version_name: Some(current.version_name.clone()),
                    new_version_name: None,
                    disabled: options
                        .removed_disabled_project_ids
                        .contains(project_id),
                });
            }
            (Some(current), Some(latest))
                if current.version_id != latest.version_id =>
            {
                diffs.push(ContentSetDiffEntry::Project {
                    kind: ContentSetDiffKind::Updated,
                    project_id: project_id.to_string(),
                    current_version_name: Some(current.version_name.clone()),
                    new_version_name: Some(latest.version_name.clone()),
                    disabled: false,
                });
            }
            _ => {}
        }
    }

    for file_name in latest.external_files.difference(&current.external_files) {
        diffs.push(ContentSetDiffEntry::ExternalFile {
            kind: ContentSetDiffKind::Added,
            file_name: file_name.clone(),
            disabled: false,
        });
    }

    if options.common_external_files_are_updated {
        for file_name in
            latest.external_files.intersection(&current.external_files)
        {
            diffs.push(ContentSetDiffEntry::ExternalFile {
                kind: ContentSetDiffKind::Updated,
                file_name: file_name.clone(),
                disabled: false,
            });
        }
    }

    for file_name in current.external_files.difference(&latest.external_files) {
        diffs.push(ContentSetDiffEntry::ExternalFile {
            kind: ContentSetDiffKind::Removed,
            file_name: file_name.clone(),
            disabled: options
                .removed_disabled_external_files
                .contains(file_name),
        });
    }

    diffs
}

fn versions_by_project(
    snapshot: &ContentSetSnapshot,
) -> HashMap<&str, &ContentSetSnapshotVersion> {
    snapshot
        .versions
        .iter()
        .map(|version| (version.project_id.as_str(), version))
        .collect()
}
