use super::input;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::OwnedRwLockReadGuard;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredFileMetadata {
    pub sha512: String,
    pub sha1: String,
    pub size: i64,
    pub relative_path: String,
    pub status: StoredFileStatus,
    pub modified_at_ns: i64,
    pub last_used_at: i64,
    pub sources: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct InstanceFileStorage {
    pub file_id: String,
    pub blob_sha512: String,
    pub storage_kind: FileStorageKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FileStorageKind {
    Reflink,
    Hardlink,
    Copy,
}

impl FileStorageKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Reflink => "reflink",
            Self::Hardlink => "hardlink",
            Self::Copy => "copy",
        }
    }

    pub(super) fn from_db(value: &str) -> crate::Result<Self> {
        match value {
            "reflink" => Ok(Self::Reflink),
            "hardlink" => Ok(Self::Hardlink),
            "copy" => Ok(Self::Copy),
            _ => Err(input("Invalid content materialization kind")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StoredFileStatus {
    Ready,
    Quarantined,
    Deleting,
}

impl StoredFileStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Quarantined => "quarantined",
            Self::Deleting => "deleting",
        }
    }

    pub(super) fn from_db(value: &str) -> crate::Result<Self> {
        match value {
            "ready" => Ok(Self::Ready),
            "quarantined" => Ok(Self::Quarantined),
            "deleting" => Ok(Self::Deleting),
            _ => Err(input("Invalid content object status")),
        }
    }
}

/// A healthy stored file. Keeping this handle alive prevents cache cleanup.
#[derive(Clone, Debug)]
pub(crate) struct StoredFileHandle {
    pub metadata: StoredFileMetadata,
    pub path: PathBuf,
    pub(super) _guard: Arc<OwnedRwLockReadGuard<()>>,
}

/// A stored-file record for recovery. Its bytes may be missing or damaged.
#[derive(Clone, Debug)]
pub(crate) struct StoredFileRecord {
    pub metadata: StoredFileMetadata,
    pub path: PathBuf,
    pub(super) _guard: Arc<OwnedRwLockReadGuard<()>>,
}

#[derive(Clone, Debug)]
pub(crate) enum FileContent {
    Unmanaged,
    Stored {
        storage: InstanceFileStorage,
        stored_file: StoredFileHandle,
    },
    Damaged(InstanceFileStorage),
}

#[derive(Clone, Debug)]
pub(crate) enum ReadableContent {
    Local(PathBuf),
    Stored(StoredFileHandle),
}

impl ReadableContent {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Local(path) => path,
            Self::Stored(stored_file) => &stored_file.path,
        }
    }
}

pub(crate) struct GetFileResult {
    pub stored_file: StoredFileHandle,
    pub reused: bool,
}

#[derive(Serialize)]
pub struct StoreUsage {
    pub unique_bytes: u64,
    pub shared_bytes: u64,
    pub unused_cache_bytes: u64,
    pub estimated_saved_bytes: u64,
    pub private_copy_bytes: u64,
    pub object_count: usize,
    pub damaged_objects: usize,
    pub cache_limit_bytes: u64,
}

#[derive(Serialize)]
pub struct StoreIssue {
    pub sha512: String,
    pub instance_ids: Vec<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct StoreVerification {
    pub checked: usize,
    pub repaired: usize,
    pub issues: Vec<StoreIssue>,
}

/// Independent files may share copy-on-write blocks, but never writes or permissions.
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FileHashes {
    pub sha512: String,
    pub sha1: String,
    pub size: u64,
}
