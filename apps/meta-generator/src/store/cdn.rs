use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use tracing::info_span;
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    store::{ContentType, StoreOps},
    util::fastly,
};

#[derive(Debug)]
pub struct CdnStore {
    imp: Box<dyn StoreOps>,
    fastly: Arc<fastly::Client>,
}

pub fn new(imp: Box<dyn StoreOps>, fastly: Arc<fastly::Client>) -> CdnStore {
    CdnStore { imp, fastly }
}

impl StoreOps for CdnStore {
    fn url_for(&self, path: &str) -> Url {
        self.imp.url_for(path)
    }

    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>> {
        self.imp.get(path)
    }

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        content_type: ContentType,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        let fut = self.imp.put(path, data, content_type);
        let fastly = self.fastly.clone();
        let url = self.imp.url_for(path);
        Box::pin(async move {
            fut.await?;
            fastly
                .purge_url(&url)
                .context(info_span!("purging Fastly cache"))
                .await?;
            anyhow::Ok(())
        })
    }
}
