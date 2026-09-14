//! Reuses downloaded and imported mods and packs across instances.
//!
//! Files are identified by their SHA-512 hash, so installing the same content again
//! can reuse the stored file without another download. Each instance still has its
//! own file path. We prefer reflinks, then hardlinks, then an ordinary copy when
//! the filesystem cannot share the data.
//!
//! Reflinks let an instance change its file without changing the stored original.
//! Hardlinks share the original file, so app content updates replace the instance
//! file rather than writing into it. The Files tab prevents edits to managed content.
//!
//! Cleanup keeps files needed by installed content, unfinished changes, and install
//! backups even when the unused-cache limit is exceeded. If a change is interrupted,
//! recovery restores the previous files. It preserves files changed outside the app
//! and reports conflicts instead of overwriting them.

mod adapters;
mod commands;
mod domain;
mod model;
mod store;

use crate::util::content_hash::hash_file_with_progress;
pub(crate) use crate::util::content_hash::{FileHashes, hash_file};
pub(crate) use adapters::filesystem::{
    remove_instance_file, sync_directory, try_shared_file,
    validate_parent_directories, writable_copy,
};
pub(crate) use adapters::sqlite::{
    file_storage, find_file, instance_storage, retained_owners,
    set_file_storage, set_setting, setting,
};
pub(crate) use commands::{PendingFileChange, migrate};
pub(crate) use domain::{
    content_file_path, file_path_on_disk, is_managed_content_path, normalize,
    relative_link, validate_digest, validate_relative,
};
use model::StoreIssue;
pub(crate) use model::{
    FileChangeRequest, FileStorageKind, FileStoragePolicy, InstanceFileStatus,
    InstanceFileStorage, StoredFileStatus,
};
pub use model::{StoreUsage, StoreVerification};
pub use store::ContentStore;
pub(crate) use store::{
    FileContent, GetFileResult, ReadableContent, StoredFileHandle,
    StoredFileRecord,
};

pub(crate) fn input(message: impl Into<String>) -> crate::Error {
    crate::ErrorKind::InputError(message.into()).into()
}

#[cfg(windows)]
pub(crate) use adapters::filesystem::link_unavailable;

pub(crate) use adapters::sqlite::StoredFileMetadata;
