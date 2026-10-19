use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId},
    util::Sha256,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct NeoforgeCatalogId(pub Uuid);

#[derive(Debug, Clone, Model)]
pub struct NeoforgeCatalog {
    #[key]
    #[auto]
    pub id: NeoforgeCatalogId,
    #[unique]
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub forge_sha256: Sha256,
    pub neoforge_sha256: Sha256,
}

/// Specific NeoForge installer versions which we can't download or process.
pub const NEOFORGE_BLACKLIST: &[&str] = &[
    // listed upstream, but installer JARs and POMs return 404
    "1.20.1-47.1.7",
    "47.1.82",
    // the installer references a missing universal JAR; upstream and launcher-meta
    // return 404, although production publishes a manifest referencing that JAR
    "21.4.31-beta",
];
