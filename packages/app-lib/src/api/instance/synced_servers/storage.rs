use super::super::synced_options::{
    nbt_from_bytes, nbt_to_bytes, safe_instance_id, synced_options_path,
};
use super::SERVERS_FILE;
use super::types::{
    CanonicalServer, LocalServer, ProjectionEntry, ProjectionOwner,
    ServerRecord, ServerSource,
};
use crate::{ErrorKind, State};
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;
use std::path::PathBuf;

pub(in crate::api::instance) async fn canonical_exists(
    state: &State,
) -> crate::Result<bool> {
    canonical_initialized(state).await
}

async fn canonical_initialized(state: &State) -> crate::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1 FROM synced_server_state WHERE singleton = 1
		) AS "initialized!: bool"
		"#,
    )
    .fetch_one(&state.pool)
    .await?)
}

pub(super) async fn read_canonical(
    state: &State,
) -> crate::Result<Vec<CanonicalServer>> {
    let rows = sqlx::query!(
        "
		SELECT id, nbt
		FROM synced_servers
		ORDER BY position
		",
    )
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(CanonicalServer {
                id: row.id,
                data: nbt_from_bytes(row.nbt)?,
            })
        })
        .collect()
}

pub(super) async fn commit_server_state(
    canonical: Option<&[CanonicalServer]>,
    local: Option<(&str, &[LocalServer])>,
    state: &State,
) -> crate::Result<bool> {
    let mut tx = state.pool.begin().await?;
    let mut canonical_changed = false;
    if let Some(canonical) = canonical {
        canonical_changed = write_canonical_rows(&mut tx, canonical).await?;
        sqlx::query!(
            "
			INSERT INTO synced_server_state (singleton, revision)
			VALUES (1, 0)
			ON CONFLICT(singleton) DO NOTHING
			",
        )
        .execute(&mut *tx)
        .await?;
        if canonical_changed {
            sqlx::query!(
                "
				UPDATE synced_server_state
				SET revision = revision + 1
				WHERE singleton = 1
				",
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    if let Some((instance_id, local)) = local {
        write_local_rows(&mut tx, instance_id, local).await?;
    }
    tx.commit().await?;
    Ok(canonical_changed)
}

async fn write_canonical_rows(
    tx: &mut Transaction<'_, Sqlite>,
    servers: &[CanonicalServer],
) -> crate::Result<bool> {
    let current = sqlx::query!(
        "
		SELECT id, position, nbt
		FROM synced_servers
		ORDER BY position
		",
    )
    .fetch_all(&mut **tx)
    .await?;
    let mut desired = Vec::with_capacity(servers.len());
    for (position, server) in servers.iter().enumerate() {
        desired.push((
            server.id.as_str(),
            position as i64,
            nbt_to_bytes(&server.data)?,
        ));
    }
    if current.len() == desired.len()
        && current.iter().zip(&desired).all(|(current, desired)| {
            current.id == desired.0
                && current.position == desired.1
                && current.nbt == desired.2
        })
    {
        return Ok(false);
    }

    let desired_ids =
        desired.iter().map(|(id, _, _)| *id).collect::<HashSet<_>>();
    for row in &current {
        if !desired_ids.contains(row.id.as_str()) {
            sqlx::query!("DELETE FROM synced_servers WHERE id = ?", row.id)
                .execute(&mut **tx)
                .await?;
        }
    }

    let position_offset = current
        .iter()
        .map(|row| row.position)
        .max()
        .unwrap_or(0)
        .max(servers.len() as i64)
        .saturating_add(1);
    sqlx::query!(
        "UPDATE synced_servers SET position = position + ?",
        position_offset,
    )
    .execute(&mut **tx)
    .await?;

    for (id, position, nbt) in desired {
        sqlx::query!(
            "
			INSERT INTO synced_servers (id, position, nbt)
			VALUES (?, ?, ?)
			ON CONFLICT(id) DO UPDATE SET
				position = excluded.position,
				nbt = excluded.nbt
			",
            id,
            position,
            nbt,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(true)
}

pub(super) async fn server_revision(state: &State) -> crate::Result<i64> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT revision AS "revision!: i64"
		FROM synced_server_state
		WHERE singleton = 1
		"#,
    )
    .fetch_optional(&state.pool)
    .await?
    .unwrap_or(0))
}

pub(super) async fn load_local(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<LocalServer>> {
    let rows = sqlx::query!(
        "
		SELECT id, source, excluded_synced_server_id, nbt, position
		FROM instance_servers
		WHERE instance_id = ?
		ORDER BY position
		",
        instance_id,
    )
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(LocalServer {
                id: row.id,
                source: ServerSource::from_str(&row.source).ok_or_else(
                    || {
                        ErrorKind::InputError(format!(
                            "Unknown server source {}",
                            row.source
                        ))
                    },
                )?,
                excluded_synced_server_id: row.excluded_synced_server_id,
                data: nbt_from_bytes(row.nbt)?,
                position: row.position,
            })
        })
        .collect()
}

pub(super) async fn write_local(
    instance_id: &str,
    servers: &[LocalServer],
    state: &State,
) -> crate::Result<()> {
    commit_server_state(None, Some((instance_id, servers)), state).await?;
    Ok(())
}

pub(super) async fn write_local_rows(
    tx: &mut Transaction<'_, Sqlite>,
    instance_id: &str,
    servers: &[LocalServer],
) -> crate::Result<()> {
    sqlx::query!(
        "DELETE FROM instance_servers WHERE instance_id = ?",
        instance_id,
    )
    .execute(&mut **tx)
    .await?;
    for server in servers {
        let source = server.source.as_str();
        let nbt = nbt_to_bytes(&server.data)?;
        sqlx::query!(
            "
			INSERT INTO instance_servers
				(instance_id, id, source, excluded_synced_server_id,
				 nbt, position)
			VALUES (?, ?, ?,
				(SELECT id FROM synced_servers WHERE id = ?), ?, ?)
			",
            instance_id,
            server.id,
            source,
            server.excluded_synced_server_id,
            nbt,
            server.position,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(super) async fn load_projection_entries(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<ProjectionEntry>> {
    let rows = sqlx::query!(
        "
		SELECT server_id, owner, nbt, position
		FROM instance_server_projection_entries
		WHERE instance_id = ?
		ORDER BY position
		",
        instance_id,
    )
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(ProjectionEntry {
                id: row.server_id,
                owner: ProjectionOwner::from_str(&row.owner).ok_or_else(
                    || {
                        ErrorKind::InputError(format!(
                            "Unknown server projection owner {}",
                            row.owner
                        ))
                    },
                )?,
                data: nbt_from_bytes(row.nbt)?,
                position: row.position,
            })
        })
        .collect()
}

pub(super) async fn begin_server_checkpoint(
    instance_id: &str,
    servers: &[ServerRecord],
    expected_sha1: &str,
    source_revision: i64,
    state: &State,
) -> crate::Result<()> {
    let mut tx = state.pool.begin().await?;
    sqlx::query!(
        "DELETE FROM instance_server_projection_entries WHERE instance_id = ?",
        instance_id,
    )
    .execute(&mut *tx)
    .await?;
    for (position, server) in servers.iter().enumerate() {
        let owner = if server.source == ServerSource::UserSynced {
            ProjectionOwner::Synced
        } else {
            ProjectionOwner::Instance
        }
        .as_str();
        let nbt = nbt_to_bytes(&server.data)?;
        let position = position as i64;
        sqlx::query!(
            "
			INSERT INTO instance_server_projection_entries
				(instance_id, owner, server_id, nbt, position)
			VALUES (?, ?, ?, ?, ?)
			",
            instance_id,
            owner,
            server.id,
            nbt,
            position,
        )
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query!(
        "
		INSERT INTO instance_sync_checkpoints
			(instance_id, feature, variant, expected_sha1, merge_base,
			 source_revision, status, link_mode)
		VALUES (?, 'multiplayer_servers', 'default', ?, NULL, ?,
			'pending', NULL)
		ON CONFLICT(instance_id, feature, variant) DO UPDATE SET
			expected_sha1 = excluded.expected_sha1,
			merge_base = NULL,
			source_revision = excluded.source_revision,
			status = 'pending',
			link_mode = NULL
		",
        instance_id,
        expected_sha1,
        source_revision,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub(super) fn generated_path(state: &State, instance_id: &str) -> PathBuf {
    synced_options_path(state)
        .join("servers/generated")
        .join(safe_instance_id(instance_id))
        .join(SERVERS_FILE)
}
