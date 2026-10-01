use anyhow::Result;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{AppState, model};

pub async fn prune(
    state: &AppState,
    dry_run: bool,
    keep_last_runs: Option<usize>,
) -> Result<()> {
    let mut db = state.db.clone();
    let mut txn = db
        .transaction()
        .context(info_span!("acquiring transaction"))
        .await?;

    if let Some(n) = keep_last_runs {
        do_keep_last_runs(&mut txn, n)
            .context(info_span!("keeping last runs", n))
            .await?;
    }

    prune_orphaned_download_blobs(&mut txn)
        .context(info_span!("pruning orphaned download blobs"))
        .await?;

    prune_orphaned_blobs(&mut txn)
        .context(info_span!("pruning orphaned JSON blobs"))
        .await?;

    if dry_run {
        info!("not committing transaction due to dry run");
        txn.rollback()
            .context(info_span!("rolling back transaction"))
            .await?;
    } else {
        txn.commit()
            .context(info_span!("committing transaction"))
            .await?;
    }

    Ok(())
}

async fn do_keep_last_runs(
    txn: &mut toasty::Transaction<'_>,
    n: usize,
) -> Result<()> {
    let num_runs_before = model::DownloadRun::all()
        .count()
        .exec(txn)
        .context(info_span!("counting download runs"))
        .await?;
    let runs_to_delete = model::DownloadRun::all()
        .select(model::DownloadRun::fields().id())
        .order_by(model::DownloadRun::fields().started_at().desc())
        .order_by(model::DownloadRun::fields().id().desc())
        .limit(i64::MAX as usize)
        .offset(n);
    model::DownloadRun::all()
        .filter(model::DownloadRun::fields().id().in_query(runs_to_delete))
        .delete()
        .exec(txn)
        .context(info_span!("deleting old download runs"))
        .await?;
    let num_runs_after = model::DownloadRun::all()
        .count()
        .exec(txn)
        .context(info_span!("counting download runs"))
        .await?;
    info!(
        "deleted {} of {num_runs_before} download runs",
        num_runs_before - num_runs_after,
    );

    Ok(())
}

async fn prune_orphaned_download_blobs(
    txn: &mut toasty::Transaction<'_>,
) -> Result<()> {
    let num_blobs_before = model::DownloadBlob::all()
        .count()
        .exec(txn)
        .context(info_span!("counting download blobs"))
        .await?;
    let existing_run_ids =
        model::DownloadRun::all().select(model::DownloadRun::fields().id());

    model::DownloadBlob::all()
        .filter(
            model::DownloadBlob::fields()
                .download_run_id()
                .in_query(existing_run_ids)
                .not(),
        )
        .delete()
        .exec(txn)
        .context(info_span!("deleting orphaned download blobs"))
        .await?;

    let num_blobs_after = model::DownloadBlob::all()
        .count()
        .exec(txn)
        .context(info_span!("counting download blobs"))
        .await?;
    info!(
        "deleted {} of {num_blobs_before} orphaned download blobs",
        num_blobs_before - num_blobs_after,
    );

    Ok(())
}

async fn prune_orphaned_blobs(txn: &mut toasty::Transaction<'_>) -> Result<()> {
    let num_blobs_before = model::JsonBlob::all()
        .count()
        .exec(txn)
        .context(info_span!("counting download runs"))
        .await?;

    let mut sha256s_to_keep = model::DownloadBlob::all()
        .select(model::DownloadBlob::fields().sha256())
        // order so that we can dedup after fetching
        .order_by(model::DownloadBlob::fields().sha256().asc())
        .exec(txn)
        .context(info_span!("fetching all download blob sha256s"))
        .await?;
    sha256s_to_keep.dedup();
    info!(
        "keeping {} of {num_blobs_before} blobs",
        sha256s_to_keep.len()
    );

    let sha256s_to_delete = model::JsonBlob::all()
        .select(model::JsonBlob::fields().sha256())
        .filter(
            model::JsonBlob::fields()
                .sha256()
                .in_list(sha256s_to_keep.as_slice())
                .not(),
        )
        .exec(txn)
        .context(info_span!("fetching all json blob sha256s"))
        .await?;

    info!("deleting {} blobs", sha256s_to_delete.len());

    model::JsonBlob::all()
        .filter(
            model::JsonBlob::fields()
                .sha256()
                .in_list(sha256s_to_delete.as_slice()),
        )
        .delete()
        .exec(txn)
        .context(info_span!("deleting blobs"))
        .await?;

    Ok(())
}
