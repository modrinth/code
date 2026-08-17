use crate::event::emit::emit_instance_groups_changed;
use crate::state::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

const MAX_GROUP_NAME_LENGTH: usize = 256;
pub const FAVORITES_GROUP_ID: &str = "group:favorites";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceGroup {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceGroupMembershipUpdate {
    pub instance_id: String,
    pub group_ids: Vec<String>,
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

fn normalize_membership_updates(
    mut updates: Vec<InstanceGroupMembershipUpdate>,
) -> crate::Result<Vec<InstanceGroupMembershipUpdate>> {
    let mut instance_ids = HashSet::new();
    for update in &mut updates {
        if !instance_ids.insert(update.instance_id.clone()) {
            return Err(crate::ErrorKind::InputError(format!(
                "Duplicate instance {} in group membership update",
                update.instance_id
            ))
            .into());
        }

        let mut group_ids = HashSet::new();
        update
            .group_ids
            .retain(|group_id| group_ids.insert(group_id.clone()));
    }

    Ok(updates)
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

pub async fn set_group_order(group_ids: Vec<String>) -> crate::Result<()> {
    let state = State::get().await?;
    instance_rows::set_instance_group_order(&group_ids, &state.pool).await
}

pub async fn set_group_memberships(
    updates: Vec<InstanceGroupMembershipUpdate>,
) -> crate::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }

    let updates = normalize_membership_updates(updates)?;

    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;
    let unique_group_ids = updates
        .iter()
        .flat_map(|update| update.group_ids.iter())
        .collect::<HashSet<_>>();

    for group_id in unique_group_ids {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM instance_groups WHERE id = ?) AS "exists!: bool""#,
            group_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        if !exists {
            return Err(crate::ErrorKind::InputError(format!(
                "Unknown instance group {group_id}"
            ))
            .into());
        }
    }

    for update in &updates {
        let result = sqlx::query!(
            "UPDATE instances SET modified = unixepoch() WHERE id = ?",
            update.instance_id,
        )
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            return Err(crate::ErrorKind::InputError(format!(
                "Unknown instance {}",
                update.instance_id
            ))
            .into());
        }

        instance_rows::replace_instance_groups(
            &update.instance_id,
            &update.group_ids,
            &mut tx,
        )
        .await?;
    }

    tx.commit().await?;

    let instance_ids = updates
        .into_iter()
        .map(|update| update.instance_id)
        .collect::<Vec<_>>();
    emit_instance_groups_changed(&instance_ids).await?;

    Ok(())
}

pub async fn rename_group(
    id: String,
    new_name: String,
) -> crate::Result<InstanceGroup> {
    if id == FAVORITES_GROUP_ID {
        return Err(crate::ErrorKind::InputError(
            "Favorites cannot be renamed".to_string(),
        )
        .into());
    }

    let new_name = validate_group_name(&new_name)?;
    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;

    let instance_ids = sqlx::query_scalar!(
        "
		SELECT instance_id
		FROM instance_group_memberships
		WHERE group_id = ?
		",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;

    let result = sqlx::query!(
        "
		UPDATE instance_groups
		SET name = ?
		WHERE id = ?
        ",
        new_name,
        id,
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown instance group {id}"
        ))
        .into());
    }

    tx.commit().await?;

    emit_instance_groups_changed(&instance_ids).await?;

    Ok(InstanceGroup {
        id,
        name: new_name.to_string(),
    })
}

pub async fn delete_group(id: String) -> crate::Result<()> {
    if id == FAVORITES_GROUP_ID {
        return Err(crate::ErrorKind::InputError(
            "Favorites cannot be deleted".to_string(),
        )
        .into());
    }

    let state = State::get().await?;
    let mut tx = state.pool.begin().await?;
    let instance_ids = sqlx::query_scalar!(
        "
		SELECT instance_id
		FROM instance_group_memberships
		WHERE group_id = ?
		",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;

    let result = sqlx::query!(
        "
        DELETE FROM instance_groups
        WHERE id = ?
        ",
        id,
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Unknown instance group {id}"
        ))
        .into());
    }

    tx.commit().await?;

    emit_instance_groups_changed(&instance_ids).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        FAVORITES_GROUP_ID, InstanceGroupMembershipUpdate,
        MAX_GROUP_NAME_LENGTH, normalize_membership_updates,
        validate_group_name,
    };

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
    fn group_name_validation_allows_favorites() {
        assert_eq!(validate_group_name("Favorites").unwrap(), "Favorites");
    }

    #[test]
    fn group_name_validation_rejects_long_names() {
        assert!(
            validate_group_name(&"a".repeat(MAX_GROUP_NAME_LENGTH + 1))
                .is_err()
        );
    }

    #[test]
    fn favorites_group_id_is_stable() {
        assert_eq!(FAVORITES_GROUP_ID, "group:favorites");
    }

    #[test]
    fn membership_updates_reject_duplicate_instances() {
        let updates = vec![
            InstanceGroupMembershipUpdate {
                instance_id: "instance".to_string(),
                group_ids: vec!["first".to_string()],
            },
            InstanceGroupMembershipUpdate {
                instance_id: "instance".to_string(),
                group_ids: vec!["second".to_string()],
            },
        ];

        assert!(normalize_membership_updates(updates).is_err());
    }

    #[test]
    fn membership_updates_deduplicate_groups() {
        let updates =
            normalize_membership_updates(vec![InstanceGroupMembershipUpdate {
                instance_id: "instance".to_string(),
                group_ids: vec![
                    "first".to_string(),
                    "second".to_string(),
                    "first".to_string(),
                ],
            }])
            .unwrap();

        assert_eq!(
            updates[0].group_ids,
            vec!["first".to_string(), "second".to_string()]
        );
    }
}
