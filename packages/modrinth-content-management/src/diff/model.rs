use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

use serde::{Deserialize, Serialize};

use crate::shared::{ContentType, Error};

/// The projects and files to compare.
///
/// Include only content relevant to the operation. For example, when receiving
/// a shared-instance update, leave out mods the player added themselves.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentSetSnapshot {
    /// Each project has one version. Keys are project IDs; values are version IDs.
    pub projects: BTreeMap<String, String>,
    pub external_files: BTreeSet<ExternalFileKey>,
}

impl ContentSetSnapshot {
    /// Adds a project's version. Adding the same version again does nothing.
    /// Adding a different version for that project returns an error and keeps the original.
    pub fn insert_project(
        &mut self,
        project_id: String,
        version_id: String,
    ) -> Result<(), Error> {
        if let Some(existing) = self.projects.get(&project_id) {
            if existing != &version_id {
                return Err(Error::ConflictingProjectVersions {
                    project_id,
                    before: existing.clone(),
                    after: version_id,
                });
            }
        } else {
            self.projects.insert(project_id, version_id);
        }
        Ok(())
    }
}

/// A file's type and path, so a mod and a plugin with the same filename are
/// treated as separate files. For example, `mods/example.jar` uses type `Mod`
/// and path `example.jar`.
#[derive(
    Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct ExternalFileKey {
    pub content_type: ContentType,
    pub path: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSetDiffKind {
    Added,
    Removed,
    Updated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Change<T> {
    Added { after: T },
    Removed { before: T },
    Updated { before: T, after: T },
}

impl<T> Change<T> {
    /// Converts the before/after values, for example from version IDs to display names.
    pub fn map<U>(self, mut map: impl FnMut(T) -> U) -> Change<U> {
        match self {
            Self::Added { after } => Change::Added { after: map(after) },
            Self::Removed { before } => Change::Removed {
                before: map(before),
            },
            Self::Updated { before, after } => Change::Updated {
                before: map(before),
                after: map(after),
            },
        }
    }

    pub fn kind(&self) -> ContentSetDiffKind {
        match self {
            Self::Added { .. } => ContentSetDiffKind::Added,
            Self::Removed { .. } => ContentSetDiffKind::Removed,
            Self::Updated { .. } => ContentSetDiffKind::Updated,
        }
    }

    /// Returns `None` for an added item because it had no previous value.
    pub fn before(&self) -> Option<&T> {
        match self {
            Self::Removed { before } | Self::Updated { before, .. } => {
                Some(before)
            }
            Self::Added { .. } => None,
        }
    }

    /// Returns `None` for a removed item because it no longer has a value.
    pub fn after(&self) -> Option<&T> {
        match self {
            Self::Added { after } | Self::Updated { after, .. } => Some(after),
            Self::Removed { .. } => None,
        }
    }
}

impl<T: Clone + PartialEq> Change<T> {
    /// Compares the old and new values. `None` means the item is absent.
    /// Returns no change if the values are equal or both absent.
    pub fn between(before: Option<&T>, after: Option<&T>) -> Option<Self> {
        match (before, after) {
            (None, Some(after)) => Some(Self::Added {
                after: after.clone(),
            }),
            (Some(before), None) => Some(Self::Removed {
                before: before.clone(),
            }),
            (Some(before), Some(after)) if before != after => {
                Some(Self::Updated {
                    before: before.clone(),
                    after: after.clone(),
                })
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentSetDiffEntry {
    Project {
        project_id: String,
        change: Change<String>,
    },
    ExternalFile {
        file: ExternalFileKey,
        kind: ContentSetDiffKind,
    },
}

/// Decides whether to show an update when a file has the same type and path
/// in both sets. The comparison does not read the files or compare their contents.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CommonExternalFilePolicy {
    #[default]
    AssumeUnchanged,
    AssumeUpdated,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentSetDiffOptions {
    pub common_external_files: CommonExternalFilePolicy,
}

/// Changes to projects and files, plus any extra changes the app or server adds,
/// such as a linked modpack or selected config files.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentSetDiff<E = Infallible> {
    pub content: Vec<ContentSetDiffEntry>,
    pub additional: Vec<E>,
}

impl<E> ContentSetDiff<E> {
    /// Returns true for extra changes too, even when no projects or files changed.
    pub fn has_changes(&self) -> bool {
        !self.content.is_empty() || !self.additional.is_empty()
    }
}

impl ContentSetDiff {
    /// Adds other changes to the result, such as a modpack change or selected config files.
    /// These count towards `has_changes()` too.
    pub fn with_additional<E>(
        self,
        entries: impl IntoIterator<Item = E>,
    ) -> ContentSetDiff<E> {
        ContentSetDiff {
            content: self.content,
            additional: entries.into_iter().collect(),
        }
    }
}
