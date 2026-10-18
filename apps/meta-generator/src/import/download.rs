use std::any::type_name;

use anyhow::{Context, Result, anyhow};
use jiff::Timestamp;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tracing::{Instrument, info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState,
    import::{fabric, forge, mojang, neoforge, quilt},
    model::{self, DownloadRunId},
    store::BlobCas,
    upstream::Upstreams,
    util::{
        ErrorAccumulator, ResponseExt, Sha256, TaskContext, from_json_str,
        from_json_value,
    },
};

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
) -> Result<Upstreams> {
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

    let mut completed = Upstreams {
        mojang: false,
        fabric: false,
        forge: false,
        neoforge: false,
        quilt: false,
    };

    if upstreams.mojang {
        let errors_before = cx.errors.len();
        let result = mojang::download(&mut cx)
            .transaction_context(|| info_span!("downloading Mojang upstream"))
            .await;
        completed.mojang =
            download_succeeded(result, &cx.errors, errors_before);
    }

    if upstreams.fabric {
        let errors_before = cx.errors.len();
        let result = fabric::download(&mut cx)
            .transaction_context(|| info_span!("downloading Fabric upstream"))
            .await;
        completed.fabric =
            download_succeeded(result, &cx.errors, errors_before);
    }

    if upstreams.quilt {
        let errors_before = cx.errors.len();
        let result = quilt::download(&mut cx)
            .transaction_context(|| info_span!("downloading Quilt upstream"))
            .await;
        completed.quilt = download_succeeded(result, &cx.errors, errors_before);
    }

    // do Forgelikes last since we have to download a lot of installers
    if upstreams.forge {
        let errors_before = cx.errors.len();
        let result = forge::download(&mut cx)
            .transaction_context(|| info_span!("downloading Forge upstream"))
            .await;
        completed.forge = download_succeeded(result, &cx.errors, errors_before);
    }

    if upstreams.neoforge {
        let errors_before = cx.errors.len();
        let result = neoforge::download(&mut cx)
            .transaction_context(|| info_span!("downloading NeoForge upstream"))
            .await;
        completed.neoforge =
            download_succeeded(result, &cx.errors, errors_before);
    }

    toasty::update!(download_run {
        completed_at: Timestamp::now(),
        errors: toasty::Json(cx.errors.finish_into_messages()),
    })
    .exec(&mut conn)
    .context(info_span!("marking download run as completed"))
    .await?;

    Ok(completed)
}

fn download_succeeded(
    result: Result<()>,
    errors: &ErrorAccumulator,
    errors_before: usize,
) -> bool {
    if let Err(err) = result {
        errors.push(&err);
        return false;
    }
    errors.len() == errors_before
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
        let blob = self
            .cas
            .store(data)
            .context(info_span!("storing blob"))
            .await?;
        let sha256 = blob.sha256;
        let mut conn = self
            .conn
            .lock()
            .instrument(info_span!("taking connection lock"))
            .await;
        let mut txn = conn
            .transaction()
            .context(info_span!("starting transaction"))
            .await?;

        blob.register(&mut txn).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_success_requires_no_new_recorded_errors() {
        let errors = ErrorAccumulator::new();
        assert!(download_succeeded(Ok(()), &errors, 0));
        errors.push(&anyhow!("failed to download a profile"));
        assert!(!download_succeeded(Ok(()), &errors, 0));
        let previous = errors.len();
        assert!(download_succeeded(Ok(()), &errors, previous));
        assert!(!download_succeeded(
            Err(anyhow!("failed to fetch catalog")),
            &errors,
            previous
        ));
        assert_eq!(errors.len(), previous + 1);
    }
}
