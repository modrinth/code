//! Functions for fetching infromation from the Internet
use crate::config::REQWEST_CLIENT;
use futures::prelude::*;
use std::{collections::LinkedList, convert::TryInto, path::Path, sync::Arc};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::{Semaphore, SemaphorePermit},
};

const FETCH_ATTEMPTS: usize = 3;

#[tracing::instrument(skip(_permit))]
pub async fn fetch<'a>(
    url: &str,
    sha1: Option<&str>,
    _permit: &SemaphorePermit<'a>,
) -> crate::Result<bytes::Bytes> {
    let mut attempts = LinkedList::new();
    for _ in 0..FETCH_ATTEMPTS {
        attempts.push_back(
            async {
                let content = REQWEST_CLIENT.get(url).send().await?;
                let bytes = content.bytes().await?;

                if let Some(hash) = sha1 {
                    let actual_hash = sha1_async(bytes.clone()).await;
                    if actual_hash != hash {
                        return Err(crate::ErrorKind::HashError(
                            actual_hash,
                            String::from(hash),
                        )
                        .into());
                    }
                }

                Ok(bytes)
            }
            .boxed(),
        )
    }

    log::debug!("Done downloading URL {url}");
    future::select_ok(attempts).map_ok(|it| it.0).await
}

// This is implemented, as it will be useful in porting modpacks
// For now, allow it to be dead code
#[allow(dead_code)]
#[tracing::instrument(skip(sem))]
pub async fn fetch_mirrors(
    urls: &[&str],
    sha1: Option<&str>,
    permits: u32,
    sem: &Semaphore,
) -> crate::Result<bytes::Bytes> {
    let _permits = sem.acquire_many(permits).await.unwrap();
    let sem = Arc::new(Semaphore::new(permits.try_into().unwrap()));

    future::select_ok(urls.iter().map(|url| {
        let sha1 = sha1.map(String::from);
        let url = String::from(*url);
        let sem = Arc::clone(&sem);

        tokio::spawn(async move {
            let permit = sem.acquire().await.unwrap();
            fetch(&url, sha1.as_deref(), &permit).await
        })
        .map(Result::unwrap)
        .boxed()
    }))
    .await
    .map(|it| it.0)
}

#[tracing::instrument(skip(bytes, _permit))]
pub async fn write<'a>(
    path: &Path,
    bytes: &[u8],
    _permit: &SemaphorePermit<'a>,
) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;
    log::debug!("Done writing file {}", path.display());
    file.write_all(bytes).await?;
    Ok(())
}

async fn sha1_async(bytes: bytes::Bytes) -> String {
    tokio::task::spawn_blocking(move || sha1::Sha1::from(bytes).hexdigest())
        .await
        .unwrap()
}
