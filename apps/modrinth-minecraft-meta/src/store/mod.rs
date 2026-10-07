mod cas;
mod fs;
mod maven;

pub use cas::BlobCas;
pub use maven::MavenStore;

use std::{fmt::Debug, pin::Pin, sync::Arc};

use anyhow::Result;

use crate::config::Config;

/// Stores arbitrary binary [`Vec<u8>`] blobs at a string `path`.
///
/// If you're looking for content-addressed storage (store by a SHA256 hash),
/// see [`ContentStore`].
#[derive(Debug, Clone)]
pub struct BlobStore {
    imp: Arc<dyn StoreOps>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreVisibility {
    Public,
    Private,
}

impl BlobStore {
    pub async fn new(
        config: &Config,
        visibility: StoreVisibility,
    ) -> Result<Self> {
        _ = config;
        fs::new(visibility).await.map(|store| Self {
            imp: Arc::new(store),
        })
    }

    /// Get a blob at a `path`.
    ///
    /// The `path` here may be vulnerable to directory traversal if using the
    /// filesystem store; it is the caller's responsibility to ensure this is
    /// a well-formed path with no traversal.
    pub async fn get(&self, path: &str) -> Result<Vec<u8>> {
        self.imp.get(path).await
    }

    /// Put a blob `data` at `path`.
    pub async fn put(&self, path: &str, data: &[u8]) -> Result<()> {
        self.imp.put(path, data).await
    }
}

trait StoreOps: Send + Sync + Debug + 'static {
    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>>;

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
}
