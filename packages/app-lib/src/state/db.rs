use crate::state::DirectoryInfo;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

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

    let pool = SqlitePoolOptions::new()
        .max_connections(100)
        .connect(&uri)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
