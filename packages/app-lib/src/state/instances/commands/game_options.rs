use crate::state::{GameOptionKind, StoredOption, StoredPreference};
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Returns whether an instance has supplied the first set of shared values.
///
/// The values are kept when sync is turned off, so this is separate from checking
/// whether sync is currently enabled.
pub(crate) async fn shared_game_options_exist(
    pool: &SqlitePool,
) -> crate::Result<bool> {
    let exists = sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1 FROM synced_game_option_state WHERE singleton = 1
		) AS "exists!: bool"
		"#,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// Returns whether game-settings sync is enabled for the app.
pub(crate) async fn game_options_sync_is_enabled(
    pool: &SqlitePool,
) -> crate::Result<bool> {
    let enabled = sqlx::query_scalar!(
        r#"
		SELECT globally_enabled AS "globally_enabled!: bool"
		FROM sync_feature_settings
		WHERE feature = 'game_options'
		"#,
    )
    .fetch_optional(pool)
    .await?
    .unwrap_or(false);
    Ok(enabled)
}

/// Loads two version numbers: one for the shared values and one for the list of
/// Minecraft settings Modrinth knows how to sync.
///
/// If the code contains a newer list than the database remembers, use the newer
/// version without writing to the database just to update this number.
pub(crate) async fn load_game_options_sync_state(
    pool: &SqlitePool,
    current_catalog_revision: u32,
) -> crate::Result<(u64, u32)> {
    let row = sqlx::query!(
        r#"
		SELECT revision AS "revision!: i64",
			catalog_revision AS "catalog_revision!: i64"
		FROM synced_game_option_state
		WHERE singleton = 1
		"#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row
        .map(|row| {
            (
                row.revision.max(0) as u64,
                (row.catalog_revision.max(1) as u32)
                    .max(current_catalog_revision),
            )
        })
        .unwrap_or((0, current_catalog_revision)))
}

/// Loads every shared setting, including ones that are currently turned off or
/// belong to mods that are not installed in any synced instance right now.
pub(crate) async fn load_shared_game_options(
    pool: &SqlitePool,
) -> crate::Result<HashMap<String, StoredOption>> {
    let rows = sqlx::query!(
        r#"
		SELECT option_id, kind, raw_key, canonical_value_json,
			seeded AS "seeded!: bool", revision AS "revision!: i64"
		FROM synced_game_option_values
		"#,
    )
    .fetch_all(pool)
    .await?;
    let mut values = HashMap::with_capacity(rows.len());
    for row in rows {
        let value = row
            .canonical_value_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?;
        values.insert(
            row.option_id.clone(),
            StoredOption {
                option_id: row.option_id,
                kind: if row.kind == "external" {
                    GameOptionKind::External
                } else {
                    GameOptionKind::Vanilla
                },
                raw_key: row.raw_key,
                value,
                seeded: row.seeded,
                revision: row.revision.max(0) as u64,
            },
        );
    }
    Ok(values)
}

/// Loads whether sync is turned on for each setting.
pub(crate) async fn load_game_option_preferences(
    pool: &SqlitePool,
) -> crate::Result<HashMap<String, StoredPreference>> {
    let rows = sqlx::query!(
        r#"
		SELECT option_id, enabled AS "enabled!: bool",
			revision AS "revision!: i64"
		FROM synced_game_option_preferences
		"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.option_id,
                StoredPreference {
                    enabled: row.enabled,
                    revision: row.revision.max(0) as u64,
                },
            )
        })
        .collect())
}
