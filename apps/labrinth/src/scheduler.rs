use actix_rt::Arbiter;
use futures::StreamExt;

pub struct Scheduler {
    arbiter: Arbiter,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
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
        let future = IntervalStream::new(actix_rt::time::interval(interval))
            .for_each_concurrent(2, move |_| task());

        self.arbiter.spawn(future);
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
    redis: RedisPool,
) {
    let version_index_interval = std::time::Duration::from_secs(
        parse_var("VERSION_INDEX_INTERVAL").unwrap_or(1800),
    );

    scheduler.run(version_index_interval, move || {
        let pool_ref = pool.clone();
        let redis = redis.clone();
        async move {
            info!("Indexing game versions list from Mojang");
            let result = update_versions(&pool_ref, &redis).await;
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

use crate::{
    database::{
        models::legacy_loader_fields::MinecraftGameVersion, redis::RedisPool,
    },
    util::env::parse_var,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tokio_stream::wrappers::IntervalStream;

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
    #[serde(rename = "releaseTime")]
    release_time: DateTime<Utc>,
}

async fn update_versions(
    pool: &sqlx::Pool<sqlx::Postgres>,
    redis: &RedisPool,
) -> Result<(), VersionIndexingError> {
    let input = reqwest::get(
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
    )
    .await?
    .json::<InputFormat>()
    .await?;

    let mut skipped_versions_count = 0u32;

    // A list of version names that contains spaces.
    // Generated using the command
    // ```sh
    // curl https://launchermeta.mojang.com/mc/game/version_manifest.json \
    //      | jq '[.versions[].id | select(contains(" "))]'
    // ```
    const HALL_OF_SHAME: [(&str, &str); 12] = [
        ("1.14.2 Pre-Release 4", "1.14.2-pre4"),
        ("1.14.2 Pre-Release 3", "1.14.2-pre3"),
        ("1.14.2 Pre-Release 2", "1.14.2-pre2"),
        ("1.14.2 Pre-Release 1", "1.14.2-pre1"),
        ("1.14.1 Pre-Release 2", "1.14.1-pre2"),
        ("1.14.1 Pre-Release 1", "1.14.1-pre1"),
        ("1.14 Pre-Release 5", "1.14-pre5"),
        ("1.14 Pre-Release 4", "1.14-pre4"),
        ("1.14 Pre-Release 3", "1.14-pre3"),
        ("1.14 Pre-Release 2", "1.14-pre2"),
        ("1.14 Pre-Release 1", "1.14-pre1"),
        ("3D Shareware v1.34", "3D-Shareware-v1.34"),
    ];

    lazy_static::lazy_static! {
        /// Mojank for some reason has versions released at the same DateTime. This hardcodes them to fix this,
        /// as most of our ordering logic is with DateTime
        static ref HALL_OF_SHAME_2: [(&'static str, chrono::DateTime<chrono::Utc>); 4] = [
            (
                "1.4.5",
                chrono::DateTime::parse_from_rfc3339("2012-12-19T22:00:00+00:00")
                    .unwrap()
                    .into(),
            ),
            (
                "1.4.6",
                chrono::DateTime::parse_from_rfc3339("2012-12-19T22:00:01+00:00")
                    .unwrap()
                    .into(),
            ),
            (
                "1.6.3",
                chrono::DateTime::parse_from_rfc3339("2013-09-13T10:54:41+00:00")
                    .unwrap()
                    .into(),
            ),
            (
                "13w37b",
                chrono::DateTime::parse_from_rfc3339("2013-09-13T10:54:42+00:00")
                    .unwrap()
                    .into(),
            ),
        ];
    }

    for version in input.versions.into_iter() {
        let mut name = version.id;
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            if let Some((_, alternate)) =
                HALL_OF_SHAME.iter().find(|(version, _)| name == *version)
            {
                name = String::from(*alternate);
            } else {
                // We'll deal with these manually
                skipped_versions_count += 1;
                continue;
            }
        }

        let type_ = match &*version.type_ {
            "release" => "release",
            "snapshot" => "snapshot",
            "old_alpha" => "alpha",
            "old_beta" => "beta",
            _ => "other",
        };

        MinecraftGameVersion::builder()
            .version(&name)?
            .version_type(type_)?
            .created(
                if let Some((_, alternate)) =
                    HALL_OF_SHAME_2.iter().find(|(version, _)| name == *version)
                {
                    alternate
                } else {
                    &version.release_time
                },
            )
            .insert(pool, redis)
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
