//! Keeps the app-wide fullscreen setting aligned with the shared Minecraft value.

use super::CATALOG_REVISION;
use crate::state::{
    CanonicalValue, State, load_game_option_preferences,
    load_game_options_sync_state, load_shared_game_options,
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};

const FULLSCREEN_OPTION_ID: &str = "fullscreen";

pub(crate) async fn shared_fullscreen_value(
    state: &State,
) -> crate::Result<Option<bool>> {
    let preferences = load_game_option_preferences(&state.pool).await?;
    if !preferences
        .get(FULLSCREEN_OPTION_ID)
        .is_some_and(|preference| preference.enabled)
    {
        return Ok(None);
    }
    let values = load_shared_game_options(&state.pool).await?;
    Ok(values.get(FULLSCREEN_OPTION_ID).and_then(|stored| {
        if !stored.seeded {
            return None;
        }
        match stored.value.as_ref() {
            Some(CanonicalValue::Bool(value)) => Some(*value),
            _ => None,
        }
    }))
}

pub(crate) async fn update_shared_fullscreen_from_app(
    state: &State,
    value: bool,
) -> crate::Result<bool> {
    let values = load_shared_game_options(&state.pool).await?;
    let preferences = load_game_option_preferences(&state.pool).await?;
    let stored = values.get(FULLSCREEN_OPTION_ID);
    let preference = preferences.get(FULLSCREEN_OPTION_ID);
    let value_matches = stored.is_some_and(|stored| {
        stored.seeded
            && matches!(
                stored.value.as_ref(),
                Some(CanonicalValue::Bool(current)) if *current == value
            )
    });
    let sync_enabled = preference.is_some_and(|preference| preference.enabled);
    if !sync_enabled || value_matches {
        return Ok(false);
    }

    let option_revision = stored
        .map(|stored| stored.revision)
        .unwrap_or(0)
        .max(
            preference
                .map(|preference| preference.revision)
                .unwrap_or(0),
        )
        .saturating_add(1) as i64;
    let (canonical_revision, _) =
        load_game_options_sync_state(&state.pool, CATALOG_REVISION).await?;
    let next_canonical_revision = canonical_revision.saturating_add(1) as i64;
    let canonical = CanonicalValue::Bool(value);
    let value_json = serde_json::to_string(&canonical)?;
    let canonical_type = canonical.type_name();
    let now = Utc::now().timestamp();
    let catalog_revision = CATALOG_REVISION as i64;
    let kind = "vanilla";
    let raw_key: Option<&str> = None;
    let value_codec = "catalog";
    let source_game_version: Option<&str> = None;
    let mut tx = state.pool.begin().await?;

    sqlx::query!(
        "\n\t\t\t\tINSERT INTO synced_game_option_values\n\t\t\t\t\t(option_id, kind, raw_key, canonical_type,\n\t\t\t\t\t canonical_value_json, value_codec, seeded, revision, origin,\n\t\t\t\t\t source_game_version, source_instance_id, updated_at)\n\t\t\t\tVALUES (?, ?, ?, ?, ?, ?, 1, ?, 'app_editor', ?, NULL, ?)\n\t\t\t\tON CONFLICT(option_id) DO UPDATE SET\n\t\t\t\t\tkind = excluded.kind, raw_key = excluded.raw_key,\n\t\t\t\t\tcanonical_type = excluded.canonical_type,\n\t\t\t\t\tcanonical_value_json = excluded.canonical_value_json,\n\t\t\t\t\tvalue_codec = excluded.value_codec, seeded = 1,\n\t\t\t\t\trevision = excluded.revision, origin = 'app_editor',\n\t\t\t\t\tsource_game_version = NULL, source_instance_id = NULL,\n\t\t\t\t\tupdated_at = excluded.updated_at\n\t\t\t\t",
        FULLSCREEN_OPTION_ID,
        kind,
        raw_key,
        canonical_type,
        value_json,
        value_codec,
        option_revision,
        source_game_version,
        now,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "\n\t\t\t\tINSERT INTO synced_game_option_preferences\n\t\t\t\t\t(option_id, enabled, source, revision)\n\t\t\t\tVALUES (?, ?, ?, ?)\n\t\t\t\tON CONFLICT(option_id) DO UPDATE SET\n\t\t\t\t\tenabled = excluded.enabled, source = 'user',\n\t\t\t\t\trevision = excluded.revision\n\t\t\t\t",
        FULLSCREEN_OPTION_ID,
        true,
        "user",
        option_revision,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "\n\t\t\tINSERT INTO synced_game_option_state (singleton, revision, catalog_revision)\n\t\t\tVALUES (1, ?, ?)\n\t\t\tON CONFLICT(singleton) DO UPDATE SET\n\t\t\t\trevision = excluded.revision,\n\t\t\t\tcatalog_revision = excluded.catalog_revision\n\t\t\t",
        next_canonical_revision,
        catalog_revision,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub(super) async fn update_app_fullscreen_setting(
    tx: &mut Transaction<'_, Sqlite>,
    value: &CanonicalValue,
    sync_enabled: bool,
) -> crate::Result<()> {
    if !sync_enabled {
        return Ok(());
    }
    let CanonicalValue::Bool(value) = value else {
        return Ok(());
    };
    let mut settings = crate::state::Settings::get(&mut **tx).await?;
    settings.force_fullscreen = *value;
    settings.update(&mut **tx).await?;
    Ok(())
}
