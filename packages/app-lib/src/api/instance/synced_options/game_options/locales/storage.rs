use super::{Observation, Origin};
use sqlx::{Sqlite, SqlitePool, Transaction};

pub(super) struct OriginRow {
    pub scope: String,
    pub option_id: String,
    pub source_instance_id: Option<String>,
    pub source_game_version: Option<String>,
    pub backfilled: bool,
    pub observation: Option<Observation>,
    pub origin: Option<Origin>,
}

#[tracing::instrument(skip_all, fields(scope), err)]
pub(super) async fn load(
    pool: &SqlitePool,
    scope: Option<&str>,
) -> crate::Result<Vec<OriginRow>> {
    let rows = sqlx::query!(
        r#"SELECT scope, option_id, source_instance_id, source_game_version,
		backfilled AS "backfilled!: bool", observation_json, origin_json
		FROM game_option_locale_origins
		WHERE (? IS NULL OR scope = ?)
		ORDER BY scope, option_id"#,
        scope,
        scope,
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().filter(|row| {
		row.option_id.strip_prefix("external:").and_then(super::mod_translation_key).is_some()
	}).map(|row| {
		let observation = row.observation_json.as_deref().map(serde_json::from_str).transpose()
			.inspect_err(|error| tracing::warn!(%error, scope = row.scope, option_id = row.option_id,
				"Game setting locales: stored observation could not be decoded"))?;
		let origin = row.origin_json.as_deref().map(serde_json::from_str).transpose()
			.inspect_err(|error| tracing::warn!(%error, scope = row.scope, option_id = row.option_id,
				"Game setting locales: stored origin could not be decoded"))?;
		Ok(OriginRow {
			scope: row.scope,
			option_id: row.option_id,
			source_instance_id: row.source_instance_id,
			source_game_version: row.source_game_version,
			backfilled: row.backfilled,
			observation,
			origin,
		})
	}).collect()
}

#[tracing::instrument(
    skip_all,
    fields(scope, option_id, instance_id, game_version),
    err
)]
pub(super) async fn observe(
    tx: &mut Transaction<'_, Sqlite>,
    scope: &str,
    option_id: &str,
    instance_id: &str,
    game_version: &str,
    observation: &Observation,
) -> crate::Result<()> {
    let json = serde_json::to_string(observation)?;
    sqlx::query!(
		"INSERT INTO game_option_locale_origins
		(scope, option_id, source_instance_id, source_game_version, observation_json)
		SELECT ?, ?, ?, ?, ?
		WHERE ? <> '' OR EXISTS (SELECT 1 FROM synced_game_option_values WHERE option_id = ?)
		ON CONFLICT(scope, option_id) DO UPDATE SET observation_json = excluded.observation_json
		WHERE game_option_locale_origins.observation_json IS NULL
		AND game_option_locale_origins.origin_json IS NULL
		AND game_option_locale_origins.backfilled = 0
		AND game_option_locale_origins.source_instance_id = excluded.source_instance_id
		AND game_option_locale_origins.source_game_version = excluded.source_game_version",
		scope, option_id, instance_id, game_version, json, scope, option_id,
	).execute(&mut **tx).await?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(scope = row.scope, option_id = row.option_id), err)]
pub(super) async fn pin(
    pool: &SqlitePool,
    row: &OriginRow,
    origin: &Origin,
) -> crate::Result<()> {
    let json = serde_json::to_string(origin)?;
    let result = sqlx::query!(
        "UPDATE game_option_locale_origins SET origin_json = ?
		WHERE scope = ? AND option_id = ? AND origin_json IS NULL",
        json,
        row.scope,
        row.option_id,
    )
    .execute(pool)
    .await?;
    tracing::debug!(
        rows_affected = result.rows_affected(),
        key = origin.translation_key,
        "Game setting locales: origin pin completed"
    );
    Ok(())
}
