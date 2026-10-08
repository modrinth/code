use std::pin::Pin;

use anyhow::{Context, Result};
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;
use url::Url;
use uuid::Uuid;

use crate::store::{ContentType, StoreOps, StoreVisibility};

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

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        _content_type: ContentType,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        let path = self
            .root
            .to_file_path()
            .expect("root is a local file URL")
            .join(path);
        Box::pin(async move {
            let parent = path.parent().context("path has no parent")?;
            fs::create_dir_all(&parent)
                .context(info_span!("creating parent dir", ?parent))
                .await?;
            let temp_path = parent.join(format!(".{}.tmp", Uuid::now_v7()));
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .context(info_span!("creating temporary file", ?temp_path))
                .await?;
            let result = async {
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
                Ok(())
            }
            .await;
            if result.is_err()
                && let Err(err) = fs::remove_file(&temp_path).await
            {
                warn!(?temp_path, ?err, "failed to remove temporary file");
            }
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_preserve_directory_slashes_and_encode_file_names() {
        let store = FsStore {
            root: Url::from_directory_path(
                std::env::temp_dir().join("minecraft-meta-url-test"),
            )
            .unwrap(),
        };
        let directory = store.url_for("maven/");
        assert!(directory.as_str().ends_with("/maven/"));
        assert_eq!(
            directory.join("net/fabricmc/library.jar").unwrap(),
            store.url_for("maven/net/fabricmc/library.jar")
        );
        let file =
            store.url_for("minecraft/v0/versions/3D Shareware v1.34.json");
        assert!(file.as_str().ends_with("/3D%20Shareware%20v1.34.json"));
        assert!(!file.as_str().ends_with('/'));
        assert!(!store.url_for("maven").as_str().ends_with('/'));
        let literal = store.url_for("maven/file%20?#.jar");
        assert!(literal.query().is_none());
        assert!(literal.fragment().is_none());
        assert_eq!(
            literal.to_file_path().unwrap(),
            store
                .root
                .to_file_path()
                .unwrap()
                .join("maven/file%20?#.jar")
        );
    }
}
