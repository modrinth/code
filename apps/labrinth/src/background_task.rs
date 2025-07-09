use crate::database::redis::RedisPool;
use crate::queue::payouts::process_payout;
use crate::search::indexing::index_projects;
use crate::{database, search};
use clap::ValueEnum;
use sqlx::Postgres;
use tracing::{info, warn};

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
#[clap(rename_all = "kebab_case")]
pub enum BackgroundTask {
    IndexSearch,
    ReleaseScheduled,
    UpdateVersions,
    Payouts,
    IndexBilling,
    IndexSubscriptions,
    Migrations,
}

impl BackgroundTask {
    pub async fn run(
        self,
        pool: sqlx::Pool<Postgres>,
        redis_pool: RedisPool,
        search_config: search::SearchConfig,
        clickhouse: clickhouse::Client,
        stripe_client: stripe::Client,
    ) {
        use BackgroundTask::*;
        match self {
            Migrations => run_migrations().await,
            IndexSearch => index_search(pool, redis_pool, search_config).await,
            ReleaseScheduled => release_scheduled(pool).await,
            UpdateVersions => update_versions(pool, redis_pool).await,
            Payouts => payouts(pool, clickhouse).await,
            IndexBilling => {
                crate::routes::internal::billing::index_billing(
                    stripe_client,
                    pool,
                    redis_pool,
                )
                .await
            }
            IndexSubscriptions => {
                crate::routes::internal::billing::index_subscriptions(
                    pool, redis_pool,
                )
                .await
            }
        }
    }
}

pub async fn run_migrations() {
    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");
}

pub async fn index_search(
    pool: sqlx::Pool<Postgres>,
    redis_pool: RedisPool,
    search_config: search::SearchConfig,
) {
    info!("Indexing local database");
    let result = index_projects(pool, redis_pool, &search_config).await;
    if let Err(e) = result {
        warn!("Local project indexing failed: {:?}", e);
    }
    info!("Done indexing local database");
}

pub async fn release_scheduled(pool: sqlx::Pool<Postgres>) {
    info!("Releasing scheduled versions/projects!");

    let projects_results = sqlx::query!(
        "
        UPDATE mods
        SET status = requested_status
        WHERE status = $1 AND approved < CURRENT_DATE AND requested_status IS NOT NULL
        ",
        crate::models::projects::ProjectStatus::Scheduled.as_str(),
    )
        .execute(&pool)
        .await;

    if let Err(e) = projects_results {
        warn!("Syncing scheduled releases for projects failed: {:?}", e);
    }

    let versions_results = sqlx::query!(
        "
        UPDATE versions
        SET status = requested_status
        WHERE status = $1 AND date_published < CURRENT_DATE AND requested_status IS NOT NULL
        ",
        crate::models::projects::VersionStatus::Scheduled.as_str(),
    )
        .execute(&pool)
        .await;

    if let Err(e) = versions_results {
        warn!("Syncing scheduled releases for versions failed: {:?}", e);
    }

    info!("Finished releasing scheduled versions/projects");
}

pub async fn update_versions(
    pool: sqlx::Pool<Postgres>,
    redis_pool: RedisPool,
) {
    info!("Indexing game versions list from Mojang");
    let result = version_updater::update_versions(&pool, &redis_pool).await;
    if let Err(e) = result {
        warn!("Version update failed: {}", e);
    }
    info!("Done indexing game versions");
}

pub async fn payouts(
    pool: sqlx::Pool<Postgres>,
    clickhouse: clickhouse::Client,
) {
    info!("Started running payouts");
    let result = process_payout(&pool, &clickhouse).await;
    if let Err(e) = result {
        warn!("Payouts run failed: {:?}", e);
    }
    info!("Done running payouts");
}

mod version_updater {
    use std::sync::LazyLock;

    use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
    use crate::database::redis::RedisPool;
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use sqlx::Postgres;
    use thiserror::Error;
    use tracing::warn;

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

    #[derive(Error, Debug)]
    pub enum VersionIndexingError {
        #[error("Network error while updating game versions list: {0}")]
        NetworkError(#[from] reqwest::Error),
        #[error("Database error while updating game versions list: {0}")]
        DatabaseError(#[from] crate::database::models::DatabaseError),
    }

    pub async fn update_versions(
        pool: &sqlx::Pool<Postgres>,
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

        /// Mojank for some reason has versions released at the same DateTime. This hardcodes them to fix this,
        /// as most of our ordering logic is with DateTime
        static HALL_OF_SHAME_2: LazyLock<[(&'static str, DateTime<Utc>); 4]> =
            LazyLock::new(|| {
                [
                    (
                        "1.4.5",
                        chrono::DateTime::parse_from_rfc3339(
                            "2012-12-19T22:00:00+00:00",
                        )
                        .unwrap()
                        .into(),
                    ),
                    (
                        "1.4.6",
                        chrono::DateTime::parse_from_rfc3339(
                            "2012-12-19T22:00:01+00:00",
                        )
                        .unwrap()
                        .into(),
                    ),
                    (
                        "1.6.3",
                        chrono::DateTime::parse_from_rfc3339(
                            "2013-09-13T10:54:41+00:00",
                        )
                        .unwrap()
                        .into(),
                    ),
                    (
                        "13w37b",
                        chrono::DateTime::parse_from_rfc3339(
                            "2013-09-13T10:54:42+00:00",
                        )
                        .unwrap()
                        .into(),
                    ),
                ]
            });

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
                    if let Some((_, alternate)) = HALL_OF_SHAME_2
                        .iter()
                        .find(|(version, _)| name == *version)
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
}
