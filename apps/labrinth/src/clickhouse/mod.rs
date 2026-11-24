use std::time::Duration;

use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::rt::TokioExecutor;

use crate::env::ENV;
use crate::queue::server_ping;

pub const DOWNLOADS: &str = "downloads";
pub const PLAYTIME: &str = "playtime";
pub const MINECRAFT_SERVER_PLAYS: &str = "minecraft_server_plays";
pub const APP_EVENTS: &str = "app_events";

pub async fn init_client() -> clickhouse::error::Result<clickhouse::Client> {
    init_client_with_database(&ENV.CLICKHOUSE_DATABASE).await
}

pub async fn init_client_with_database(
    database: &str,
) -> clickhouse::error::Result<clickhouse::Client> {
    Ok(connect()?.with_database(database))
}

fn connect() -> clickhouse::error::Result<clickhouse::Client> {
    let https_connector = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_all_versions()
        .build();
    let hyper_client =
        hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            // ClickHouse closes idle keep-alive connections after 3s by default.
            // Hyper's 90s default reuses dead sockets and fails with
            // "connection closed before message completed".
            .pool_idle_timeout(Duration::from_secs(2))
            .build(https_connector);

    Ok(clickhouse::Client::with_http_client(hyper_client)
        .with_url(&ENV.CLICKHOUSE_URL)
        .with_user(&ENV.CLICKHOUSE_USER)
        .with_password(&ENV.CLICKHOUSE_PASSWORD)
        .with_validation(false))
}

#[cfg(feature = "test")]
pub async fn create_database(database: &str) -> clickhouse::error::Result<()> {
    connect()?
        .query(&format!("CREATE DATABASE IF NOT EXISTS {database}"))
        .execute()
        .await
}

pub async fn run_migrations() -> clickhouse::error::Result<()> {
    run_migrations_on_database(&ENV.CLICKHOUSE_DATABASE).await
}

pub async fn run_migrations_on_database(
    database: &str,
) -> clickhouse::error::Result<()> {
    const MINECRAFT_JAVA_SERVER_PINGS: &str = server_ping::CLICKHOUSE_TABLE;

    let client = connect()?;

    let clickhouse_replicated = ENV.CLICKHOUSE_REPLICATED;
    let cluster_line = if clickhouse_replicated {
        "ON cluster '{cluster}'"
    } else {
        ""
    };

    let engine = if clickhouse_replicated {
        "ReplicatedMergeTree('/clickhouse/{installation}/{cluster}/tables/{shard}/{database}/{table}', '{replica}')"
    } else {
        "MergeTree()"
    };

    let replacing_engine = if clickhouse_replicated {
        "ReplicatedReplacingMergeTree('/clickhouse/{installation}/{cluster}/tables/{shard}/{database}/{table}', '{replica}', recorded_ts)"
    } else {
        "ReplacingMergeTree(recorded_ts)"
    };

    // For the Clickhouse database on the staging environment, set a TTL to avoid accumulating too much data
    let ttl = if database == "staging_analytics" {
        "TTL toDateTime(recorded) + INTERVAL 1 DAY"
    } else {
        ""
    };

    let app_events_ttl = if database == "staging_analytics" {
        "TTL toDateTime(recorded_ts) + INTERVAL 3 DAY"
    } else {
        "TTL toStartOfMonth(event_ts) + INTERVAL 25 MONTH"
    };

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.views {cluster_line}
            (
                recorded DateTime64(4),
                domain String,
                site_path String,

                user_id UInt64,
                project_id UInt64,
                monetized Bool DEFAULT True,

                ip IPv6,
                country String,
                user_agent String,
                headers Array(Tuple(String, String)) DEFAULT [] TTL toDateTime(recorded) + INTERVAL 60 DAY
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded, ip)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{DOWNLOADS} {cluster_line}
            (
                recorded DateTime64(4),
                domain String,
                site_path String,

                user_id UInt64,
                project_id UInt64,
                version_id UInt64,

                ip IPv6,
                country String,
                user_agent String,
                headers Array(Tuple(String, String)) DEFAULT [] TTL toDateTime(recorded) + INTERVAL 60 DAY
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded, ip)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{PLAYTIME} {cluster_line}
            (
                recorded DateTime64(4),
                seconds UInt64,

                user_id UInt64,
                project_id UInt64,
                version_id UInt64,

                loader String,
                game_version String,
                parent UInt64
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded, user_id)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{MINECRAFT_JAVA_SERVER_PINGS} {cluster_line}
            (
                recorded DateTime64(4),
                project_id UInt64,
                address String,
                online Bool,
                latency_ms Nullable(UInt32),
                description Nullable(String),
                version_name Nullable(String),
                version_protocol Nullable(Int32),
                players_online Nullable(Int32),
                players_max Nullable(Int32)
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{MINECRAFT_SERVER_PLAYS} {cluster_line}
            (
                recorded DateTime64(4),
                user_id UInt64,
                project_id UInt64,
                minecraft_uuid UUID
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    // Deduplication is eventual and only covers retries received on the same
    // UTC day, so queries should count `uniqExact(client_installation_id, event_id)`
    // or use `LIMIT 1 BY client_installation_id, event_id`.
    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{APP_EVENTS} {cluster_line}
            (
                event_id               UUID                          COMMENT 'Client-controlled event ID',
                client_installation_id UUID                          COMMENT 'Client-controlled installation ID',
                event_type             LowCardinality(String)        COMMENT 'Type of event, validated at ingest',
                event_ts               DateTime64(6, 'UTC')          COMMENT 'Event timestamp, corrected for client clock skew',
                client_ts              DateTime64(6, 'UTC')          COMMENT 'Informational, client-side event timestamp',
                recorded_ts            DateTime64(6, 'UTC')          COMMENT 'Informational, server receive timestamp',
                source                 String                        COMMENT 'Event source variant',
                app_version            String                        COMMENT 'App version',
                os                     LowCardinality(String)        COMMENT 'Client operating system, validated at ingest',
                arch                   LowCardinality(String)        COMMENT 'Client CPU architecture, validated at ingest',
                ip_country             FixedString(2)                COMMENT 'ISO 3166-1 alpha-2 client country, XX for unknown and T1 for Tor',
                metadata               JSON(max_dynamic_paths = 256) COMMENT 'Event metadata, schema validated at ingest'
            )
            ENGINE = {replacing_engine}
            PARTITION BY toYYYYMM(event_ts)
            ORDER BY (toDate(event_ts), event_type, cityHash64(client_installation_id), event_id)
            SAMPLE BY cityHash64(client_installation_id)
            {app_events_ttl}
            SETTINGS index_granularity = 8192, ttl_only_drop_parts = 1
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{MINECRAFT_SERVER_PLAYS} {cluster_line}
            ADD COLUMN IF NOT EXISTS minecraft_uuid UUID
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{MINECRAFT_SERVER_PLAYS} {cluster_line}
            ADD COLUMN IF NOT EXISTS ip IPv6 DEFAULT toIPv6('::')
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{MINECRAFT_JAVA_SERVER_PINGS} {cluster_line}
            DROP COLUMN IF EXISTS port
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{DOWNLOADS} {cluster_line}
            ADD COLUMN IF NOT EXISTS reason String,
            ADD COLUMN IF NOT EXISTS game_version String,
            ADD COLUMN IF NOT EXISTS loader String,
            ADD COLUMN IF NOT EXISTS dependent_on_version_id UInt64
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{PLAYTIME} {cluster_line}
            ADD COLUMN IF NOT EXISTS country String
            "
        ))
        .execute()
        .await?;

    // Modpack playtime is looked up by `parent`, which isn't in the primary
    // key. Existing parts need a one-off `MATERIALIZE PROJECTION` to use it.
    client
        .query(&format!(
            "
            ALTER TABLE {database}.{PLAYTIME} {cluster_line}
            ADD PROJECTION IF NOT EXISTS by_parent (
                SELECT recorded, parent, seconds, version_id, loader, game_version, country
                ORDER BY (parent, recorded)
            )
            "
        ))
        .execute()
        .await?;

    Ok(())
}
