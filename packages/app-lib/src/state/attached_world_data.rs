use crate::worlds::{DisplayStatus, WorldType};
use paste::paste;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AttachedWorldData {
    pub display_status: DisplayStatus,
    pub project_id: Option<String>,
    pub content_kind: Option<String>,
}

impl AttachedWorldData {
    pub async fn get_for_world(
        instance_id: &str,
        world_type: WorldType,
        world_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        let world_type = world_type.as_str();

        let attached_data =
            sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
                "
			SELECT display_status, project_id, content_kind
			FROM attached_world_data
			WHERE instance_id = ? and world_type = ? and world_id = ?
			",
            )
            .bind(instance_id)
            .bind(world_type)
            .bind(world_id)
            .fetch_optional(exec)
            .await?;

        Ok(
            attached_data.map(|(display_status, project_id, content_kind)| {
                AttachedWorldData {
                    display_status: DisplayStatus::from_string(&display_status),
                    project_id,
                    content_kind,
                }
            }),
        )
    }

    pub async fn get_all_for_instance(
        instance_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<HashMap<(WorldType, String), Self>> {
        let attached_data = sqlx::query_as::<
            _,
            (String, String, String, Option<String>, Option<String>),
        >(
            "
			SELECT world_type, world_id, display_status, project_id, content_kind
			FROM attached_world_data
			WHERE instance_id = ?
			",
        )
        .bind(instance_id)
        .fetch_all(exec)
        .await?;

        Ok(attached_data
            .into_iter()
            .map(
                |(
                    world_type,
                    world_id,
                    display_status,
                    project_id,
                    content_kind,
                )| {
                    let world_type = WorldType::from_string(&world_type);
                    let display_status =
                        DisplayStatus::from_string(&display_status);
                    (
                        (world_type, world_id),
                        AttachedWorldData {
                            display_status,
                            project_id,
                            content_kind,
                        },
                    )
                },
            )
            .collect())
    }

    pub async fn remove_for_world(
        instance_id: &str,
        world_type: WorldType,
        world_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let world_type = world_type.as_str();

        sqlx::query(
            "
			DELETE FROM attached_world_data
			WHERE instance_id = ? and world_type = ? and world_id = ?
			",
        )
        .bind(instance_id)
        .bind(world_type)
        .bind(world_id)
        .execute(exec)
        .await?;

        Ok(())
    }
}

macro_rules! attached_data_setter {
    ($parameter:ident: $parameter_type:ty, $column:expr $(=> $adapter:expr)?) => {
        paste! {
            pub async fn [<set_ $parameter>](
				instance_id: &str,
				world_type: WorldType,
				world_id: &str,
                $parameter: $parameter_type,
                exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
            ) -> crate::Result<()> {
                let world_type = world_type.as_str();
                $(let $parameter = $adapter;)?

				let query = format!(
					"INSERT INTO attached_world_data (instance_id, world_type, world_id, {0})\n\
					VALUES (?, ?, ?, ?)\n\
					ON CONFLICT (instance_id, world_type, world_id) DO UPDATE\n\
						SET {0} = excluded.{0}",
					$column,
				);

				sqlx::query(&query)
				.bind(instance_id)
				.bind(world_type)
				.bind(world_id)
				.bind($parameter)
				.execute(exec)
				.await?;

                Ok(())
            }
        }
    }
}

attached_data_setter!(display_status: DisplayStatus, "display_status" => display_status.as_str());
attached_data_setter!(project_id: &str, "project_id");
attached_data_setter!(content_kind: &str, "content_kind");
