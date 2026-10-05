use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId},
    util::Sha256,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct MojangCatalogId(pub Uuid);

#[derive(Debug, Clone, Model)]
pub struct MojangCatalog {
    #[key]
    #[auto]
    pub id: MojangCatalogId,
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub sha256: Sha256,
}
