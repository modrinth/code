use log::info;
use sqlx::migrate::{Migrate, MigrateDatabase, Migrator};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Connection, PgConnection, Postgres};
use std::path::Path;

const MIGRATION_FOLDER: &str = "migrations";

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    info!("Initializing database connection");

    let database_url = dotenv::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    let pool = PgPoolOptions::new()
        .min_connections(
            dotenv::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .map(|x| x.parse::<u32>().ok())
                .flatten()
                .unwrap_or(16),
        )
        .max_connections(
            dotenv::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .map(|x| x.parse::<u32>().ok())
                .flatten()
                .unwrap_or(16),
        )
        .connect(&database_url)
        .await?;

    Ok(pool)
}
pub async fn check_for_migrations() -> Result<(), sqlx::Error> {
    let uri = &*dotenv::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    if !Postgres::database_exists(uri).await? {
        info!("Creating database...");
        Postgres::create_database(uri).await?;
    }
    info!("Applying migrations...");
    run_migrations(uri).await?;

    Ok(())
}

pub async fn run_migrations(uri: &str) -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(Path::new(MIGRATION_FOLDER)).await?;
    let mut conn: PgConnection = PgConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let (version, dirty) = conn.version().await?.unwrap_or((0, false));

    if dirty {
        panic!("The database is dirty ! Please check your database status.");
    }

    for migration in migrator.iter() {
        if migration.version > version {
            let _elapsed = conn.apply(migration).await?;
        } else {
            conn.validate(migration).await?;
        }
    }

    Ok(())
}
