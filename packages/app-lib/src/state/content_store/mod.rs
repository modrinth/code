//! Stores immutable content once and installs it into instances using filesystem sharing.
//!
//! Downloads and imports are verified before publication. Handles prevent cache cleanup
//! while content is in use; retained references protect installed files and rollback data.
//! Instance changes are journaled before filesystem mutations and committed with their
//! database records. Recovery preserves externally changed files.

mod acquire_content;
mod cache;
pub(crate) mod catalog;
mod file_changes;
pub(crate) mod file_io;
mod instance_files;
pub(crate) mod migration;
mod read_content;
mod recovery;
mod store;
mod types;
mod verify_content;

pub(crate) use file_changes::{FileChangeRequest, PendingFileChange};
use file_io::hash_file_with_progress;
pub(crate) use file_io::{
    hash_file, is_managed_content_path, normalize, relative_link,
    sync_directory, validate_digest, validate_relative, writable_copy,
};
pub(crate) use instance_files::{
    InstanceFileStatus, content_file_path, file_path_on_disk,
};
pub use store::ContentStore;
use types::StoreIssue;
pub(crate) use types::{
    FileContent, FileHashes, FileStorageKind, FileStoragePolicy, GetFileResult,
    InstanceFileStorage, ReadableContent, StoredFileHandle, StoredFileMetadata,
    StoredFileRecord, StoredFileStatus,
};
pub use types::{StoreUsage, StoreVerification};

pub(crate) fn input(message: impl Into<String>) -> crate::Error {
    crate::ErrorKind::InputError(message.into()).into()
}
