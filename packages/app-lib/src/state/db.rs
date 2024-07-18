use crate::state::DirectoryInfo;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Connection, Pool, Sqlite, SqliteConnection};

pub(crate) async fn connect() -> crate::Result<Pool<Sqlite>> {
    let uri =
        format!("sqlite:{}", DirectoryInfo::get_database_file()?.display());

    if !Sqlite::database_exists(&uri).await? {
        Sqlite::create_database(&uri).await?;
    }

    let mut conn: SqliteConnection = SqliteConnection::connect(&uri).await?;
    sqlx::migrate!().run(&mut conn).await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(100)
        .connect(&uri)
        .await?;

    Ok(pool)
}
