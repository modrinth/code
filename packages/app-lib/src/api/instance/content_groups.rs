use crate::{ErrorKind, State};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const MAX_GROUP_NAME_LENGTH: usize = 256;
const MAX_CONTENT_TYPE_LENGTH: usize = 32;
const MAX_ITEM_KEY_LENGTH: usize = 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstanceContentGroup {
    pub id: String,
    pub content_type: String,
    pub name: String,
    pub item_keys: Vec<String>,
}

fn validate_group_name(name: &str) -> crate::Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        bail!(ErrorKind::InputError(
            "group name cannot be empty".to_string(),
        ));
    }
    if name.chars().count() > MAX_GROUP_NAME_LENGTH {
        bail!(ErrorKind::InputError(format!(
            "group name cannot exceed {MAX_GROUP_NAME_LENGTH} characters"
        )));
    }
    Ok(name)
}

fn validate_content_type(content_type: &str) -> crate::Result<()> {
    if content_type.trim().is_empty()
        || content_type.len() > MAX_CONTENT_TYPE_LENGTH
    {
        bail!(ErrorKind::InputError(format!(
            "invalid content type {content_type}"
        )));
    }
    Ok(())
}

fn normalize_item_keys(item_keys: Vec<String>) -> crate::Result<Vec<String>> {
    let mut unique = HashSet::new();
    let mut normalized = Vec::with_capacity(item_keys.len());
    for item_key in item_keys {
        if item_key.trim().is_empty() || item_key.len() > MAX_ITEM_KEY_LENGTH {
            bail!(ErrorKind::InputError(
                "content group item key is invalid".to_string(),
            ));
        }
        if unique.insert(item_key.clone()) {
            normalized.push(item_key);
        }
    }
    Ok(normalized)
}

pub async fn list_content_groups(
    instance_id: String,
) -> crate::Result<Vec<InstanceContentGroup>> {
    let state = State::get().await?;
    let groups = sqlx::query!(
        "
		SELECT id, content_type, name
		FROM instance_content_groups
		WHERE instance_id = ?
		ORDER BY content_type, display_order, rowid
		",
        instance_id,
    )
    .fetch_all(&state.pool)
    .await?;
    let memberships = sqlx::query!(
        "
		SELECT group_id, item_key
		FROM instance_content_group_memberships
		WHERE instance_id = ?
		ORDER BY item_key
		",
        instance_id,
    )
    .fetch_all(&state.pool)
    .await?;

    let mut item_keys_by_group = HashMap::<String, Vec<String>>::new();
    for membership in memberships {
        item_keys_by_group
            .entry(membership.group_id)
            .or_default()
            .push(membership.item_key);
    }

    Ok(groups
        .into_iter()
        .map(|group| InstanceContentGroup {
            item_keys: item_keys_by_group.remove(&group.id).unwrap_or_default(),
            id: group.id,
            content_type: group.content_type,
            name: group.name,
        })
        .collect())
}

pub async fn create_content_group(
    instance_id: String,
    content_type: String,
    name: String,
    item_keys: Vec<String>,
) -> crate::Result<InstanceContentGroup> {
    validate_content_type(&content_type)?;
    let name = validate_group_name(&name)?;
    let item_keys = normalize_item_keys(item_keys)?;
    let state = State::get().await?;
    let id = Uuid::new_v4().to_string();
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;

    sqlx::query!(
        "
		UPDATE instance_content_groups
		SET display_order = display_order + 1
		WHERE instance_id = ? AND content_type = ?
		",
        instance_id,
        content_type,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "
		INSERT INTO instance_content_groups (id, instance_id, content_type, name, display_order)
		VALUES (?, ?, ?, ?, 0)
		",
        id,
        instance_id,
        content_type,
        name,
    )
    .execute(&mut *tx)
    .await?;
    set_memberships(
        &instance_id,
        &content_type,
        &item_keys,
        Some(&id),
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    Ok(InstanceContentGroup {
        id,
        content_type,
        name: name.to_string(),
        item_keys,
    })
}

pub async fn rename_content_group(
    id: String,
    new_name: String,
) -> crate::Result<()> {
    let new_name = validate_group_name(&new_name)?;
    let state = State::get().await?;
    let result = sqlx::query!(
        "UPDATE instance_content_groups SET name = ? WHERE id = ?",
        new_name,
        id,
    )
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        bail!(ErrorKind::InputError(format!("unknown content group {id}")));
    }
    Ok(())
}

pub async fn delete_content_group(id: String) -> crate::Result<()> {
    let state = State::get().await?;
    let result =
        sqlx::query!("DELETE FROM instance_content_groups WHERE id = ?", id)
            .execute(&state.pool)
            .await?;
    if result.rows_affected() == 0 {
        bail!(ErrorKind::InputError(format!("unknown content group {id}")));
    }
    Ok(())
}

pub async fn set_content_group_order(
    instance_id: String,
    content_type: String,
    group_ids: Vec<String>,
) -> crate::Result<()> {
    validate_content_type(&content_type)?;
    let state = State::get().await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    let existing_ids = sqlx::query_scalar!(
        "
		SELECT id
		FROM instance_content_groups
		WHERE instance_id = ? AND content_type = ?
		",
        instance_id,
        content_type,
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .collect::<HashSet<_>>();
    let ordered_ids = group_ids.iter().cloned().collect::<HashSet<_>>();
    if ordered_ids.len() != group_ids.len() || ordered_ids != existing_ids {
        bail!(ErrorKind::InputError(
            "content group order must list every group exactly once"
                .to_string(),
        ));
    }

    for (index, group_id) in group_ids.iter().enumerate() {
        let display_order = index as i64;
        sqlx::query!(
            "UPDATE instance_content_groups SET display_order = ? WHERE id = ?",
            display_order,
            group_id,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn set_content_group_memberships(
    instance_id: String,
    content_type: String,
    item_keys: Vec<String>,
    group_id: Option<String>,
) -> crate::Result<()> {
    validate_content_type(&content_type)?;
    let item_keys = normalize_item_keys(item_keys)?;
    if item_keys.is_empty() {
        return Ok(());
    }

    let state = State::get().await?;
    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    set_memberships(
        &instance_id,
        &content_type,
        &item_keys,
        group_id.as_deref(),
        &mut tx,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn set_memberships(
    instance_id: &str,
    content_type: &str,
    item_keys: &[String],
    group_id: Option<&str>,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    if let Some(group_id) = group_id {
        ensure_group_matches(group_id, instance_id, content_type, tx).await?;
    }

    for item_key in item_keys {
        match group_id {
            Some(group_id) => {
                sqlx::query!(
                    "
					INSERT INTO instance_content_group_memberships (instance_id, content_type, item_key, group_id)
					VALUES (?, ?, ?, ?)
					ON CONFLICT (instance_id, content_type, item_key)
					DO UPDATE SET group_id = excluded.group_id
					",
                    instance_id,
                    content_type,
                    item_key,
                    group_id,
                )
                .execute(&mut **tx)
                .await?;
            }
            None => {
                sqlx::query!(
                    "
					DELETE FROM instance_content_group_memberships
					WHERE instance_id = ? AND content_type = ? AND item_key = ?
					",
                    instance_id,
                    content_type,
                    item_key,
                )
                .execute(&mut **tx)
                .await?;
            }
        }
    }
    Ok(())
}

async fn ensure_group_matches(
    group_id: &str,
    instance_id: &str,
    content_type: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let exists = sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1
			FROM instance_content_groups
			WHERE id = ? AND instance_id = ? AND content_type = ?
		) AS "exists!: bool"
		"#,
        group_id,
        instance_id,
        content_type,
    )
    .fetch_one(&mut **tx)
    .await?;
    if !exists {
        bail!(ErrorKind::InputError(format!(
            "unknown content group {group_id}"
        )));
    }
    Ok(())
}
