use derive_more::Display;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId},
    util::Sha256,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub struct ForgeCatalogId(pub Uuid);

#[derive(Debug, Clone, Model)]
pub struct ForgeCatalog {
    #[key]
    #[auto]
    pub id: ForgeCatalogId,
    #[unique]
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub sha256: Sha256,
}

/// Name of a Minecraft game version, plus a `-`, plus the Forge loader
/// version name.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Embed,
)]
pub struct ForgeInstallerName(pub String);

#[derive(Debug, Clone, Model)]
pub struct ForgeInstaller {
    #[key]
    pub name: ForgeInstallerName,
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    pub sha256: Sha256,
    pub processed_at: Option<Timestamp>,
}
