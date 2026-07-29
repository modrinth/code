use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_GROUP_NAME_LENGTH: usize = 128;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceGroup {
    pub id: String,
    pub name: String,
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

    if name.eq_ignore_ascii_case("none") {
        return Err(crate::ErrorKind::InputError(
            "Group name cannot be None".to_string(),
        )
        .into());
    }

    Ok(name)
}

pub async fn list_groups() -> crate::Result<Vec<InstanceGroup>> {
    let state = State::get().await?;
    Ok(instance_rows::list_instance_groups(&state.pool)
        .await?
        .into_iter()
        .map(|(id, name)| InstanceGroup { id, name })
        .collect())
}

pub async fn create_group(name: String) -> crate::Result<InstanceGroup> {
    let name = validate_group_name(&name)?;
    let state = State::get().await?;
    let id = Uuid::new_v4().to_string();
    instance_rows::create_instance_group(&id, name, &state.pool).await?;

    Ok(InstanceGroup {
        id,
        name: name.to_string(),
    })
}

pub async fn rename_group(
    id: String,
    new_name: String,
) -> crate::Result<InstanceGroup> {
    let new_name = validate_group_name(&new_name)?;
    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;

    let instance_ids = sqlx::query_scalar::<_, String>(
        "
		SELECT instance_id
		FROM instance_group_memberships
		WHERE group_id = ?
		",
    )
    .bind(&id)
    .fetch_all(&mut *tx)
    .await?;

    let result = sqlx::query(
        "
		UPDATE instance_groups
		SET name = ?
		WHERE id = ?
		",
    )
    .bind(new_name)
    .bind(&id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown instance group {id}"
        ))
        .into());
    }

    tx.commit().await?;

    for instance_id in instance_ids {
        emit_instance(&instance_id, InstancePayloadType::Edited).await?;
    }

    Ok(InstanceGroup {
        id,
        name: new_name.to_string(),
    })
}

pub async fn delete_group(id: String) -> crate::Result<()> {
    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;
    let instance_ids = sqlx::query_scalar::<_, String>(
        "
		SELECT instance_id
		FROM instance_group_memberships
		WHERE group_id = ?
		",
    )
    .bind(&id)
    .fetch_all(&mut *tx)
    .await?;

    let result = sqlx::query(
        "
		DELETE FROM instance_groups
		WHERE id = ?
		",
    )
    .bind(&id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown instance group {id}"
        ))
        .into());
    }

    tx.commit().await?;

    for instance_id in instance_ids {
        emit_instance(&instance_id, InstancePayloadType::Edited).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_group_name;

    #[test]
    fn group_name_validation_trims_valid_names() {
        assert_eq!(validate_group_name("  My group  ").unwrap(), "My group");
    }

    #[test]
    fn group_name_validation_rejects_empty_names() {
        assert!(validate_group_name("   ").is_err());
    }

    #[test]
    fn group_name_validation_rejects_reserved_name() {
        assert!(validate_group_name("NoNe").is_err());
    }

    #[test]
    fn group_name_validation_rejects_long_names() {
        assert!(validate_group_name(&"a".repeat(33)).is_err());
    }
}
