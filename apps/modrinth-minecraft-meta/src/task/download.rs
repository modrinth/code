use std::any::type_name;

use anyhow::{Context, Result, anyhow};
use jiff::Timestamp;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState,
    model::{self, DownloadRunId},
    store::BlobCas,
    upstream,
    util::{
        ErrorAccumulator, ResponseExt, Sha256, from_json_str, from_json_value,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Upstreams {
    pub mojang: bool,
    pub fabric: bool,
    pub forge: bool,
    pub neoforge: bool,
    pub quilt: bool,
}

impl Default for Upstreams {
    fn default() -> Self {
        Self {
            mojang: true,
            fabric: true,
            forge: true,
            neoforge: true,
            quilt: true,
        }
    }
}

pub struct DownloadRunContext<'cx> {
    pub http: &'cx reqwest::Client,
    pub cas: &'cx BlobCas,
    pub conn: Mutex<&'cx mut toasty::Connection>,
    pub download_run_id: DownloadRunId,
    pub download_concurrency: usize,
    pub errors: ErrorAccumulator,
}

pub async fn download_from_upstreams(
    app: &AppState,
    upstreams: Upstreams,
) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring db connection"))
        .await?;

    let mut download_run = toasty::create!(model::DownloadRun {
        started_at: Timestamp::now(),
    })
    .exec(&mut conn)
    .context(info_span!("creating download run"))
    .await?;
    info!(?download_run.id, "starting download run");

    let mut cx = DownloadRunContext {
        http: &app.http,
        cas: &app.cas,
        conn: Mutex::new(&mut conn),
        download_run_id: download_run.id,
        download_concurrency: app.concurrency.download.get(),
        errors: ErrorAccumulator::new(),
    };

    if upstreams.mojang {
        let result = upstream::mojang::download(&mut cx)
            .context(info_span!("downloading Mojang upstream"))
            .await;
        result.inspect_err(|err| cx.errors.push(err)).ok();
    }

    if upstreams.fabric {
        let result = upstream::fabric::download(&mut cx)
            .context(info_span!("downloading Fabric upstream"))
            .await;
        result.inspect_err(|err| cx.errors.push(err)).ok();
    }

    if upstreams.neoforge {
        let result = upstream::neoforge::download(&mut cx)
            .context(info_span!("downloading NeoForge upstream"))
            .await;
        result.inspect_err(|err| cx.errors.push(err)).ok();
    }

    if upstreams.quilt {
        let result = upstream::quilt::download(&mut cx)
            .context(info_span!("downloading Quilt upstream"))
            .await;
        result.inspect_err(|err| cx.errors.push(err)).ok();
    }

    // do Forge last since we have to download a lot of installers
    if upstreams.forge {
        let result = upstream::forge::download(&mut cx)
            .context(info_span!("downloading Forge upstream"))
            .await;
        result.inspect_err(|err| cx.errors.push(err)).ok();
    }

    toasty::update!(download_run {
        completed_at: Timestamp::now(),
        errors: toasty::Json(cx.errors.finish()),
    })
    .exec(&mut conn)
    .context(info_span!("marking download run as completed"))
    .await?;

    Ok(())
}

impl<'cx> DownloadRunContext<'cx> {
    pub async fn conn(
        &self,
    ) -> tokio::sync::MutexGuard<'_, &'cx mut toasty::Connection> {
        self.conn.lock().await
    }

    pub async fn download_blob(&self, url: impl IntoUrl) -> Result<Sha256> {
        let url = url.into_url().context("converting to URL")?;
        let bytes = async {
            self.http
                .get(url.clone())
                .send()
                .await?
                .error_for_status_ext()
                .await?
                .bytes()
                .await
                .map_err(anyhow::Error::from)
        }
        .context(info_span!("fetching bytes"))
        .await?;

        let sha256 = self.insert_blob(url.as_str(), &bytes).await?;
        Ok(sha256)
    }

    pub async fn download_json<T: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
    ) -> Result<(T, Sha256)> {
        let url = url.into_url().context("converting to URL")?;
        let text = async {
            self.http
                .get(url.clone())
                .send()
                .await?
                .error_for_status_ext()
                .await?
                .text()
                .await
                .map_err(anyhow::Error::from)
        }
        .context(info_span!("fetching text"))
        .await?;
        let json = from_json_str::<serde_json::Value>(&text)
            .context("text is not valid JSON")?;

        let sha256 = self.insert_blob(url.as_str(), text.as_bytes()).await?;

        let t = from_json_value::<T>(&json)
            .with_context(|| anyhow!("parsing as `{}`", type_name::<T>()))?;
        Ok((t, sha256))
    }

    async fn insert_blob(&self, url: &str, data: &[u8]) -> Result<Sha256> {
        let mut conn = self.conn.lock().await;
        let mut txn = conn
            .transaction()
            .context(info_span!("starting transaction"))
            .await?;

        let sha256 = self
            .cas
            .put(&mut txn, data)
            .context(info_span!("storing blob"))
            .await?;
        toasty::create!(model::BlobDownload {
            download_run_id: self.download_run_id,
            sha256,
            url,
        })
        .exec(&mut txn)
        .context(info_span!("inserting blob download"))
        .await?;

        txn.commit()
            .context(info_span!("committing transaction"))
            .await?;

        Ok(sha256)
    }
}
