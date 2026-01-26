//! Test that valid uses of the API compile.
#![expect(dead_code, reason = "only here to check that the code compiles")]

use sqlx::Postgres;

async fn a(db: sqlx_tracing::Pool<Postgres>) {
    let _conn: sqlx_tracing::PoolConnection<Postgres> =
        db.acquire().await.unwrap();
}

async fn b<'a, E>(exec: E)
where
    E: sqlx_tracing::Acquire<'a, Database = Postgres>,
{
    let mut conn: sqlx_tracing::AnyConnection<Postgres> =
        exec.acquire().await.unwrap();
    // sqlx::query("SELECT 1").execute(&mut conn).await.unwrap();
    sqlx::query("SELECT 1").execute(&mut conn).await.unwrap();
}

async fn c<'a, E>(exec: E)
where
    E: sqlx_tracing::Executor<'a, Database = Postgres>,
{
    sqlx::query("SELECT 1").execute(exec).await.unwrap();
}

pub async fn list_many<'a, E>(exec: E)
where
    E: sqlx::Executor<'a, Database = Postgres>,
{
    sqlx::query(
        "
        SELECT
            id, enum_id, value, ordering,
            metadata, created
        FROM loader_field_enum_values
        WHERE enum_id = ANY($1)
        ORDER BY enum_id, ordering, created DESC
        ",
    )
    .fetch_all(exec)
    .await
    .unwrap();
}

async fn insert_sqlx(transaction: &mut sqlx::Transaction<'_, Postgres>) {
    get_id_sqlx(&mut *transaction).await;
}

async fn insert<'t, 'c>(
    transaction: &'t mut sqlx_tracing::Transaction<'c, Postgres>,
) {
    get_id(&mut *transaction).await;
    get_id(&mut *transaction).await;

    sqlx::query("SELECT 1")
        .execute(&mut *transaction)
        .await
        .unwrap();
    sqlx::query("SELECT 1")
        .execute(&mut *transaction)
        .await
        .unwrap();
}

async fn get_id_sqlx<'a, E>(_executor: E)
where
    E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
{
}

async fn get_id<'a, E>(_executor: E)
where
    E: sqlx_tracing::Acquire<'a, Database = sqlx::Postgres>,
{
}
