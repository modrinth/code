mod cas;
mod fs;
mod maven;
mod s3;

pub use cas::BlobCas;
pub use maven::MavenStore;
use minecraft_meta_manifest::util::Sha256;
use url::Url;

use std::{fmt::Debug, pin::Pin, sync::Arc};

use anyhow::{Result, ensure};

use crate::config;

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
        visibility: StoreVisibility,
    ) -> Result<Self> {
        let imp: Box<dyn StoreOps> = match s3_config {
            Some(config) => Box::new(s3::new(&config)?),
            None => Box::new(fs::new(visibility).await?),
        };
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

    /// Put a blob `data` at `path`, returning whether its contents changed.
    pub async fn put(
        &self,
        path: &str,
        data: &[u8],
        content_type: ContentType,
    ) -> Result<PutStatus> {
        self.imp.put(path, data, content_type, PutMode::Check).await
    }

    /// Fetch the source only when the destination hash differs, validating it before writing.
    pub(crate) async fn put_from(
        &self,
        path: &str,
        sha256: Sha256,
        content_type: ContentType,
        source: impl Future<Output = Result<Vec<u8>>>,
    ) -> Result<PutStatus> {
        if self.imp.matches_hash(path, sha256).await? {
            return Ok(PutStatus::Unchanged);
        }
        let data = source.await?;
        ensure!(
            Sha256::from_digest(&data) == sha256,
            "source SHA256 mismatch for {path}"
        );
        self.imp
            .put(path, &data, content_type, PutMode::Prechecked(sha256))
            .await
    }
}

#[derive(Debug, Clone, Copy)]
enum PutMode {
    Check,
    Prechecked(Sha256),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutStatus {
    Changed,
    Unchanged,
}

trait StoreOps: Send + Sync + Debug + 'static {
    fn url_for(&self, path: &str) -> Url;

    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>>;

    fn matches_hash<'a>(
        &self,
        path: &'a str,
        sha256: Sha256,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + 'a>>;

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        content_type: ContentType,
        mode: PutMode,
    ) -> Pin<Box<dyn Future<Output = Result<PutStatus>> + 'a>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct MockStore {
        matches: bool,
        fail: bool,
        checks: AtomicUsize,
        writes: AtomicUsize,
    }

    impl StoreOps for MockStore {
        fn url_for(&self, _: &str) -> Url {
            unreachable!()
        }

        fn get<'a>(
            &self,
            _: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>> {
            unreachable!()
        }

        fn matches_hash<'a>(
            &self,
            _: &'a str,
            _: Sha256,
        ) -> Pin<Box<dyn Future<Output = Result<bool>> + 'a>> {
            self.checks.fetch_add(1, Ordering::SeqCst);
            let (matches, fail) = (self.matches, self.fail);
            Box::pin(async move {
                ensure!(!fail, "preflight failed");
                Ok(matches)
            })
        }

        fn put<'a>(
            &self,
            _: &'a str,
            data: &'a [u8],
            _: ContentType,
            mode: PutMode,
        ) -> Pin<Box<dyn Future<Output = Result<PutStatus>> + 'a>> {
            let PutMode::Prechecked(hash) = mode else {
                panic!("write must not recheck the destination");
            };
            assert_eq!(hash, Sha256::from_digest(data));
            self.writes.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(PutStatus::Changed) })
        }
    }

    #[tokio::test]
    async fn lazy_preflight() {
        for (matches, fail, expected_reads, expected_writes) in [
            (true, false, 0, 0),
            (false, false, 1, 1),
            (false, true, 0, 0),
        ] {
            let imp = Arc::new(MockStore {
                matches,
                fail,
                checks: AtomicUsize::new(0),
                writes: AtomicUsize::new(0),
            });
            let store = BlobStore { imp: imp.clone() };
            let reads = AtomicUsize::new(0);
            let result = store
                .put_from(
                    "artifact",
                    Sha256::from_digest(b"data"),
                    ContentType::Binary,
                    async {
                        reads.fetch_add(1, Ordering::SeqCst);
                        Ok(b"data".to_vec())
                    },
                )
                .await;
            if fail {
                assert!(result.is_err());
            } else {
                assert_eq!(
                    result.unwrap(),
                    if matches {
                        PutStatus::Unchanged
                    } else {
                        PutStatus::Changed
                    }
                );
            }
            assert_eq!(imp.checks.load(Ordering::SeqCst), 1);
            assert_eq!(reads.load(Ordering::SeqCst), expected_reads);
            assert_eq!(imp.writes.load(Ordering::SeqCst), expected_writes);
        }
    }

    #[tokio::test]
    async fn invalid_or_failed_source_never_writes() {
        let imp = Arc::new(MockStore {
            matches: false,
            fail: false,
            checks: AtomicUsize::new(0),
            writes: AtomicUsize::new(0),
        });
        let store = BlobStore { imp: imp.clone() };
        let hash = Sha256::from_digest(b"expected");
        assert!(
            store
                .put_from("artifact", hash, ContentType::Binary, async {
                    Ok(b"corrupt".to_vec())
                })
                .await
                .is_err()
        );
        assert!(
            store
                .put_from("artifact", hash, ContentType::Binary, async {
                    anyhow::bail!("source failed")
                })
                .await
                .is_err()
        );
        assert_eq!(imp.writes.load(Ordering::SeqCst), 0);
    }
}
