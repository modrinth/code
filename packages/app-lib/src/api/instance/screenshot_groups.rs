use crate::State;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::screenshot_rows;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

const MAX_GROUP_NAME_LENGTH: usize = 256;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScreenshotGroup {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScreenshotGroupMembershipUpdate {
    pub screenshot_id: String,
    pub group_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScreenshotGroupImport {
    pub id: String,
    pub name: String,
    pub screenshot_ids: Vec<String>,
}

fn validate_group_name(name: &str) -> crate::Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "Group name cannot be empty".to_string(),
        )
        .into());
    }
    if name.chars().count() > MAX_GROUP_NAME_LENGTH {
        return Err(crate::ErrorKind::InputError(format!(
            "Group name cannot exceed {MAX_GROUP_NAME_LENGTH} characters"
        ))
        .into());
    }
    Ok(name)
}

pub async fn list_screenshot_groups() -> crate::Result<Vec<ScreenshotGroup>> {
    let state = State::get().await?;
    Ok(screenshot_rows::list_groups(&state.pool)
        .await?
        .into_iter()
        .map(|group| ScreenshotGroup {
            id: group.id,
            name: group.name,
        })
        .collect())
}

pub async fn create_screenshot_group(
    name: String,
    screenshot_ids: Vec<String>,
) -> crate::Result<ScreenshotGroup> {
    let name = validate_group_name(&name)?;
    let state = State::get().await?;
    let id = Uuid::new_v4().to_string();
    let mut tx = state.pool.begin().await?;

    sqlx::query!(
        "UPDATE screenshot_groups SET display_order = display_order + 1",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO screenshot_groups (id, name, display_order) VALUES (?, ?, 0)",
        id,
        name,
    )
    .execute(&mut *tx)
    .await?;

    let instance_ids = set_group_members(
        &id,
        normalize_screenshot_ids(screenshot_ids)?,
        &mut tx,
    )
    .await?;
    tx.commit().await?;
    emit_screenshot_updates(instance_ids).await;

    Ok(ScreenshotGroup {
        id,
        name: name.to_string(),
    })
}

pub async fn rename_screenshot_group(
    id: String,
    new_name: String,
) -> crate::Result<ScreenshotGroup> {
    let new_name = validate_group_name(&new_name)?;
    let state = State::get().await?;
    let result = sqlx::query!(
        "UPDATE screenshot_groups SET name = ? WHERE id = ?",
        new_name,
        id,
    )
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown screenshot group {id}"
        ))
        .into());
    }

    Ok(ScreenshotGroup {
        id,
        name: new_name.to_string(),
    })
}

pub async fn delete_screenshot_group(id: String) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_ids = sqlx::query_scalar!(
        "
		SELECT DISTINCT screenshots.instance_id
		FROM screenshot_group_memberships memberships
		INNER JOIN screenshots ON screenshots.id = memberships.screenshot_id
		WHERE memberships.group_id = ?
		",
        id,
    )
    .fetch_all(&state.pool)
    .await?;
    let result =
        sqlx::query!("DELETE FROM screenshot_groups WHERE id = ?", id,)
            .execute(&state.pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown screenshot group {id}"
        ))
        .into());
    }
    emit_screenshot_updates(instance_ids).await;
    Ok(())
}

pub async fn set_screenshot_group_memberships(
    updates: Vec<ScreenshotGroupMembershipUpdate>,
) -> crate::Result<()> {
    let updates = normalize_membership_updates(updates)?;
    if updates.is_empty() {
        return Ok(());
    }

    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;
    let group_ids = updates
        .iter()
        .filter_map(|update| update.group_id.as_deref())
        .collect::<HashSet<_>>();
    for group_id in group_ids {
        ensure_group_exists(group_id, &mut tx).await?;
    }

    let mut instance_ids = HashSet::new();
    for update in updates {
        let instance_id =
            screenshot_instance_id(&update.screenshot_id, &mut tx).await?;
        instance_ids.insert(instance_id);
        sqlx::query!(
            "DELETE FROM screenshot_group_memberships WHERE screenshot_id = ?",
            update.screenshot_id,
        )
        .execute(&mut *tx)
        .await?;
        if let Some(group_id) = update.group_id {
            sqlx::query!(
                "INSERT INTO screenshot_group_memberships (screenshot_id, group_id) VALUES (?, ?)",
                update.screenshot_id,
                group_id,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    emit_screenshot_updates(instance_ids).await;
    Ok(())
}

pub async fn import_screenshot_groups(
    groups: Vec<ScreenshotGroupImport>,
) -> crate::Result<()> {
    if groups.is_empty() {
        return Ok(());
    }

    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;
    let base_order = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(display_order) + 1, 0) AS 'display_order!: i64' FROM screenshot_groups",
    )
    .fetch_one(&mut *tx)
    .await?;
    let mut group_ids = HashSet::new();
    let mut affected_instance_ids = HashSet::new();

    for (index, group) in groups.into_iter().enumerate() {
        if group.id.trim().is_empty() || !group_ids.insert(group.id.clone()) {
            return Err(crate::ErrorKind::InputError(
                "Screenshot group import contains an invalid group ID"
                    .to_string(),
            )
            .into());
        }
        let name = validate_group_name(&group.name)?;
        let display_order = base_order + index as i64;
        sqlx::query!(
            "
			INSERT INTO screenshot_groups (id, name, display_order)
			VALUES (?, ?, ?)
			ON CONFLICT(id) DO UPDATE SET name = excluded.name
			",
            group.id,
            name,
            display_order,
        )
        .execute(&mut *tx)
        .await?;
        affected_instance_ids.extend(
            set_group_members(
                &group.id,
                normalize_screenshot_ids(group.screenshot_ids)?,
                &mut tx,
            )
            .await?,
        );
    }
    tx.commit().await?;
    emit_screenshot_updates(affected_instance_ids).await;
    Ok(())
}

fn normalize_screenshot_ids(
    screenshot_ids: Vec<String>,
) -> crate::Result<Vec<String>> {
    let mut unique = HashSet::new();
    if screenshot_ids
        .iter()
        .all(|screenshot_id| unique.insert(screenshot_id.clone()))
    {
        return Ok(screenshot_ids);
    }
    Err(crate::ErrorKind::InputError(
        "Screenshot group contains duplicate screenshots".to_string(),
    )
    .into())
}

fn normalize_membership_updates(
    updates: Vec<ScreenshotGroupMembershipUpdate>,
) -> crate::Result<Vec<ScreenshotGroupMembershipUpdate>> {
    let mut screenshot_ids = HashSet::new();
    if updates
        .iter()
        .all(|update| screenshot_ids.insert(update.screenshot_id.clone()))
    {
        return Ok(updates);
    }
    Err(crate::ErrorKind::InputError(
        "Screenshot membership update contains duplicate screenshots"
            .to_string(),
    )
    .into())
}

async fn set_group_members(
    group_id: &str,
    screenshot_ids: Vec<String>,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<HashSet<String>> {
    let mut instance_ids = HashSet::new();
    for screenshot_id in screenshot_ids {
        instance_ids.insert(screenshot_instance_id(&screenshot_id, tx).await?);
        sqlx::query!(
            "
			INSERT INTO screenshot_group_memberships (screenshot_id, group_id)
			VALUES (?, ?)
			ON CONFLICT(screenshot_id) DO UPDATE SET group_id = excluded.group_id
			",
            screenshot_id,
            group_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(instance_ids)
}

async fn ensure_group_exists(
    group_id: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM screenshot_groups WHERE id = ?) AS "exists!: bool""#,
        group_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    if !exists {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown screenshot group {group_id}"
        ))
        .into());
    }
    Ok(())
}

async fn screenshot_instance_id(
    screenshot_id: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<String> {
    sqlx::query_scalar!(
        "SELECT instance_id FROM screenshots WHERE id = ?",
        screenshot_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unknown screenshot {screenshot_id}"
        ))
    })
    .map_err(Into::into)
}

async fn emit_screenshot_updates(
    instance_ids: impl IntoIterator<Item = String>,
) {
    for instance_id in instance_ids {
        let _ = emit_instance(
            &instance_id,
            InstancePayloadType::ScreenshotsUpdated,
        )
        .await;
    }
}
