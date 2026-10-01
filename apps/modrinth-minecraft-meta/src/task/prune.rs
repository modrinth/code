use anyhow::Result;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{AppState, model};

pub async fn prune(state: &AppState, dry_run: bool) -> Result<()> {
    let mut db = state.db.clone();
    let mut txn = db
        .transaction()
        .context(info_span!("acquiring transaction"))
        .await?;

    prune_unused_blobs(&mut txn, dry_run)
        .context(info_span!("pruning unused blobs"))
        .await?;

    txn.commit()
        .context(info_span!("committing transaction"))
        .await?;

    Ok(())
}

async fn prune_unused_blobs(
    txn: &mut toasty::Transaction<'_>,
    dry_run: bool,
) -> Result<()> {
    let mut download_blob_sha256s = model::DownloadBlob::all()
        .select(model::DownloadBlob::fields().sha256())
        // order so that we can dedup after fetching
        .order_by(model::DownloadBlob::fields().sha256().asc())
        .exec(txn)
        .context(info_span!("fetching all download blob sha256s"))
        .await?;
    download_blob_sha256s.dedup();

    if dry_run {
        info!("would delete {} blobs", download_blob_sha256s.len());
    } else {
        info!("deleting {} blobs", download_blob_sha256s.len());

        model::DownloadBlob::all()
            .filter(
                model::DownloadBlob::fields()
                    .sha256()
                    .in_list(download_blob_sha256s.as_slice()),
            )
            .delete()
            .exec(txn)
            .context(info_span!("deleting blobs"))
            .await?;
    }

    Ok(())
}
