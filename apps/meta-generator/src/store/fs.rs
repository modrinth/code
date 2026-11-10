use std::{path::PathBuf, pin::Pin};

use anyhow::{Context, Result};
use minecraft_meta_manifest::util::Sha256;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;
use url::Url;
use uuid::Uuid;

use crate::store::{
    ContentType, PutMode, PutStatus, StoreOps, StoreVisibility,
};

#[derive(Debug)]
pub struct FsStore {
    root: Url,
}

pub async fn new(visibility: StoreVisibility) -> Result<FsStore> {
    let dirs = directories::ProjectDirs::from(
        "com.modrinth",
        "Modrinth",
        "modrinth-minecraft-meta",
    )
    .context("fetching project dirs")?;
    let root = dirs.data_dir().join(match visibility {
        StoreVisibility::Public => "public",
        StoreVisibility::Private => "private",
    });
    fs::create_dir_all(&root)
        .context(info_span!("creating blobs dir"))
        .await?;
    let root = fs::canonicalize(root)
        .context(info_span!("canonicalizing root dir"))
        .await?;
    let root = Url::from_directory_path(root)
        .expect("canonicalized root is an absolute directory path");
    Ok(FsStore { root })
}

impl StoreOps for FsStore {
    fn url_for(&self, path: &str) -> Url {
        let mut url = self.root.clone();
        url.path_segments_mut()
            .expect("root is a directory URL")
            .pop_if_empty()
            .extend(path.split('/'));
        url
    }

    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>> {
        // this is vulnerable to path traversal,
        // but we always use wrappers on top of `StoreOps`,
        // which must create well-formed, non-malicious paths.
        let path = self
            .root
            .to_file_path()
            .expect("root is a local file URL")
            .join(path);
        Box::pin(async move {
            let data = fs::read(&path)
                .context(info_span!("reading file", ?path))
                .await?;
            Ok(data)
        })
    }

    fn matches_hash<'a>(
        &self,
        path: &'a str,
        sha256: Sha256,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + 'a>> {
        let path = self
            .root
            .to_file_path()
            .expect("root is a local file URL")
            .join(path);
        Box::pin(async move {
            match fs::read(&path).await {
                Ok(data) => Ok(Sha256::from_digest(&data) == sha256),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    Ok(false)
                }
                Err(err) => {
                    Err(err).context(format!("reading original file {path:?}"))
                }
            }
        })
    }

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        _content_type: ContentType,
        mode: PutMode,
    ) -> Pin<Box<dyn Future<Output = Result<PutStatus>> + 'a>> {
        let path = self
            .root
            .to_file_path()
            .expect("root is a local file URL")
            .join(path);
        Box::pin(async move {
            if matches!(mode, PutMode::Check) {
                match fs::read(&path).await {
                    Ok(existing) if existing == data => {
                        return Ok(PutStatus::Unchanged);
                    }
                    Ok(_) => {}
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) => {
                        return Err(err).context(format!(
                            "reading original file {path:?}"
                        ));
                    }
                }
            }

            let parent = path.parent().context("path has no parent")?;
            fs::create_dir_all(&parent)
                .context(info_span!("creating parent dir", ?parent))
                .await?;
            let temp_path = parent.join(format!(".{}.tmp", Uuid::now_v7()));
            let _delete_on_drop = DeleteOnDrop {
                path: temp_path.clone(),
            };
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .context(info_span!("creating temporary file", ?temp_path))
                .await?;
            file.write_all(data)
                .context(info_span!("writing temporary file", ?temp_path))
                .await?;
            file.flush()
                .context(info_span!("flushing temporary file", ?temp_path))
                .await?;
            drop(file);
            fs::rename(&temp_path, &path)
                .context(info_span!("publishing file", ?temp_path, ?path))
                .await?;
            Ok(PutStatus::Changed)
        })
    }
}

struct DeleteOnDrop {
    path: PathBuf,
}

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_file(&self.path)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            warn!(temp_path = ?self.path, ?err, "failed to remove temporary file");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::BlobStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn filesystem_put_and_preflight() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("meta-store-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).await?;
        let store = BlobStore {
            imp: Arc::new(FsStore {
                root: Url::from_directory_path(&root).unwrap(),
            }),
        };
        assert_eq!(
            store
                .put("nested/blob", b"old", ContentType::Binary)
                .await?,
            PutStatus::Changed
        );
        assert_eq!(
            store
                .put("nested/blob", b"old", ContentType::Binary)
                .await?,
            PutStatus::Unchanged
        );
        assert_eq!(
            store
                .put_from(
                    "nested/blob",
                    Sha256::from_digest(b"old"),
                    ContentType::Binary,
                    async {
                        panic!("matching destination must not fetch the source")
                    }
                )
                .await?,
            PutStatus::Unchanged
        );
        assert_eq!(
            store
                .put_from(
                    "nested/blob",
                    Sha256::from_digest(b"new"),
                    ContentType::Binary,
                    async { Ok(b"new".to_vec()) }
                )
                .await?,
            PutStatus::Changed
        );
        assert_eq!(store.get("nested/blob").await?, b"new");
        assert_eq!(
            store
                .put_from(
                    "missing",
                    Sha256::from_digest(b"new"),
                    ContentType::Binary,
                    async { Ok(b"new".to_vec()) }
                )
                .await?,
            PutStatus::Changed
        );
        let mut entries = fs::read_dir(root.join("nested")).await?;
        assert_eq!(entries.next_entry().await?.unwrap().file_name(), "blob");
        assert!(entries.next_entry().await?.is_none());
        fs::remove_dir_all(root).await?;
        Ok(())
    }
}
