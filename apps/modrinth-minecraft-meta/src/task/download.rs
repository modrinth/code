use std::any::type_name;

use anyhow::{Context, Result, anyhow};
use jiff::Timestamp;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use tracing::{info, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{
    AppState,
    model::{self, DownloadRunId},
    upstream,
    util::{ErrorVec, Sha256, json_from_str, json_from_value},
};

pub async fn download_from_upstreams(state: &AppState) -> Result<()> {
    let mut conn = state
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

    let mut errors = ErrorVec::new();
    let mut cx = DownloadRunContext {
        http: &state.http,
        conn: &mut conn,
        download_run_id: download_run.id,
        errors: &mut errors,
    };

    download_from_mojang(&mut cx)
        .context(info_span!("downloading Mojang upstream"))
        .await
        .inspect_err(|err| warn!("failed to download Mojang upstream: {err:?}"))
        .ok();

    toasty::update!(download_run {
        completed_at: Timestamp::now(),
        errors: toasty::Json(errors),
    })
    .exec(&mut conn)
    .context(info_span!("marking download run as completed"))
    .await?;

    Ok(())
}

async fn download_from_mojang(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let meta_manifest = cx
        .download_json::<upstream::mojang::MetaManifest>(
            upstream::mojang::META_MANIFEST_URL,
        )
        .context(info_span!("fetching meta manifest"))
        .await?;
    info!(
        num_versions = meta_manifest.versions.len(),
        "downloaded Mojang meta manifest"
    );

    for (index, version) in meta_manifest.versions.into_iter().enumerate() {
        cx.download_json::<upstream::mojang::VersionManifest>(
            version.url.clone(),
        )
        .context(
            info_span!("fetching version manifest", version.id = %version.id, %version.url),
        )
        .await
        .inspect_err(|err| cx.errors.push(err))
        .ok();

        if (index + 1) % 10 == 0 {
            info!("downloaded {} version manifests", index);
        }
    }

    Ok(())
}

struct DownloadRunContext<'cx> {
    http: &'cx reqwest::Client,
    conn: &'cx mut toasty::Connection,
    download_run_id: DownloadRunId,
    errors: &'cx mut ErrorVec,
}

impl DownloadRunContext<'_> {
    async fn download_json<T: DeserializeOwned>(
        &mut self,
        url: impl IntoUrl,
    ) -> Result<T> {
        let url = url.into_url().context("converting to URL")?;
        let text =
            async { self.http.get(url.clone()).send().await?.text().await }
                .context(info_span!("fetching text"))
                .await?;
        let sha256 = Sha256::from_digest(text.as_bytes());
        let json = json_from_str::<serde_json::Value>(&text)
            .context("text is not valid JSON")?;

        let mut txn = self
            .conn
            .transaction()
            .context(info_span!("starting transaction"))
            .await?;

        model::JsonBlob::upsert_by_sha256(sha256)
            .json(json.clone())
            .or_ignore()
            .exec(&mut txn)
            .context(info_span!("inserting `JsonBlob`"))
            .await?;
        toasty::create!(model::DownloadBlob {
            download_run_id: self.download_run_id,
            url: url.to_string(),
            sha256,
        })
        .exec(&mut txn)
        .context(info_span!("inserting `DownloadBlob`"))
        .await?;

        txn.commit()
            .context(info_span!("committing transaction"))
            .await?;

        json_from_value::<T>(&json)
            .with_context(|| anyhow!("parsing as `{}`", type_name::<T>()))
    }
}
