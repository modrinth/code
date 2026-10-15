use std::{path::PathBuf, pin::Pin};

use anyhow::{Context, Result};
use modrinth_minecraft_meta::util::Sha256;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;
use url::Url;
use uuid::Uuid;

use crate::store::{ContentType, PutStatus, StoreOps, StoreVisibility};

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
        _skip_if_matches: Option<Sha256>,
    ) -> Pin<Box<dyn Future<Output = Result<PutStatus>> + 'a>> {
        let path = self
            .root
            .to_file_path()
            .expect("root is a local file URL")
            .join(path);
        Box::pin(async move {
            // avoid write if unchanged
            match fs::read(&path).await {
                Ok(existing) if existing == data => {
                    return Ok(PutStatus::Unchanged);
                }
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(err)
                        .context(format!("reading original file {path:?}"));
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
