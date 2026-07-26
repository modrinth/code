use crate::database;
use crate::database::PgPool;
use crate::database::models::ids::DBUserId;
use crate::database::models::notification_item::NotificationBuilder;
use crate::file_hosting::FileHost;
use crate::models::notifications::NotificationBody;
use crate::queue::analytics::cache::cache_analytics;
use crate::queue::billing::{index_billing, index_subscriptions};
use crate::queue::email::EmailQueue;
use crate::queue::file_scan::scan_all_pending_files;
use crate::queue::payouts::{
    PayoutsQueue, index_payouts_notifications,
    insert_bank_balances_and_webhook, process_affiliate_payouts,
    process_payout, remove_payouts_for_refunded_charges,
};
use crate::search::SearchBackend;
use crate::util::anrok;
use actix_web::web;
use clap::ValueEnum;
use eyre::WrapErr;
use tracing::info;
use xredis::RedisPool;

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
#[clap(rename_all = "kebab_case")]
pub enum BackgroundTask {
    IndexSearch,
    ReleaseScheduled,
    UpdateVersions,
    Payouts,
    SyncPayoutStatuses,
    IndexBilling,
    IndexSubscriptions,
    IncrementalIndexSearch,
    Migrations,
    Mail,
    /// Queries server project analytics (e.g. number of verified plays in last
    /// 2 weeks for server projects) and caches them in Redis.
    CacheAnalytics,
    /// Attempts to ping Minecraft Java servers as if we were a client, to
    /// collect info on if they're online, game version, description, etc.
    PingMinecraftJavaServers,
    /// Finds files of versions which have not been scanned for attributions
    /// yet, extracts them to find file overrides, and finds any overrides which
    /// require attribution from the creator.
    ScanPendingFiles,
    /// Queues Discord Creator Club role claim emails for newly eligible users.
    DiscordRoleEmailCampaign,
}

impl BackgroundTask {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        self,
        pool: PgPool,
        ro_pool: PgPool,
        redis_pool: RedisPool,
        search_backend: web::Data<dyn SearchBackend>,
        file_host: web::Data<dyn FileHost>,
        kafka_client: web::Data<crate::util::kafka::KafkaClientState>,
        clickhouse: clickhouse::Client,
        stripe_client: stripe::Client,
        anrok_client: anrok::Client,
        email_queue: EmailQueue,
        mural_client: muralpay::Client,
    ) -> eyre::Result<()> {
        use BackgroundTask::*;
        match self {
            Migrations => run_migrations().await,
            IndexSearch => {
                index_search(ro_pool, redis_pool, search_backend).await
            }
            ReleaseScheduled => release_scheduled(pool).await,
            UpdateVersions => update_versions(pool, redis_pool).await,
            Payouts => payouts(pool, clickhouse, redis_pool).await,
            SyncPayoutStatuses => {
                sync_payout_statuses(pool, mural_client).await
            }
            IndexBilling => {
                index_billing(
                    stripe_client,
                    anrok_client,
                    pool.clone(),
                    redis_pool,
                )
                .await;

                update_bank_balances(pool).await
            }
            IndexSubscriptions => {
                index_subscriptions(
                    pool,
                    redis_pool,
                    stripe_client,
                    anrok_client,
                )
                .await;
                Ok(())
            }
            IncrementalIndexSearch => {
                crate::search::incremental::consume::run(
                    ro_pool,
                    redis_pool,
                    search_backend,
                    kafka_client,
                )
                .await
            }
            Mail => run_email(email_queue).await,
            CacheAnalytics => {
                cache_analytics(&pool, &redis_pool, &clickhouse).await
            }
            PingMinecraftJavaServers => {
                ping_minecraft_java_servers(
                    pool,
                    redis_pool,
                    clickhouse,
                    kafka_client,
                )
                .await
            }
            ScanPendingFiles => {
                scan_all_pending_files(
                    &pool,
                    &redis_pool,
                    file_host.into_inner(),
                )
                .await
            }
            DiscordRoleEmailCampaign => {
                discord_role_email_campaign(pool, redis_pool).await
            }
        }
    }
}

pub async fn run_email(email_queue: EmailQueue) -> eyre::Result<()> {
    // Only index for 5 emails at a time, to reduce transaction length,
    // for a total of 100 emails.
    for _ in 0..20 {
        let then = std::time::Instant::now();

        let indexed = email_queue
            .index(5)
            .await
            .wrap_err("failed to index email queue")?;
        if indexed {
            info!("Indexed email queue in {}ms", then.elapsed().as_millis());
        } else {
            info!("No more emails to index");
            break;
        }
    }

    Ok(())
}

pub async fn update_bank_balances(pool: PgPool) -> eyre::Result<()> {
    let payouts_queue = PayoutsQueue::new();

    insert_bank_balances_and_webhook(&payouts_queue, &pool)
        .await
        .wrap_err("failed to update bank balances")?;
    info!("Bank balances updated successfully");
    Ok(())
}

pub async fn run_migrations() -> eyre::Result<()> {
    database::check_for_migrations().await?;
    Ok(())
}

pub async fn index_search(
    ro_pool: PgPool,
    redis_pool: RedisPool,
    search_backend: web::Data<dyn SearchBackend>,
) -> eyre::Result<()> {
    info!("Indexing local database");
    search_backend.index_projects(ro_pool, redis_pool).await
}

pub async fn release_scheduled(pool: PgPool) -> eyre::Result<()> {
    info!("Releasing scheduled versions/projects!");

    sqlx::query!(
        "
        UPDATE mods
        SET status = requested_status
        WHERE status = $1 AND approved < CURRENT_DATE AND requested_status IS NOT NULL
        ",
        crate::models::projects::ProjectStatus::Scheduled.as_str(),
    )
    .execute(&pool)
    .await
    .wrap_err("failed syncing scheduled releases for projects")?;

    sqlx::query!(
        "
        UPDATE versions
        SET status = requested_status
        WHERE status = $1 AND date_published < CURRENT_DATE AND requested_status IS NOT NULL
        ",
        crate::models::projects::VersionStatus::Scheduled.as_str(),
    )
    .execute(&pool)
    .await
    .wrap_err("failed syncing scheduled releases for versions")?;

    info!("Finished releasing scheduled versions/projects");
    Ok(())
}

pub async fn update_versions(
    pool: PgPool,
    redis_pool: RedisPool,
) -> eyre::Result<()> {
    info!("Indexing game versions list from Mojang");
    version_updater::update_versions(&pool, &redis_pool)
        .await
        .wrap_err("failed to update game versions")?;
    info!("Done indexing game versions");
    Ok(())
}

pub async fn payouts(
    pool: PgPool,
    clickhouse: clickhouse::Client,
    redis_pool: RedisPool,
) -> eyre::Result<()> {
    info!("Started running payouts");
    process_payout(&pool, &clickhouse)
        .await
        .wrap_err("payout processing failed")?;

    index_payouts_notifications(&pool, &redis_pool)
        .await
        .wrap_err("payout notifications indexing failed")?;

    process_affiliate_payouts(&pool)
        .await
        .wrap_err("affiliate payouts processing failed")?;

    remove_payouts_for_refunded_charges(&pool)
        .await
        .wrap_err("removing payouts for refunded charges failed")?;

    info!("Done running payouts");
    Ok(())
}

pub async fn discord_role_email_campaign(
    pool: PgPool,
    redis_pool: RedisPool,
) -> eyre::Result<()> {
    info!("Started indexing Discord role email campaign");

    let mut txn = pool
        .begin()
        .await
        .wrap_err("failed to begin Discord role email campaign transaction")?;

    let lock_acquired = sqlx::query_scalar!(
        r#"SELECT pg_try_advisory_xact_lock(hashtextextended('discord_role_email_campaign', 0)) AS "lock_acquired!""#,
    )
    .fetch_one(&mut txn)
    .await
    .wrap_err("failed to acquire Discord role email campaign lock")?;

    if !lock_acquired {
        info!("Discord role email campaign is already running");
        return Ok(());
    }

    let user_ids = sqlx::query_scalar!(
        r#"
        WITH
          user_project_downloads AS (
            SELECT
              tm.user_id,
              SUM(m.downloads)::BIGINT total_downloads
            FROM team_members tm
            INNER JOIN mods m ON m.team_id = tm.team_id
            WHERE tm.accepted = TRUE
            GROUP BY tm.user_id
          )
        SELECT u.id AS "id!"
        FROM users u
        INNER JOIN user_project_downloads upd ON upd.user_id = u.id
        WHERE u.email IS NOT NULL
          AND u.email_verified = TRUE
          AND upd.total_downloads > 20000
          AND NOT EXISTS (
            SELECT 1
            FROM notifications n
            WHERE n.user_id = u.id
              AND n.body ->> 'type' = 'discord_role_creator_club'
          )
        ORDER BY upd.total_downloads DESC, u.id
        LIMIT 1000
        "#,
    )
    .fetch_all(&mut txn)
    .await
    .wrap_err("failed to fetch Discord role email campaign recipients")?
    .into_iter()
    .map(DBUserId)
    .collect::<Vec<_>>();

    let count = user_ids.len();

    if !user_ids.is_empty() {
        NotificationBuilder {
            body: NotificationBody::DiscordRoleCreatorClub,
        }
        .insert_many(user_ids, &mut txn, &redis_pool)
        .await
        .wrap_err("failed to queue Discord role email notifications")?;
    }

    txn.commit()
        .await
        .wrap_err("failed to commit Discord role email campaign transaction")?;

    info!(count, "Finished indexing Discord role email campaign");
    Ok(())
}

pub async fn sync_payout_statuses(
    pool: PgPool,
    mural: muralpay::Client,
) -> eyre::Result<()> {
    // Mural sets a max limit of 100 for search payouts endpoint
    const LIMIT: u32 = 100;

    info!("Started syncing payout statuses");

    crate::queue::payouts::mural::sync_pending_payouts_from_mural(
        &pool, &mural, LIMIT,
    )
    .await
    .wrap_err("failed to sync pending payouts from Mural")?;

    crate::queue::payouts::mural::sync_failed_mural_payouts_to_labrinth(
        &pool, &mural, LIMIT,
    )
    .await
    .wrap_err("failed to sync failed Mural payouts to Labrinth")?;

    info!("Done syncing payout statuses");
    Ok(())
}

pub async fn ping_minecraft_java_servers(
    pool: PgPool,
    redis_pool: RedisPool,
    clickhouse: clickhouse::Client,
    kafka_client: web::Data<crate::util::kafka::KafkaClientState>,
) -> eyre::Result<()> {
    info!("Started pinging Minecraft Java servers");

    let incremental_search_queue =
        crate::search::incremental::IncrementalSearchQueue::new(kafka_client);
    let server_ping_queue = crate::queue::server_ping::ServerPingQueue::new(
        pool,
        redis_pool,
        clickhouse,
        incremental_search_queue.clone(),
    );

    server_ping_queue
        .ping_minecraft_java_servers()
        .await
        .wrap_err("failed to ping Minecraft Java servers")?;
    incremental_search_queue
        .drain()
        .await
        .wrap_err("failed to drain incremental search queue")?;
    info!("Successfully pinged Minecraft Java servers");

    info!("Done pinging Minecraft Java servers");
    Ok(())
}

mod version_updater {
    use std::sync::LazyLock;

    use crate::database::PgPool;
    use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use thiserror::Error;
    use tracing::warn;
    use xredis::RedisPool;

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
        pool: &PgPool,
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
