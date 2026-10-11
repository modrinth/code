pub mod installation_repo;
pub mod instance_repo;
pub mod java_repo;

use std::{path::Path, time::Duration};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};

/// Connect to (or create) the SQLite database at the given path.
/// Uses WAL journal mode and a 5-second busy timeout to reduce contention.
pub async fn connect(path: &Path) -> color_eyre::eyre::Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(options).await?;
    Ok(pool)
}

/// Apply Core's schema.
///
/// A database created before the migrations were squashed still lists the old
/// history, which sqlx would refuse. Forget that history so `001_init` adopts
/// the existing tables; server data is left as it is.
pub async fn migrate(pool: &SqlitePool) -> color_eyre::eyre::Result<()> {
    let has_history: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await?;
    if has_history {
        let old_history: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = 2 AND description = 'full rewrite')",
        )
        .fetch_one(pool)
        .await?;
        if old_history {
            sqlx::query("DELETE FROM _sqlx_migrations")
                .execute(pool)
                .await?;
        }
    }
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
