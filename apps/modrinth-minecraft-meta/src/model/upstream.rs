use jiff::Timestamp;
use toasty::{Embed, Model};
use uuid::Uuid;

use crate::util::{ErrorVec, Sha256};

/// Generic content-addressed JSON blob, keyed by the [`Blob::data`]'s
/// [`Sha256`].
#[derive(Debug, Clone, Model)]
pub struct JsonBlob {
    /// [`Sha256`] digest of the [`Blob::data`].
    #[key]
    pub sha256: Sha256,
    /// Raw JSON of this entity.
    #[column(type = text)]
    pub json: serde_json::Value,
}

/// ID for a [`DownloadRun`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct DownloadRunId(pub Uuid);

/// Single run of a task to download sources from our upstreams.
///
/// During a [`DownloadRun`], we download a bunch of files and save them into
/// our database as [`DownloadBlob`]s. Any blobs saved within the same run can
/// be logically grouped together.
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

/// ID of a [`DownloadBlob`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct DownloadBlobId(pub Uuid);

/// Single file downloaded as part of a [`DownloadRun`].
///
/// This is unique on `(download_run_id, url)` - so during a single run, we will
/// only download any given file at a URL once.
///
/// If we do two runs close together in time, and they download the same file,
/// we'll create two [`DownloadBlob`]s with the same URL but different IDs.
/// However, we won't duplicate the contents of the download - that's
/// deduplicated by the [`JsonBlob`] table, and we just store a [`Sha256`] key
/// into that.
#[derive(Debug, Clone, Model)]
#[unique(download_run_id, url)]
pub struct DownloadBlob {
    #[key]
    #[auto]
    pub id: DownloadBlobId,
    /// ID of the [`DownloadRun`] during which we downloaded this blob.
    pub download_run_id: DownloadRunId,
    /// [`DownloadRun`] during which we downloaded this blob.
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    /// URL at which we downloaded this file.
    pub url: String,
    /// [`Sha256`] of the [`JsonBlob`] that we downloaded.
    #[index]
    pub sha256: Sha256,
    /// [`JsonBlob`] that we downloaded.
    #[belongs_to(key = sha256, references = sha256)]
    pub json: toasty::Deferred<JsonBlob>,
}
