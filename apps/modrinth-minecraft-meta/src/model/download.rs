use jiff::Timestamp;
use toasty::{Embed, Model};
use uuid::Uuid;

use crate::util::{ErrorVec, Sha1, Sha256};

#[derive(Debug, Clone, Model)]
#[table = "blob_hashes"]
pub struct BlobHash {
    #[key]
    pub sha256: Sha256,
    #[index]
    pub sha1: Sha1,
}

/// ID for a [`DownloadRun`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct DownloadRunId(pub Uuid);

/// Single run of a task to download sources from our upstreams.
///
/// During a [`DownloadRun`], we download a bunch of files and save them into
/// our database and file store. Any blobs saved within the same run can be
/// logically grouped together.
#[derive(Debug, Clone, Model)]
pub struct DownloadRun {
    #[key]
    #[auto]
    pub id: DownloadRunId,
    /// When this run started.
    pub started_at: Timestamp,
    /// When this run finished.
    ///
    /// If this run is still ongoing, this may be [`None`].
    pub completed_at: Option<Timestamp>,
    /// Errors accumulated during the run.
    ///
    /// If this run is still ongoing, this may be [`None`].
    #[column(type = text)]
    pub errors: Option<toasty::Json<ErrorVec>>,
}

#[derive(Debug, Clone, Model)]
#[key(download_run_id, url)]
pub struct BlobDownload {
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub url: String,
    #[index]
    pub sha256: Sha256,
}
