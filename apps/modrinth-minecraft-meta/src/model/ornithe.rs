use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId},
    util::Sha256,
};

/// ID of an [`OrnitheCatalog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct OrnitheCatalogId(pub Uuid);

/// Ornithe game and loader version lists downloaded during some
/// [`DownloadRun`].
#[derive(Debug, Clone, Model)]
pub struct OrnitheCatalog {
    #[key]
    #[auto]
    pub id: OrnitheCatalogId,
    #[unique]
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub games_sha256: Sha256,
    pub loaders_sha256: Sha256,
}
