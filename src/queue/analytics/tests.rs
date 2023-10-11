use futures::Future;
use uuid::Uuid;

use super::*;
use crate::clickhouse::init_client_with_database;
use std::net::Ipv6Addr;

#[tokio::test]
async fn test_indexing() {
    with_test_clickhouse_db(|clickhouse_client| async move {
        let analytics = AnalyticsQueue::new();

        analytics.add_download(get_default_download());
        analytics.add_playtime(get_default_playtime());
        analytics.add_view(get_default_views());

        analytics.index(clickhouse_client.clone()).await.unwrap();
        assert_table_counts(&clickhouse_client, 1, 1, 1).await;

        analytics.index(clickhouse_client.clone()).await.unwrap();
        assert_table_counts(&clickhouse_client, 1, 1, 1).await;
    })
    .await;
}

#[tokio::test]
async fn can_insert_many_downloads() {
    with_test_clickhouse_db(|clickhouse_client| async move {
        let analytics = AnalyticsQueue::new();
        let n_downloads = 100_000;

        for _ in 0..n_downloads {
            analytics.add_download(get_default_download());
        }

        analytics.index(clickhouse_client.clone()).await.unwrap();
        assert_table_count(DOWNLOADS_TABLENAME, &clickhouse_client, n_downloads).await;
    })
    .await;
}

async fn assert_table_counts(
    client: &clickhouse::Client,
    downloads: u64,
    playtimes: u64,
    views: u64,
) {
    assert_table_count(DOWNLOADS_TABLENAME, client, downloads).await;
    assert_table_count(PLAYTIME_TABLENAME, client, playtimes).await;
    assert_table_count(VIEWS_TABLENAME, client, views).await;
}

async fn assert_table_count(table_name: &str, client: &clickhouse::Client, expected_count: u64) {
    let count = client
        .query(&format!("SELECT COUNT(*) from {table_name}"))
        .fetch_one::<u64>()
        .await
        .unwrap();
    assert_eq!(expected_count, count);
}

async fn with_test_clickhouse_db<Fut>(f: impl FnOnce(clickhouse::Client) -> Fut)
where
    Fut: Future<Output = ()>,
{
    let db_name = format!("test_{}", uuid::Uuid::new_v4().as_simple());
    println!("Clickhouse test db: {}", db_name);
    let clickhouse_client = init_client_with_database(&db_name)
        .await
        .expect("A real clickhouse instance should be running locally");

    f(clickhouse_client.clone()).await;

    clickhouse_client
        .query(&format!("DROP DATABASE IF EXISTS {db_name}"))
        .execute()
        .await
        .unwrap();
}

fn get_default_download() -> Download {
    Download {
        id: Uuid::new_v4(),
        recorded: Default::default(),
        domain: Default::default(),
        site_path: Default::default(),
        user_id: Default::default(),
        project_id: Default::default(),
        version_id: Default::default(),
        ip: get_default_ipv6(),
        country: Default::default(),
        user_agent: Default::default(),
        headers: Default::default(),
    }
}

fn get_default_playtime() -> Playtime {
    Playtime {
        id: Uuid::new_v4(),
        recorded: Default::default(),
        seconds: Default::default(),
        user_id: Default::default(),
        project_id: Default::default(),
        version_id: Default::default(),
        loader: Default::default(),
        game_version: Default::default(),
        parent: Default::default(),
    }
}

fn get_default_views() -> PageView {
    PageView {
        id: Uuid::new_v4(),
        recorded: Default::default(),
        domain: Default::default(),
        site_path: Default::default(),
        user_id: Default::default(),
        project_id: Default::default(),
        ip: get_default_ipv6(),
        country: Default::default(),
        user_agent: Default::default(),
        headers: Default::default(),
    }
}

fn get_default_ipv6() -> Ipv6Addr {
    Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
}
