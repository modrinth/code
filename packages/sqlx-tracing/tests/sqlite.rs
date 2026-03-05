#![cfg(feature = "sqlite")]

use sqlx::Sqlite;

mod common;

#[tokio::test]
async fn execute() {
    let observability = opentelemetry_testing::ObservabilityContainer::create().await;
    let provider = observability.install().await;

    let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
    let pool = sqlx_tracing::Pool::from(pool);

    common::should_trace("trace_pool", "sqlite", &observability, &provider, &pool).await;

    {
        let mut conn = pool.acquire().await.unwrap();
        common::should_trace("trace_conn", "sqlite", &observability, &provider, &mut conn).await;
    }

    {
        let mut tx: sqlx_tracing::Transaction<'_, Sqlite> = pool.begin().await.unwrap();
        common::should_trace(
            "trace_tx",
            "sqlite",
            &observability,
            &provider,
            &mut tx.executor(),
        )
        .await;
    }
}
