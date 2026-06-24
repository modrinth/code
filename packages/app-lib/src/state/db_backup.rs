use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use std::path::{Path, PathBuf};
use std::time::Duration;

const CURRENT_APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) async fn maybe_backup_existing_app_db(
    db_path: &Path,
) -> crate::Result<()> {
    if !db_path.try_exists()? {
        tracing::debug!(
            "Skipping pre-migration app database backup because {} does not exist",
            db_path.display()
        );
        return Ok(());
    }

    tracing::debug!(
        "Inspecting {} for a pre-migration app database backup",
        db_path.display()
    );

    let mut conn = match open_read_only_db(db_path).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(
                "Failed to open {} read-only before migrations: {err}",
                db_path.display()
            );
            return Err(err);
        }
    };

    let has_user_tables = match has_user_tables(&mut conn).await {
        Ok(has_user_tables) => has_user_tables,
        Err(err) => {
            tracing::error!(
                "Failed to inspect app database tables before migrations: {err}"
            );
            return Err(err);
        }
    };
    if !has_user_tables {
        tracing::debug!(
            "Skipping pre-migration app database backup because {} has no app data tables",
            db_path.display()
        );
        return Ok(());
    }

    let stored_version = match read_stored_app_version(&mut conn).await {
        Ok(version) => version,
        Err(err) => {
            tracing::error!(
                "Failed to read stored app database version before migrations: {err}"
            );
            return Err(err);
        }
    };
    if stored_version.as_deref() == Some(CURRENT_APP_VERSION) {
        tracing::debug!(
            "Skipping pre-migration app database backup because app version is already recorded as {CURRENT_APP_VERSION}"
        );
        return Ok(());
    }

    let stored_version = stored_version.as_deref().unwrap_or("unknown");
    let backup_dir = match app_db_backup_dir() {
        Ok(path) => path,
        Err(err) => {
            tracing::error!(
                "Failed to resolve app database backup directory before migrations: {err}"
            );
            return Err(err);
        }
    };
    let backup_path = match next_backup_path(
        &backup_dir,
        stored_version,
        CURRENT_APP_VERSION,
    )
    .await
    {
        Ok(path) => path,
        Err(err) => {
            tracing::error!(
                "Failed to choose app database backup path in {} before migrations: {err}",
                backup_dir.display()
            );
            return Err(err);
        }
    };

    tracing::info!(
        "Creating pre-migration app database backup from version {stored_version} before opening with version {CURRENT_APP_VERSION} at {}",
        backup_path.display()
    );

    if let Err(err) = create_sqlite_snapshot(&mut conn, &backup_path).await {
        tracing::error!(
            "Failed to create pre-migration app database backup at {}: {err}",
            backup_path.display()
        );
        return Err(err);
    }

    tracing::info!(
        "Created pre-migration app database backup at {}",
        backup_path.display()
    );

    Ok(())
}

async fn open_read_only_db(db_path: &Path) -> crate::Result<SqliteConnection> {
    let conn_options = SqliteConnectOptions::new()
        .filename(db_path)
        .busy_timeout(Duration::from_secs(30))
        .read_only(true)
        .create_if_missing(false);

    Ok(conn_options.connect().await?)
}

pub fn app_db_backup_dir() -> crate::Result<PathBuf> {
    if let Some(path) = std::env::var_os("THESEUS_DB_BACKUP_DIR") {
        return Ok(PathBuf::from(path));
    }

    let base = dirs::data_local_dir().or_else(dirs::data_dir).ok_or(
        crate::ErrorKind::FSError(
            "Could not find valid data dir for app database backups"
                .to_string(),
        ),
    )?;

    Ok(base.join("Modrinth").join("Backups").join("app-db"))
}

async fn has_user_tables(conn: &mut SqliteConnection) -> crate::Result<bool> {
    let count = sqlx::query_scalar!(
        "
		SELECT COUNT(*)
		FROM sqlite_master
		WHERE type = 'table'
			AND name NOT LIKE 'sqlite_%'
			AND name NOT IN ('_sqlx_migrations', 'app_metadata')
		",
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(count > 0)
}

async fn read_stored_app_version(
    conn: &mut SqliteConnection,
) -> crate::Result<Option<String>> {
    if !has_table(conn, "app_metadata").await? {
        return Ok(None);
    }

    Ok(sqlx::query_scalar!(
        "SELECT value FROM app_metadata WHERE key = 'app_version'"
    )
    .fetch_optional(&mut *conn)
    .await?)
}

async fn has_table(
    conn: &mut SqliteConnection,
    table_name: &str,
) -> crate::Result<bool> {
    let count = sqlx::query_scalar!(
        "
		SELECT COUNT(*)
		FROM sqlite_master
		WHERE type = 'table' AND name = ?
		",
        table_name,
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(count > 0)
}

async fn next_backup_path(
    backup_dir: &Path,
    stored_version: &str,
    current_version: &str,
) -> crate::Result<PathBuf> {
    crate::util::io::create_dir_all(backup_dir).await?;

    let stored_version = sanitize_version_for_filename(stored_version);
    let current_version = sanitize_version_for_filename(current_version);

    let backup_path = backup_dir.join(format!(
        "app-db-before-{current_version}-from-{stored_version}.db"
    ));
    if !backup_path.try_exists()? {
        return Ok(backup_path);
    }

    for suffix in 2.. {
        let backup_path = backup_dir.join(format!(
            "app-db-before-{current_version}-from-{stored_version}-{suffix}.db"
        ));
        if !backup_path.try_exists()? {
            return Ok(backup_path);
        }
    }

    unreachable!()
}

fn sanitize_version_for_filename(version: &str) -> String {
    let mut sanitized = String::new();
    let mut replaced_last_char = false;

    for character in version.chars() {
        if character.is_ascii_alphanumeric()
            || character == '.'
            || character == '-'
            || character == '_'
        {
            sanitized.push(character);
            replaced_last_char = false;
        } else if !replaced_last_char {
            sanitized.push('-');
            replaced_last_char = true;
        }
    }

    let sanitized = sanitized.trim_matches(&['.', '-', '_'][..]);
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized.to_string()
    }
}

async fn create_sqlite_snapshot(
    conn: &mut SqliteConnection,
    backup_path: &Path,
) -> crate::Result<()> {
    let backup_path = backup_path
        .to_str()
        .ok_or_else(|| crate::ErrorKind::UTFError(backup_path.to_path_buf()))?;

    sqlx::query!("VACUUM INTO ?", backup_path)
        .execute(&mut *conn)
        .await?;

    Ok(())
}
