use eyre::Context;
use prometheus::{IntGauge, Registry};
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Connection, PgConnection, Postgres};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tracing::info;

#[derive(Clone)]
#[repr(transparent)]
pub struct ReadOnlyPgPool(PgPool);

impl From<PgPool> for ReadOnlyPgPool {
    fn from(pool: PgPool) -> Self {
        ReadOnlyPgPool(pool)
    }
}

impl Deref for ReadOnlyPgPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReadOnlyPgPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub async fn connect_all() -> Result<(PgPool, ReadOnlyPgPool), sqlx::Error> {
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

    if let Ok(url) = dotenvy::var("READONLY_DATABASE_URL") {
        let ro_pool = PgPoolOptions::new()
            .min_connections(
                dotenvy::var("READONLY_DATABASE_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(0),
            )
            .max_connections(
                dotenvy::var("READONLY_DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(1),
            )
            .max_lifetime(Some(Duration::from_secs(60 * 60)))
            .connect(&url)
            .await?;

        Ok((pool, ReadOnlyPgPool(ro_pool)))
    } else {
        let ro = ReadOnlyPgPool(pool.clone());
        Ok((pool, ro))
    }
}

pub async fn check_for_migrations() -> eyre::Result<()> {
    let uri =
        dotenvy::var("DATABASE_URL").wrap_err("`DATABASE_URL` not in .env")?;
    let uri = uri.as_str();
    if !Postgres::database_exists(uri)
        .await
        .wrap_err("failed to check if database exists")?
    {
        info!("Creating database...");
        Postgres::create_database(uri)
            .await
            .wrap_err("failed to create database")?;
    }

    info!("Applying migrations...");

    let mut conn: PgConnection = PgConnection::connect(uri)
        .await
        .wrap_err("failed to connect to database")?;
    MIGRATOR
        .run(&mut conn)
        .await
        .wrap_err("failed to run database migrations")?;

    Ok(())
}

pub static MIGRATOR: Migrator = sqlx::migrate!();

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
