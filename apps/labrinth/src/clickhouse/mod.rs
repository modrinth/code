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
        let tls_connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("no native root CA certificates found")
            .https_only()
            .enable_http1()
            .build();
        let hyper_client = hyper_util::client::legacy::Client::builder(
            hyper_util::rt::TokioExecutor::new(),
        )
        .build(tls_connector);

        clickhouse::Client::with_http_client(hyper_client)
            .with_url(dotenvy::var("CLICKHOUSE_URL").unwrap())
            .with_user(dotenvy::var("CLICKHOUSE_USER").unwrap())
            .with_password(dotenvy::var("CLICKHOUSE_PASSWORD").unwrap())
    };

    client
        .query(&format!("CREATE DATABASE IF NOT EXISTS {database}"))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.views
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
            ENGINE = MergeTree()
            PRIMARY KEY (project_id, recorded, ip)
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.downloads
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
            ENGINE = MergeTree()
            PRIMARY KEY (project_id, recorded, ip)
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.playtime
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
            ENGINE = MergeTree()
            PRIMARY KEY (project_id, recorded, user_id)
            "
        ))
        .execute()
        .await?;

    Ok(client.with_database(database))
}
