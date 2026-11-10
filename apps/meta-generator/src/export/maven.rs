use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::RwLock;
use tracing::info_span;
use tracing_anyhow::FutureContext;

use crate::{
    AppState,
    model::MavenExport,
    store::PutStatus,
    util::{MavenCoordinate, Sha256},
};

/// Publication ledger scoped to one export run. Callers deduplicate paths within
/// each concurrent batch and finish each batch before starting the next.
pub(super) struct MavenExports<'a> {
    db: &'a toasty::Db,
    exports: RwLock<HashMap<String, Sha256>>,
}

impl<'a> MavenExports<'a> {
    pub async fn load(db: &'a toasty::Db) -> Result<Self> {
        let mut conn = db
            .connection()
            .context(info_span!("acquiring Maven export connection"))
            .await?;
        let rows = MavenExport::all()
            .exec(&mut conn)
            .context(info_span!("loading Maven exports"))
            .await?;
        Ok(Self {
            db,
            exports: RwLock::new(
                rows.into_iter().map(|row| (row.path, row.sha256)).collect(),
            ),
        })
    }

    pub async fn is_published(
        &self,
        coordinate: &MavenCoordinate,
        sha256: Sha256,
    ) -> bool {
        self.exports.read().await.get(&coordinate.to_maven_path())
            == Some(&sha256)
    }

    pub async fn publish(
        &self,
        app: &AppState,
        coordinate: &MavenCoordinate,
        sha256: Sha256,
    ) -> Result<PutStatus> {
        let path = coordinate.to_maven_path();
        if self.is_published(coordinate, sha256).await {
            return Ok(PutStatus::Unchanged);
        }
        let exports = &self.exports;
        let mut conn = self
            .db
            .connection()
            .context(info_span!("acquiring Maven export connection"))
            .await?;
        // A prior upsert may have committed before its cache update was cancelled.
        MavenExport::all()
            .filter(MavenExport::fields().path().eq(path.clone()))
            .delete()
            .exec(&mut conn)
            .context(info_span!("invalidating Maven export", %path))
            .await?;
        drop(conn);
        exports.write().await.remove(&path);
        let status = app.maven.publish(coordinate, sha256, &app.cas).await?;
        let mut conn = self
            .db
            .connection()
            .context(info_span!("acquiring Maven export connection"))
            .await?;
        MavenExport::upsert_by_path(path.clone())
            .sha256(sha256)
            .exec(&mut conn)
            .context(info_span!("recording Maven export", %path, %sha256))
            .await?;
        exports.write().await.insert(path, sha256);
        Ok(status)
    }
}
