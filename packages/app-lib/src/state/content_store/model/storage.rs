use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct InstanceFileStorage {
    pub file_id: String,
    pub blob_sha512: String,
    pub storage_kind: FileStorageKind,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub(crate) enum FileStorageKind {
    Reflink,
    Hardlink,
    Copy,
}

#[derive(Clone, Copy)]
pub(crate) enum FileStoragePolicy {
    Shared,
    Independent,
}

impl FileStorageKind {
    pub(crate) fn restore_policy(self) -> FileStoragePolicy {
        match self {
            Self::Copy | Self::Reflink => FileStoragePolicy::Independent,
            Self::Hardlink => FileStoragePolicy::Shared,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub(crate) enum StoredFileStatus {
    Ready,
    Quarantined,
    Deleting,
}
