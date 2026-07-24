use crate::state::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use uuid::Uuid;

const MAX_GROUP_NAME_LENGTH: usize = 32;

pub async fn list_groups() -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    instance_rows::list_instance_group_names(&state.pool).await
}

pub async fn create_group(name: String) -> crate::Result<String> {
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

    let state = State::get().await?;
    let existing_group = sqlx::query_scalar::<_, String>(
        "
		SELECT name
		FROM instance_groups
		WHERE lower(name) = lower(?)
		",
    )
    .bind(name)
    .fetch_optional(&state.pool)
    .await?;

    if existing_group.is_some() {
        return Err(crate::ErrorKind::InputError(
            "A group with this name already exists".to_string(),
        )
        .into());
    }

    instance_rows::create_instance_group(
        &Uuid::new_v4().to_string(),
        name,
        &state.pool,
    )
    .await?;

    Ok(name.to_string())
}
