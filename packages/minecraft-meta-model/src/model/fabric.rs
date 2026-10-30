use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId},
    util::Sha256,
};

/// ID of a [`FabricCatalog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct FabricCatalogId(pub Uuid);

/// Fabric version catalog downloaded during some [`DownloadRun`].
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
