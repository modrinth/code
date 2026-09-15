use crate::state::content_store::FileStorageKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(in crate::state::content_store) struct FileState {
    pub(in crate::state::content_store) relative_path: String,
    pub(in crate::state::content_store) sha512: String,
    pub(in crate::state::content_store) present: bool,
    pub(in crate::state::content_store) storage_kind: Option<FileStorageKind>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(in crate::state::content_store) struct FileChangeJournal {
    pub(in crate::state::content_store) id: String,
    pub(in crate::state::content_store) instance_id: String,
    pub(in crate::state::content_store) instance_path: String,
    pub(in crate::state::content_store) before: Option<FileState>,
    pub(in crate::state::content_store) after: Option<FileState>,
}

use crate::state::content_store::StoredFileHandle;

pub(crate) struct FileChangeRequest<'a> {
    pub relative_path: &'a str,
    pub replacement: Option<&'a StoredFileHandle>,
    pub enabled: bool,
    pub legacy_path: Option<&'a str>,
    pub previous_content: Option<&'a StoredFileHandle>,
}
