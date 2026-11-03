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
use crate::search::SearchBackend;
use crate::util::anrok;
use actix_web::web;
use clap::ValueEnum;
use eyre::WrapErr;
use tracing::{info, instrument};
use xredis::RedisPool;

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
#[clap(rename_all = "kebab_case")]
pub enum BackgroundTask {
    IndexSearch,
    ReleaseScheduled,
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
    #[instrument(skip_all, fields(background_task = ?self))]
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
    ) -> eyre::Result<()> {
        use BackgroundTask::*;
        match self {
            Migrations => run_migrations().await,
            IndexSearch => {
                index_search(ro_pool, redis_pool, search_backend).await
            }
            ReleaseScheduled => release_scheduled(pool).await,
            IndexBilling => {
                index_billing(stripe_client, anrok_client, pool, redis_pool)
                    .await;
                Ok(())
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

pub async fn run_migrations() -> eyre::Result<()> {
    database::check_for_migrations().await?;
    crate::clickhouse::run_migrations()
        .await
        .wrap_err("failed to run ClickHouse migrations")?;
    Ok(())
}

pub async fn index_search(
    ro_pool: PgPool,
    redis_pool: RedisPool,
    search_backend: web::Data<dyn SearchBackend>,
) -> eyre::Result<()> {
    info!("Indexing local database");
    search_backend.rebuild_index(ro_pool, redis_pool).await
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

pub async fn discord_role_email_campaign(
    pool: PgPool,
    redis_pool: RedisPool,
) -> eyre::Result<()> {
    info!("Started indexing Discord role email campaign");

    let mut txn = pool
        .begin()
        .await
        .wrap_err("failed to begin Discord role email campaign transaction")?;

    let lock_acquired =
        crate::database::advisory_lock::AdvisoryLock::DiscordRoleEmailCampaign
            .try_acquire(&mut txn)
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
