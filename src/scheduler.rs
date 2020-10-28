use actix_rt::time;
use actix_rt::Arbiter;
use futures::StreamExt;

pub struct Scheduler {
    arbiter: Arbiter,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            arbiter: Arbiter::new(),
        }
    }

    pub fn run<F, R>(&mut self, interval: std::time::Duration, mut task: F)
    where
        F: FnMut() -> R + Send + 'static,
        R: std::future::Future<Output = ()> + Send + 'static,
    {
        let future = time::interval(interval).for_each_concurrent(2, move |_| task());
        self.arbiter.send(future);
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.arbiter.stop();
    }
}

use log::{info, warn};

pub fn schedule_versions(
    scheduler: &mut Scheduler,
    pool: sqlx::Pool<sqlx::Postgres>,
    skip_initial: bool,
) {
    // Check mojang's versions every 6 hours
    let version_index_interval = std::time::Duration::from_secs(60 * 60 * 6);

    let mut skip = skip_initial;
    scheduler.run(version_index_interval, move || {
        let pool_ref = pool.clone();
        let local_skip = skip;
        if skip {
            skip = false;
        }
        async move {
            if local_skip {
                return;
            }
            info!("Indexing game versions list from Mojang");
            let result = update_versions(&pool_ref).await;
            if let Err(e) = result {
                warn!("Version update failed: {}", e);
            }
            info!("Done indexing game versions");
        }
    });
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VersionIndexingError {
    #[error("Network error while updating game versions list: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Database error while updating game versions list: {0}")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
}

use serde::Deserialize;

#[derive(Deserialize)]
struct InputFormat<'a> {
    // latest: LatestFormat,
    versions: Vec<VersionFormat<'a>>,
}
#[derive(Deserialize)]
struct VersionFormat<'a> {
    id: String,
    #[serde(rename = "type")]
    type_: std::borrow::Cow<'a, str>,
}

async fn update_versions(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), VersionIndexingError> {
    let input = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await?
        .json::<InputFormat>()
        .await?;

    let mut skipped_versions_count = 0u32;

    for version in input.versions.into_iter() {
        let name = version.id;
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            // We'll deal with these manually
            skipped_versions_count += 1;
            continue;
        }

        let type_ = match &*version.type_ {
            "release" => "release",
            "snapshot" => "snapshot",
            "old_alpha" => "alpha",
            "old_beta" => "beta",
            _ => "other",
        };

        crate::database::models::categories::GameVersion::builder()
            .version(&name)?
            .version_type(type_)?
            .insert(pool)
            .await?;
    }

    if skipped_versions_count > 0 {
        // This will currently always trigger due to 1.14 pre releases
        // and the shareware april fools update. We could set a threshold
        // that accounts for those versions and update it whenever we
        // manually fix another version.
        warn!(
            "Skipped {} game versions; check for new versions and add them manually",
            skipped_versions_count
        );
    }

    Ok(())
}
