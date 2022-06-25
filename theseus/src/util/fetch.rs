//! Functions for fetching infromation from the Internet
use futures::prelude::*;
use std::{collections::LinkedList, path::Path, time};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

const FETCH_ATTEMPTS: usize = 3;

pub async fn fetch(
    url: &str,
    sha1: Option<&str>,
) -> crate::Result<bytes::Bytes> {
    let st = crate::State::get().await?;
    let _permit = st.io_semaphore.acquire().await.unwrap();

    let client = reqwest::Client::builder()
        .tcp_keepalive(Some(time::Duration::from_secs(10)))
        .build()?;

    let mut attempts = LinkedList::new();
    for _ in 0..FETCH_ATTEMPTS {
        attempts.push_back(
            async {
                let content = client.get(url).send().await?;
                let bytes = content.bytes().await?;

                if let Some(hash) = sha1 {
                    let actual_hash = sha1_async(bytes.clone()).await;
                    if actual_hash != hash {
                        return Err(crate::Error::HashError(
                            actual_hash,
                            String::from(hash),
                        ));
                    }
                }

                Ok(bytes)
            }
            .boxed(),
        )
    }

    future::select_ok(attempts).map_ok(|it| it.0).await
}

pub async fn write(path: &Path, bytes: &[u8]) -> crate::Result<()> {
    let st = crate::State::get().await?;
    let _permit = st.io_semaphore.acquire().await.unwrap();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;
    file.write_all(bytes).await?;
    Ok(())
}

async fn sha1_async(bytes: bytes::Bytes) -> String {
    tokio::task::spawn_blocking(move || sha1::Sha1::from(bytes).hexdigest())
        .await
        .unwrap()
}
