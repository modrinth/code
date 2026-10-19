mod name;
pub use name::ForgelikeInstallerName;

use crate::model::NEOFORGE_BLACKLIST;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::{Embed, Model};

use crate::{
    model::{DownloadRun, DownloadRunId, ProfileMetadata},
    util::{MavenCoordinate, Sha256},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum ForgelikeLoader {
    Forge,
    Neoforge,
}

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

impl ForgelikeInstaller {
    pub fn is_blacklisted(&self) -> bool {
        match self.loader {
            ForgelikeLoader::Forge => false,
            ForgelikeLoader::Neoforge => {
                NEOFORGE_BLACKLIST.contains(&self.name.as_str())
            }
        }
    }
}

/// An embedded or downloaded Maven artifact retained in content-addressed storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgelikeStoredArtifact {
    pub coordinate: MavenCoordinate,
    pub sha256: Sha256,
}

/// Parsed metadata and stored dependencies from a Forge or NeoForge installer.
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
    /// Referenced embedded and downloaded artifacts, with coordinates and CAS hashes.
    /// The field name is retained for compatibility with existing extraction records.
    #[column(type = json)]
    pub embedded_maven_artifacts: toasty::Json<Vec<ForgelikeStoredArtifact>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stored_artifact_round_trips_and_requires_hash() {
        let value = json!({
            "coordinate": "net.example:library:1",
            "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
        });
        let stored: ForgelikeStoredArtifact =
            serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(stored).unwrap(), value);
        assert!(
            serde_json::from_value::<ForgelikeStoredArtifact>(json!({
                "coordinate": "net.example:library:1"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<Vec<ForgelikeStoredArtifact>>(json!([
                "maven/net/example/library/1/library-1.jar"
            ]))
            .is_err()
        );
    }
}
