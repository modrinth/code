use crate::state::DirectoryInfo;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use std::time::Duration;

pub(crate) async fn connect() -> crate::Result<Pool<Sqlite>> {
    let settings_dir = DirectoryInfo::get_initial_settings_dir().ok_or(
        crate::ErrorKind::FSError(
            "Could not find valid config dir".to_string(),
        ),
    )?;

    if !settings_dir.exists() {
        crate::util::io::create_dir_all(&settings_dir).await?;
    }

    let uri = format!("sqlite:{}", settings_dir.join("app.db").display());

    if !Sqlite::database_exists(&uri).await? {
        Sqlite::create_database(&uri).await?;
    }

    let conn_options = SqliteConnectOptions::from_str(&uri)?
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .optimize_on_close(true, None);

    let pool = SqlitePoolOptions::new()
        .max_connections(100)
        .connect_with(conn_options)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
