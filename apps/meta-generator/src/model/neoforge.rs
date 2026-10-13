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
