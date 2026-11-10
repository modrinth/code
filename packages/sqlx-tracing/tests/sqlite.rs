#![cfg(feature = "sqlite")]

use sqlx::{Executor, Sqlite, Statement};

mod common;

#[tokio::test]
async fn preserves_owned_queries_prepared_statements_and_transactions() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    let pool = sqlx_tracing::Pool::from(pool);
    sqlx::query("create table entries (value integer not null)")
        .execute(&pool)
        .await
        .unwrap();

    let mut transaction = pool.begin().await.unwrap();
    let sql = String::from("insert into entries (value) values ($1)");
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(42_i32)
        .persistent(false)
        .execute(&mut transaction)
        .await
        .unwrap();

    let statement = (&mut transaction)
        .prepare(sqlx::SqlStr::from_static("select value from entries"))
        .await
        .unwrap();
    let value: i32 = statement
        .query_scalar()
        .fetch_one(&mut transaction)
        .await
        .unwrap();
    assert_eq!(value, 42);

    let mut nested = sqlx_tracing::Acquire::begin(&mut transaction)
        .await
        .unwrap();
    sqlx::query("insert into entries (value) values ($1)")
        .bind(99_i32)
        .execute(&mut nested)
        .await
        .unwrap();
    nested.rollback().await.unwrap();
    transaction.commit().await.unwrap();

    let mut connection = pool.acquire().await.unwrap();
    let count: i64 = sqlx::query_scalar("select count(*) from entries")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn execute() {
    let observability =
        opentelemetry_testing::ObservabilityContainer::create().await;
    let provider = observability.install().await;

    let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
    let pool = sqlx_tracing::Pool::from(pool);

    common::should_trace(
        "trace_pool",
        "sqlite",
        &observability,
        &provider,
        &pool,
    )
    .await;

    {
        let mut conn = pool.acquire().await.unwrap();
        common::should_trace(
            "trace_conn",
            "sqlite",
            &observability,
            &provider,
            &mut conn,
        )
        .await;
    }

    {
        let mut tx: sqlx_tracing::Transaction<'_, Sqlite> =
            pool.begin().await.unwrap();
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
