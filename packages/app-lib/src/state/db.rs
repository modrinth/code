use crate::state::DirectoryInfo;
use sqlx::sqlite::{
	SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::time::Duration;

pub(crate) async fn connect(
	app_identifier: &str,
) -> crate::Result<Pool<Sqlite>> {
	let settings_dir = DirectoryInfo::initial_settings_dir_path(app_identifier)
		.ok_or(crate::ErrorKind::FSError(
			"Could not find valid config dir".to_string(),
		))?;

	crate::util::io::create_dir_all(&settings_dir).await?;

	let db_path = settings_dir.join("app.db");

	connect_app_db(&db_path).await
}

async fn connect_app_db(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
	super::db_backup::maybe_backup_existing_app_db(db_path).await?;
	open_migrated_app_db(db_path).await
}

async fn open_migrated_app_db(
	db_path: &Path,
) -> crate::Result<Pool<Sqlite>> {
	let pool = open_app_db_pool(db_path).await?;

	if let Err(err) = stale_data_cleanup(&pool).await {
		tracing::warn!(
			"Failed to clean up stale data from state database before migrations: {err}"
		);
	}

	sqlx::migrate!().run(&pool).await?;
	record_current_app_version(&pool).await?;

	if let Err(err) = stale_data_cleanup(&pool).await {
		tracing::warn!(
			"Failed to clean up stale data from state database: {err}"
		);
	}

	Ok(pool)
}

async fn open_app_db_pool(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
	let conn_options = SqliteConnectOptions::new()
		.filename(db_path)
		.busy_timeout(Duration::from_secs(30))
		.journal_mode(SqliteJournalMode::Wal)
		.optimize_on_close(true, None)
		.create_if_missing(true);

	Ok(SqlitePoolOptions::new()
		.max_connections(100)
		.connect_with(conn_options)
		.await?)
}

async fn record_current_app_version(
	pool: &Pool<Sqlite>,
) -> crate::Result<()> {
	sqlx::query(
		"
		INSERT INTO app_metadata (key, value, updated_at)
		VALUES ('app_version', ?, unixepoch())
		ON CONFLICT(key) DO UPDATE SET
			value = excluded.value,
			updated_at = excluded.updated_at
		",
	)
	.bind(env!("CARGO_PKG_VERSION"))
	.execute(pool)
	.await?;

	Ok(())
}

/// Cleans up data from the database that is no longer referenced, but must be
/// kept around for a little while to allow users to recover from accidental
/// deletions.
async fn stale_data_cleanup(pool: &Pool<Sqlite>) -> crate::Result<()> {
	let mut tx = pool.begin().await?;

	let has_skin_tables = sqlx::query!(
		"SELECT COUNT(*) AS \"count!: i64\" FROM sqlite_master WHERE type = 'table' AND name IN ('custom_minecraft_skins', 'minecraft_users')",
	)
	.fetch_one(&mut *tx)
	.await?
	.count == 2;

	if has_skin_tables {
		sqlx::query!(
			"DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
		)
		.execute(&mut *tx)
		.await?;
	}

	tx.commit().await?;

	Ok(())
}
