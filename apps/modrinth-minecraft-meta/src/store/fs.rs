use std::{path::PathBuf, pin::Pin};

use anyhow::{Context, Result};
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;
use uuid::Uuid;

use crate::store::{StoreOps, StoreVisibility};

#[derive(Debug)]
pub struct FsStore {
    root: PathBuf,
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
    Ok(FsStore { root })
}

impl StoreOps for FsStore {
    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>> {
        // this is vulnerable to path traversal,
        // but we always use wrappers on top of `StoreOps`,
        // which must create well-formed, non-malicious paths.
        let path = self.root.join(path);
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
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        let path = self.root.join(path);
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
