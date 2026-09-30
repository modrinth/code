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
pub(crate) use commands::{
    PendingFileChange, migrate, migrate_instance_copies,
};
pub(crate) use domain::{
    content_file_path, file_path_on_disk, is_managed_content_path, normalize,
    object_relative_path, relative_link, validate_digest,
    validate_instance_path, validate_relative,
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
