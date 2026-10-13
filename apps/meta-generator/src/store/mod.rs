mod cas;
mod cdn;
mod fs;
mod maven;
mod s3;

pub use cas::BlobCas;
pub use maven::MavenStore;
use url::Url;

use std::{fmt::Debug, pin::Pin, sync::Arc};

use anyhow::Result;

use crate::{
    config::{self},
    util::fastly,
};

/// Stores arbitrary binary [`Vec<u8>`] blobs at a string `path`.
///
/// If you're looking for content-addressed storage (store by a SHA256 hash),
/// see [`ContentStore`].
#[derive(Debug, Clone)]
pub struct BlobStore {
    imp: Arc<dyn StoreOps>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Binary,
}

impl ContentType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Binary => "application/octet-stream",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreVisibility {
    Public,
    Private,
}

impl BlobStore {
    pub async fn new(
        s3_config: Option<config::S3>,
        fastly: Option<Arc<fastly::Client>>,
        visibility: StoreVisibility,
    ) -> Result<Self> {
        let mut imp: Box<dyn StoreOps> = match s3_config {
            Some(config) => Box::new(s3::new(&config)?),
            None => Box::new(fs::new(visibility).await?),
        };

        if visibility == StoreVisibility::Public
            && let Some(fastly) = fastly
        {
            imp = Box::new(cdn::new(imp, fastly));
        }

        Ok(Self {
            imp: Arc::from(imp),
        })
    }

    /// Get the [`Url`] for a file at a specific `path`.
    pub fn url_for(&self, path: &str) -> Url {
        self.imp.url_for(path)
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
    pub async fn put(
        &self,
        path: &str,
        data: &[u8],
        content_type: ContentType,
    ) -> Result<()> {
        self.imp.put(path, data, content_type).await
    }
}

trait StoreOps: Send + Sync + Debug + 'static {
    fn url_for(&self, path: &str) -> Url;

    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>>;

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        content_type: ContentType,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
}
