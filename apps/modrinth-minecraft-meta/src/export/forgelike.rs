use std::{
    collections::{BTreeMap, HashSet},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Result, ensure};
use futures::{TryStreamExt, stream};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState,
    model::{
        ForgelikeExtract, ForgelikeInstaller, ForgelikeLoader,
        ForgelikeStoredArtifact,
    },
    upstream::Upstreams,
};

/// Publishes referenced embedded artifacts only; Forge-like manifests are not implemented.
pub async fn export(app: &AppState, upstreams: Upstreams) -> Result<()> {
    let mut loaders = Vec::new();
    if upstreams.forge {
        loaders.push(ForgelikeLoader::Forge);
    }
    if upstreams.neoforge {
        loaders.push(ForgelikeLoader::Neoforge);
    }
    if loaders.is_empty() {
        return Ok(());
    }
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring Forge-like export connection"))
        .await?;
    let mut txn = conn
        .transaction()
        .context(info_span!("starting Forge-like export snapshot"))
        .await?;
    toasty::sql::statement(
        "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY",
    )
    .exec(&mut txn)
    .context(info_span!("setting Forge-like export snapshot isolation"))
    .await?;
    let fields = ForgelikeInstaller::fields();
    let installers = ForgelikeInstaller::all()
        .filter(fields.loader().in_list(loaders.clone()))
        .select((fields.loader(), fields.sha256()))
        .exec(&mut txn)
        .context(info_span!("fetching selected Forge-like installers"))
        .await?;
    for loader in loaders {
        ensure!(
            installers.iter().any(|(candidate, _)| *candidate == loader),
            "no imported {loader:?} installers; run import before exporting"
        );
    }
    let installer_hashes = installers
        .into_iter()
        .map(|(_, sha256)| sha256)
        .collect::<HashSet<_>>();
    let extracts = ForgelikeExtract::all()
        .filter(
            ForgelikeExtract::fields()
                .installer_sha256()
                .in_list(installer_hashes.iter().copied().collect::<Vec<_>>()),
        )
        .exec(&mut txn)
        .context(info_span!("fetching selected Forge-like extracts"))
        .await?;
    let extracted_hashes = extracts
        .iter()
        .map(|extract| extract.installer_sha256)
        .collect::<HashSet<_>>();
    let missing = installer_hashes.difference(&extracted_hashes).count();
    ensure!(
        missing == 0,
        "{missing} Forge-like installers have no extracted metadata; run import before exporting"
    );
    txn.commit()
        .context(info_span!("finishing Forge-like export snapshot"))
        .await?;
    drop(conn);

    let artifacts = deduplicate_artifacts(
        extracts
            .into_iter()
            .flat_map(|extract| extract.embedded_maven_artifacts.0),
    )?;
    let num_total = artifacts.len();
    let num_done = AtomicUsize::new(0);
    info!(
        num_artifacts = num_total,
        "publishing Forge-like embedded artifacts"
    );
    let num_done = &num_done;
    stream::iter(artifacts.into_iter().map(Ok::<_, anyhow::Error>))
        .try_for_each_concurrent(app.concurrency.extract_files.get(), |artifact| async move {
            let bytes = app.cas.get(artifact.sha256)
                .context(info_span!("reading embedded Maven artifact", coordinate = %artifact.coordinate, sha256 = %artifact.sha256))
                .await?;
            app.maven.put(&artifact.coordinate, &bytes)
                .context(info_span!("publishing embedded Maven artifact", coordinate = %artifact.coordinate))
                .await?;
            let done = num_done.fetch_add(1, Ordering::Relaxed) + 1;
            if done.is_multiple_of(100) || done == num_total {
                info!("published {done}/{num_total} Forge-like embedded artifacts");
            }
            anyhow::Ok(())
        })
        .await?;
    info!(
        num_artifacts = num_total,
        "finished publishing Forge-like embedded artifacts"
    );
    Ok(())
}

fn deduplicate_artifacts(
    artifacts: impl IntoIterator<Item = ForgelikeStoredArtifact>,
) -> Result<Vec<ForgelikeStoredArtifact>> {
    let mut paths = BTreeMap::<String, ForgelikeStoredArtifact>::new();
    for artifact in artifacts {
        let path = artifact.coordinate.to_maven_path();
        if let Some(existing) = paths.get(&path) {
            ensure!(
                existing.sha256 == artifact.sha256,
                "conflicting embedded Maven artifacts at {path}: {} and {}",
                existing.sha256,
                artifact.sha256
            );
        } else {
            paths.insert(path, artifact);
        }
    }
    Ok(paths.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::Sha256;

    fn artifact(coordinate: &str, bytes: &[u8]) -> ForgelikeStoredArtifact {
        ForgelikeStoredArtifact {
            coordinate: coordinate.parse().unwrap(),
            sha256: Sha256::from_digest(bytes),
        }
    }

    #[test]
    fn deduplicates_coordinate_paths_including_default_extension() {
        let artifacts = deduplicate_artifacts([
            artifact("net.example:library:1", b"same"),
            artifact("net.example:library:1@jar", b"same"),
        ])
        .unwrap();
        assert_eq!(artifacts.len(), 1);
    }

    #[test]
    fn rejects_conflicting_content_before_publication() {
        assert!(
            deduplicate_artifacts([
                artifact("net.example:library:1", b"first"),
                artifact("net.example:library:1", b"second"),
            ])
            .is_err()
        );
    }
}
