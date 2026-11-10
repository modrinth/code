use anyhow::Result;
use tracing::info_span;
use tracing_anyhow::FutureContext;

use crate::{
    model,
    store::{BlobStore, ContentType},
    util::{Sha1, Sha256},
};

/// Stores arbitrary binary [`Vec<u8>`] blobs keyed by the binary data's
/// [`Sha256`] hash.
///
/// This is a _content-addressed store_ (CAS) model.
#[derive(Debug, Clone)]
pub struct BlobCas {
    imp: BlobStore,
}

#[must_use = "hashes will not exist in database until registered"]
pub struct StoredBlob {
    pub sha256: Sha256,
    sha1: Sha1,
}

impl StoredBlob {
    pub async fn register(
        &self,
        exec: &mut dyn toasty::Executor,
    ) -> Result<()> {
        model::BlobHash::upsert_by_sha256(self.sha256)
            .sha1(self.sha1)
            .or_ignore()
            .exec(exec)
            .context(info_span!("upserting blob hash record"))
            .await?;
        Ok(())
    }
}

impl BlobCas {
    #[must_use]
    pub const fn new(imp: BlobStore) -> Self {
        Self { imp }
    }

    pub async fn get(&self, sha256: Sha256) -> Result<Vec<u8>> {
        self.imp.get(&path_for(sha256)).await
    }

    /// Put a blob of data into the store, but don't immediately register it
    /// into the database.
    ///
    /// Use the returned [`StoredBlob`] to add its hashes into the database
    /// later.
    pub async fn store(&self, data: &[u8]) -> Result<StoredBlob> {
        let sha256 = Sha256::from_digest(data);
        let sha1 = Sha1::from_digest(data);
        self.imp
            .put(&path_for(sha256), data, ContentType::Binary)
            .await?;
        Ok(StoredBlob { sha256, sha1 })
    }
}

fn path_for(sha256: Sha256) -> String {
    let sha256 = sha256.to_string();
    let (first, _) = sha256.split_at(2);
    format!("blobs/{first}/{sha256}")
}
