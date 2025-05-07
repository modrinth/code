use hyper_tls::{HttpsConnector, native_tls};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;

mod fetch;

pub use fetch::*;

pub async fn init_client() -> clickhouse::error::Result<clickhouse::Client> {
    init_client_with_database(&dotenvy::var("CLICKHOUSE_DATABASE").unwrap())
        .await
}

pub async fn init_client_with_database(
    database: &str,
) -> clickhouse::error::Result<clickhouse::Client> {
    let client = {
        let mut http_connector = HttpConnector::new();
        http_connector.enforce_http(false); // allow https URLs

        let tls_connector =
            native_tls::TlsConnector::builder().build().unwrap().into();
        let https_connector =
            HttpsConnector::from((http_connector, tls_connector));
        let hyper_client =
            hyper_util::client::legacy::Client::builder(TokioExecutor::new())
                .build(https_connector);

        clickhouse::Client::with_http_client(hyper_client)
            .with_url(dotenvy::var("CLICKHOUSE_URL").unwrap())
            .with_user(dotenvy::var("CLICKHOUSE_USER").unwrap())
            .with_password(dotenvy::var("CLICKHOUSE_PASSWORD").unwrap())
    };

    client
        .query(&format!("CREATE DATABASE IF NOT EXISTS {database}"))
        .execute()
        .await?;

    let clickhouse_replicated =
        dotenvy::var("CLICKHOUSE_REPLICATED").unwrap() == "true";
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
                headers Array(Tuple(String, String))
            )
            ENGINE = {engine}
            PRIMARY KEY (project_id, recorded, ip)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.downloads {cluster_line}
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
                headers Array(Tuple(String, String))
            )
            ENGINE = {engine}
            PRIMARY KEY (project_id, recorded, ip)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.playtime {cluster_line}
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
            PRIMARY KEY (project_id, recorded, user_id)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    Ok(client.with_database(database))
}
