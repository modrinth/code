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
    pub const fn new(imp: BlobStore) -> Self {
        Self { imp }
    }

    /// Publish a CAS blob only if the destination hash differs.
    pub async fn publish(
        &self,
        coordinate: &MavenCoordinate,
        sha256: Sha256,
        cas: &BlobCas,
    ) -> Result<PutStatus> {
        let path = coordinate.to_maven_path();
        self.imp
            .put_from(&path, sha256, ContentType::Binary, async {
                cas.get(sha256).await
            })
            .await
    }
}
