use derive_more::Display;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::{Embed, Model};
use uuid::Uuid;

use crate::{
    model::{DownloadRun, DownloadRunId, ProfileMetadata},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum ForgelikeLoader {
    Forge,
    Neoforge,
}

/// Full upstream version name identifying a Forge or NeoForge installer.
///
/// Forge uses `<minecraft version>-<loader version>`; modern NeoForge uses
/// its loader version alone.
#[derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Embed,
)]
pub struct ForgelikeInstallerName(pub String);

#[derive(Debug, Clone, Model)]
#[key(loader, name)]
pub struct ForgelikeInstaller {
    pub loader: ForgelikeLoader,
    pub name: ForgelikeInstallerName,
    pub download_run_id: DownloadRunId,
    #[belongs_to]
    pub download_run: toasty::Deferred<DownloadRun>,
    #[index]
    pub sha256: Sha256,
}

/// Parsed metadata and embedded artifact paths from a Forge or NeoForge installer.
#[derive(Debug, Clone, Model)]
pub struct ForgelikeExtract {
    /// [`Sha256`] hash of the installer JAR file which we extracted this
    /// information from.
    ///
    /// We key on a file's hash since that's the source of truth for this info;
    /// not a specific installer record.
    #[key]
    pub installer_sha256: Sha256,
    /// When this installer was extracted, and the extraction record was
    /// created.
    #[auto]
    pub created_at: Timestamp,
    #[column(type = json)]
    pub metadata: toasty::Json<ProfileMetadata>,
    /// Paths inside the installer, not references to extracted blobs.
    #[column(type = json)]
    pub embedded_maven_artifacts: toasty::Json<Vec<String>>,
}
