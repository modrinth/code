use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

use serde::{Deserialize, Serialize};

use crate::shared::{ContentType, Error};

/// A caller-prepared set of content in the scope being compared.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentSetSnapshot {
	/// Modrinth project IDs mapped to version IDs.
	pub projects: BTreeMap<String, String>,
	pub external_files: BTreeSet<ExternalFileKey>,
}

impl ContentSetSnapshot {
	/// Adds a project, rejecting conflicting versions without replacing it.
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

/// Identifies a file by its content type and path within that type's directory.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
	/// Transforms the values while preserving the classification of the change.
	pub fn map<U>(self, mut map: impl FnMut(T) -> U) -> Change<U> {
		match self {
			Self::Added { after } => Change::Added { after: map(after) },
			Self::Removed { before } => Change::Removed { before: map(before) },
			Self::Updated { before, after } => Change::Updated {
				before: map(before),
				after: map(after),
			},
		}
	}

	/// Returns the classification of this change.
	pub fn kind(&self) -> ContentSetDiffKind {
		match self {
			Self::Added { .. } => ContentSetDiffKind::Added,
			Self::Removed { .. } => ContentSetDiffKind::Removed,
			Self::Updated { .. } => ContentSetDiffKind::Updated,
		}
	}

	/// Returns the value before the change, if it existed.
	pub fn before(&self) -> Option<&T> {
		match self {
			Self::Removed { before } | Self::Updated { before, .. } => Some(before),
			Self::Added { .. } => None,
		}
	}

	/// Returns the value after the change, if it exists.
	pub fn after(&self) -> Option<&T> {
		match self {
			Self::Added { after } | Self::Updated { after, .. } => Some(after),
			Self::Removed { .. } => None,
		}
	}
}

impl<T: Clone + PartialEq> Change<T> {
	/// Compares optional values, returning no entry when they are equal.
	pub fn between(before: Option<&T>, after: Option<&T>) -> Option<Self> {
		match (before, after) {
			(None, Some(after)) => Some(Self::Added { after: after.clone() }),
			(Some(before), None) => Some(Self::Removed { before: before.clone() }),
			(Some(before), Some(after)) if before != after => Some(Self::Updated {
				before: before.clone(),
				after: after.clone(),
			}),
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

/// Policy for files present on both sides when their bytes are not compared.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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

/// Content changes and typed entries contributed by the caller.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentSetDiff<E = Infallible> {
	pub content: Vec<ContentSetDiffEntry>,
	pub additional: Vec<E>,
}

impl<E> ContentSetDiff<E> {
	/// Includes both content changes and caller-provided changes.
	pub fn has_changes(&self) -> bool {
		!self.content.is_empty() || !self.additional.is_empty()
	}
}

impl ContentSetDiff {
	/// Attaches typed changes without coupling the comparator to the caller.
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

