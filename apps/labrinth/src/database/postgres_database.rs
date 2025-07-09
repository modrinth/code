use prometheus::{IntGauge, Registry};
use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Connection, PgConnection, Postgres};
use std::time::Duration;
use tracing::info;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    info!("Initializing database connection");
    let database_url =
        dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    let pool = PgPoolOptions::new()
        .min_connections(
            dotenvy::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(0),
        )
        .max_connections(
            dotenvy::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(16),
        )
        .max_lifetime(Some(Duration::from_secs(60 * 60)))
        .connect(&database_url)
        .await?;

    Ok(pool)
}
pub async fn check_for_migrations() -> Result<(), sqlx::Error> {
    let uri = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    let uri = uri.as_str();
    if !Postgres::database_exists(uri).await? {
        info!("Creating database...");
        Postgres::create_database(uri).await?;
    }

    info!("Applying migrations...");

    let mut conn: PgConnection = PgConnection::connect(uri).await?;
    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("Error while running database migrations!");

    Ok(())
}

pub async fn register_and_set_metrics(
    pool: &PgPool,
    registry: &Registry,
) -> Result<(), prometheus::Error> {
    let pg_pool_size =
        IntGauge::new("labrinth_pg_pool_size", "Size of Postgres pool")?;
    let pg_pool_idle = IntGauge::new(
        "labrinth_pg_pool_idle",
        "Number of idle Postgres connections",
    )?;

    registry.register(Box::new(pg_pool_size.clone()))?;
    registry.register(Box::new(pg_pool_idle.clone()))?;

    let pool_ref = pool.clone();
    tokio::spawn(async move {
        loop {
            pg_pool_size.set(pool_ref.size() as i64);
            pg_pool_idle.set(pool_ref.num_idle() as i64);

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}
