mod fs;

use std::{fmt::Debug, pin::Pin, sync::Arc};

use anyhow::Result;
use tracing::info_span;
use tracing_anyhow::FutureContext;

use crate::{
    config::Config,
    model,
    util::{Sha1, Sha256},
};

#[derive(Debug)]
pub struct BlobStore {
    imp: Arc<dyn StoreOps>,
}

impl BlobStore {
    pub async fn new(config: &Config) -> Result<Self> {
        let imp = fs::new(config).await?;
        Ok(Self { imp: Arc::new(imp) })
    }

    pub async fn get(&self, sha256: Sha256) -> Result<Vec<u8>> {
        self.imp.get(sha256).await
    }

    pub async fn put(
        &self,
        exec: &mut dyn toasty::Executor,
        data: &[u8],
    ) -> Result<Sha256> {
        let sha256 = Sha256::from_digest(data);
        let sha1 = Sha1::from_digest(data);
        model::BlobHash::upsert_by_sha256(sha256)
            .sha1(sha1)
            .or_ignore()
            .exec(exec)
            .context(info_span!("inserting blob hash row"))
            .await?;
        self.imp.put(sha256, data).await?;
        Ok(sha256)
    }
}

trait StoreOps: Send + Sync + Debug + 'static {
    fn get(
        &self,
        sha256: Sha256,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>>>>;

    fn put<'a>(
        &self,
        sha256: Sha256,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
}
