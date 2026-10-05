use std::{path::PathBuf, pin::Pin};

use anyhow::Result;
use tokio::fs;
use tracing::info_span;
use tracing_anyhow::FutureContext;

use crate::{config::Config, store::StoreOps, util::Sha256};

#[derive(Debug)]
pub struct FsStore {
    root: PathBuf,
}

pub async fn new(config: &Config) -> Result<FsStore> {
    let root = config.data_directory.join("blobs");
    fs::create_dir_all(&root)
        .context(info_span!("creating blobs dir"))
        .await?;
    Ok(FsStore { root })
}

impl StoreOps for FsStore {
    fn get(
        &self,
        sha256: Sha256,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>>>> {
        let sha256 = sha256.to_string();
        let (first, _) = sha256.split_at(2);
        let path = self.root.join(first).join(&sha256);
        Box::pin(async move {
            let data = fs::read(&path)
                .context(info_span!("reading file", ?path))
                .await?;
            Ok(data)
        })
    }

    fn put<'a>(
        &self,
        sha256: Sha256,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        let sha256 = sha256.to_string();
        let (first, _) = sha256.split_at(2);
        let parent = self.root.join(first);
        let path = parent.join(&sha256);
        Box::pin(async move {
            fs::create_dir_all(&parent)
                .context(info_span!("creating parent dir", ?parent))
                .await?;
            fs::write(&path, data)
                .context(info_span!("writing file", ?path))
                .await?;
            Ok(())
        })
    }
}
