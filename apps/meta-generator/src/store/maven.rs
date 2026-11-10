use anyhow::Result;

use crate::{
    store::{BlobCas, BlobStore, ContentType, PutStatus},
    util::{MavenCoordinate, Sha256},
};

#[derive(Debug, Clone)]
pub struct MavenStore {
    imp: BlobStore,
}

impl MavenStore {
    #[must_use]
    pub fn new(imp: BlobStore) -> Self {
        Self { imp }
    }

    /// Publishes an artifact, fetching it from CAS only if the destination differs.
    pub async fn publish(
        &self,
        coordinate: &MavenCoordinate,
        sha256: Sha256,
        cas: &BlobCas,
    ) -> Result<PutStatus> {
        self.imp
            .put_from(
                &coordinate.to_maven_path(),
                sha256,
                ContentType::Binary,
                async { cas.get(sha256).await },
            )
            .await
    }
}
