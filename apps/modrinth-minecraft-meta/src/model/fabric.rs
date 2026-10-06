use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId},
    util::Sha256,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct FabricCatalogId(pub Uuid);

#[derive(Debug, Clone, Model)]
pub struct FabricCatalog {
    #[key]
    #[auto]
    pub id: FabricCatalogId,
    #[unique]
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub sha256: Sha256,
}
